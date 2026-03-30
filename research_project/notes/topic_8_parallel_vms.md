# Topic 8: Parallel VM Management and Scaling

## 1. libvirt/virsh Core Commands

```bash
# Lifecycle
virsh list --all
virsh start worker-01
virsh shutdown worker-01         # graceful
virsh destroy worker-01          # force kill
virsh autostart worker-01        # start on host boot

# Bulk start
for i in $(seq 1 10); do virsh start worker-$(printf '%02d' $i); done

# Snapshots (for reset to clean state)
virsh snapshot-create-as worker-01 snap-clean "clean state"
virsh snapshot-revert worker-01 snap-clean

# Stats
virsh domstats worker-01         # CPU/memory/net/disk stats
virsh dommemstat worker-01       # memory details
virsh vcpuinfo worker-01         # vCPU to pCPU mapping
```

## 2. Terraform + libvirt (Infrastructure as Code)

```hcl
terraform {
  required_providers {
    libvirt = { source = "dmacvicar/libvirt" }
  }
}

provider "libvirt" {
  uri = "qemu:///system"
}

# Base image (downloaded once)
resource "libvirt_volume" "base" {
  name   = "ubuntu-22.04-base.qcow2"
  pool   = "default"
  source = "https://cloud-images.ubuntu.com/jammy/current/jammy-server-cloudimg-amd64.img"
  format = "qcow2"
}

# Per-VM thin-provisioned overlay
resource "libvirt_volume" "worker" {
  for_each       = toset(["worker-01","worker-02","worker-03"])
  name           = "${each.key}.qcow2"
  pool           = "default"
  base_volume_id = libvirt_volume.base.id
  size           = 21474836480  # 20 GB
}

# Cloud-init per VM
resource "libvirt_cloudinit_disk" "init" {
  for_each  = toset(["worker-01","worker-02","worker-03"])
  name      = "${each.key}-cloudinit.iso"
  user_data = templatefile("cloud_init.cfg.tpl", { hostname = each.key })
}

# VM
resource "libvirt_domain" "worker" {
  for_each = toset(["worker-01","worker-02","worker-03"])
  name     = each.key
  memory   = 2048
  vcpu     = 2
  cloudinit = libvirt_cloudinit_disk.init[each.key].id
  disk { volume_id = libvirt_volume.worker[each.key].id }
  network_interface {
    network_name   = "worker-net"
    wait_for_lease = true
  }
}
```

Scale from 3 to 10 workers by changing one number and `terraform apply`.

## 3. Resource Limits Per VM

### CPU Pinning and Hard Caps
```xml
<cputune>
  <vcpupin vcpu='0' cpuset='2'/>
  <vcpupin vcpu='1' cpuset='3'/>
  <emulatorpin cpuset='2,3'/>
  <period>100000</period>
  <quota>100000</quota>   <!-- 100% of 1 core (50% of 2 cores) -->
</cputune>
```

Via virsh: `virsh schedinfo worker-01 --set vcpu_quota=100000 --set vcpu_period=200000`

### Disk I/O Throttling
```xml
<disk ...>
  <iotune>
    <total_iops_sec>500</total_iops_sec>
    <read_bytes_sec>52428800</read_bytes_sec>   <!-- 50 MB/s -->
    <write_bytes_sec>26214400</write_bytes_sec>  <!-- 25 MB/s -->
  </iotune>
</disk>
```

## 4. Network Isolation

```bash
# Create isolated network
cat > worker-net.xml << 'EOF'
<network>
  <name>worker-net</name>
  <bridge name='virbr10'/>
  <ip address='10.10.10.1' netmask='255.255.255.0'>
    <dhcp><range start='10.10.10.10' end='10.10.10.99'/></dhcp>
  </ip>
</network>
EOF
virsh net-define worker-net.xml
virsh net-start worker-net
virsh net-autostart worker-net
```

Isolation levels (most → least isolated):
1. Isolated network (VMs can only talk to each other on bridge)
2. NAT network (VMs reach internet via host NAT)
3. Bridged to physical NIC (VMs are first-class LAN hosts)
4. Macvtap (high-performance, but VMs can't talk to host)

## 5. Fast VM Cloning (qcow2 Backing Store)

```bash
# Prepare base (run once)
virsh shutdown worker-base
virt-sysprep -d worker-base   # Reset machine-id, SSH keys, logs

# Thin clone (takes ~1 second, uses ~1 MB initially)
qemu-img create -f qcow2 -F qcow2 \
  -b /var/lib/libvirt/images/base.qcow2 \
  /var/lib/libvirt/images/worker-06.qcow2

# Clone VM definition
virt-clone --original worker-base --name worker-06 --auto-clone
```

10 workers sharing 4 GB base = ~4 GB total instead of 40 GB (plus small overlay deltas).

**Important:** `virt-sysprep` must be run to reset `/etc/machine-id`, SSH host keys, etc. to avoid network conflicts.

## 6. Monitoring and Auto-Restart

### Prometheus + libvirt-exporter

```bash
docker run -d \
  -v /var/run/libvirt/libvirt-sock:/var/run/libvirt/libvirt-sock \
  -p 9177:9177 \
  ghcr.io/tinkoff/libvirt-exporter
```

Metrics exposed: CPU time, memory usage, disk I/O, network I/O, VM state.
Grafana dashboard IDs: 13633 and 15682.

### Auto-Restart (systemd wrapper)

```ini
# /etc/systemd/system/libvirt-vm@.service
[Unit]
Description=KVM VM %i
After=libvirtd.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/virsh start %i
ExecStop=/usr/bin/virsh shutdown %i
Restart=on-failure
RestartSec=10s

[Install]
WantedBy=multi-user.target
```

```bash
systemctl enable --now libvirt-vm@worker-01
```

Or set in VM XML: `<on_crash>restart</on_crash>`

### Watchdog Script (cron every 60s)

```bash
#!/bin/bash
WORKERS=(worker-01 worker-02 worker-03)
for vm in "${WORKERS[@]}"; do
  state=$(virsh domstate "$vm" 2>/dev/null)
  if [[ "$state" != "running" ]]; then
    echo "$(date): $vm is $state, restarting" >> /var/log/vm-watchdog.log
    virsh start "$vm"
  fi
done
```

## 7. VM Density on 32 GB RAM / 8-Core Host

RAM is almost always the binding constraint (not CPU).

| VM Config | VMs Possible | Notes |
|---|---|---|
| 512 MB RAM, 1 vCPU | 20–28 | Minimal workers |
| 1 GB RAM, 1 vCPU | 14–16 | Comfortable Linux server |
| 2 GB RAM, 2 vCPU | 8–10 | General-purpose workers |
| 4 GB RAM, 2 vCPU | 4–6 | Heavier workloads |

**Rules of thumb:**
1. Never allocate more than 80–85% of host RAM to VMs
2. CPU overcommit up to 3:1 safe for I/O-bound workloads
3. Use KSM + memory ballooning for effective overcommit
4. Use NVMe or SSD for VM images (HDD will thrash with 10+ VMs)
5. 10+ VMs on 1 GbE NIC may saturate it

**Realistic for 32 GB / 8-core:** **6–12 workers** with 2 GB RAM / 2 vCPU each is comfortable.

## Sources
- https://computingforgeeks.com/how-to-provision-vms-on-kvm-with-terraform/
- https://metamost.com/post/tech/libvirt-linked-clones/
- https://github.com/Tinkoff/libvirt-exporter
- https://grafana.com/grafana/dashboards/15682-libvirt/
- https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/8/html/configuring_and_managing_virtualization/
