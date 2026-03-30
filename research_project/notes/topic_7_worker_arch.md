# Topic 7: Worker Architecture — Custom ISO vs Install Script

## Comparison Table

| Criterion | Custom ISO (Packer) | Cloud-Init + Ansible |
|---|---|---|
| Initial setup time | High (days) | Low (hours) |
| CI/CD integration | Packer + artifact store | `ansible-playbook`, straightforward |
| Update turnaround | Rebuild + redistribute + reflash (slow) | Push playbook to live fleet (minutes) |
| Configuration drift | None (immutable image) | Possible; mitigate with scheduled runs |
| Works offline/air-gapped | Yes | Needs APT mirror |
| Rollback | Reflash previous ISO | `git revert` + re-run playbook |
| Team onboarding | Must learn Packer + preseed | Ansible is industry-standard |
| PR review | Diffs in HCL + preseed | Full YAML diffs, very readable |
| Best for | Large stable fleets, air-gapped | Active development, changing requirements |

## Custom ISO Approach

**Tools:**
- **Packer** (recommended): Scriptable, version-controlled, headless. Supports QEMU/KVM builders.
- **Cubic**: GUI wizard on top of live-build. Not CI-friendly.
- **live-build**: Debian's native toolchain, verbose but flexible.

**Packer example (Ubuntu 22.04 autoinstall):**
```hcl
source "qemu" "ubuntu" {
  iso_url      = "https://releases.ubuntu.com/22.04/ubuntu-22.04-live-server-amd64.iso"
  iso_checksum = "sha256:..."
  boot_command = [
    "<wait>c<wait>",
    "linux /casper/vmlinuz autoinstall ds=nocloud-net;seedfrom=http://{{.HTTPIP}}:{{.HTTPPort}}/ ---",
    "<enter><wait>",
    "initrd /casper/initrd<enter><wait>",
    "boot<enter>"
  ]
  http_directory = "http"
}
```

**How HiveOS (mining) does it:** Ships a stripped Ubuntu ISO with all software pre-baked. Users flash + add `rig.conf` for identity. Worker auto-registers on first boot via config file. This is the gold standard for large stable fleets.

## Cloud-Init + Install Script Approach (Recommended for Development)

### cloud-init user-data example:
```yaml
#cloud-config
hostname: worker-01
users:
  - name: worker
    groups: sudo
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys:
      - ssh-ed25519 AAAA...

package_update: true
packages:
  - qemu-kvm
  - libvirt-daemon-system
  - python3
  - curl

runcmd:
  - curl -fsSL https://install.example.com/worker.sh | bash
  - systemctl enable --now worker-agent
```

For local VMs: deliver via second virtual CD-ROM (nocloud datasource).

### Ansible for fleet management:
```yaml
- hosts: workers
  tasks:
    - name: Install KVM packages
      apt:
        name:
          - qemu-kvm
          - libvirt-daemon-system
        state: present

    - name: Deploy worker service
      copy:
        src: worker-agent
        dest: /usr/local/bin/worker-agent
        mode: '0755'

    - name: Start worker service
      systemd:
        name: worker-agent
        enabled: yes
        state: started
```

## Recommended Hybrid Approach

1. **Packer builds a "golden base image"** with OS + common dependencies pre-installed
2. **Cloud-init handles per-worker identity** on first boot (hostname, credentials, server URL)
3. **Ansible handles day-2 operations** (software updates, config changes)

This gives the speed of custom images with the flexibility of scripts.

## Dependency Management

**For install script approach:**
```bash
#!/bin/bash
# worker-install.sh

set -e

# System dependencies
apt-get update
apt-get install -y \
    python3 python3-pip \
    libvirt-daemon-system \
    qemu-kvm \
    qemu-utils \
    virtinst

# Python worker agent
pip3 install worker-agent-package

# Configure
cat > /etc/worker/config.json << EOF
{
  "server": "${SERVER_URL}",
  "worker_id": "${WORKER_ID:-$(hostname)}"
}
EOF

# Enable service
systemctl enable --now worker-agent
```

Run as: `SERVER_URL=http://server:8080 bash worker-install.sh`

## Sources
- https://developer.hashicorp.com/packer/guides/automatic-operating-system-installs/preseed_ubuntu
- https://ubuntu.com/server/docs/explanation/intro-to/cloud-init/
- https://hiveon.com/features/ (HiveOS mining OS example)
- https://github.com/canonical/packer-maas
