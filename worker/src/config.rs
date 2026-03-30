use crate::spoof::HwIdentity;

/// VM configuration parameters.
#[derive(Debug, Clone)]
pub struct VmConfig {
    pub name: String,
    pub ram_mb: u32,
    pub vcpus: u32,
    pub disk_path: String,
    pub vnc_port: u16,
    pub hw: HwIdentity,
    pub virtiofs_source: Option<String>,
    pub virtiofs_tag: Option<String>,
}

impl VmConfig {
    /// Produce libvirt-compatible XML domain definition.
    pub fn to_xml(&self) -> String {
        let virtiofs_xml = match (&self.virtiofs_source, &self.virtiofs_tag) {
            (Some(src), Some(tag)) => format!(
                r#"
    <filesystem type='mount' accessmode='passthrough'>
      <driver type='virtiofs'/>
      <source dir='{src}'/>
      <target dir='{tag}'/>
    </filesystem>"#
            ),
            _ => String::new(),
        };

        format!(
            r#"<domain type='kvm'>
  <name>{name}</name>
  <memory unit='MiB'>{ram}</memory>
  <vcpu placement='static'>{vcpus}</vcpu>

  <os>
    <type arch='x86_64' machine='q35'>hvm</type>
    <boot dev='hd'/>
  </os>

  <features>
    <acpi/>
    <apic/>
    <kvm>
      <hidden state='on'/>
    </kvm>
  </features>

  <cpu mode='host-passthrough' check='none'>
    <feature policy='disable' name='hypervisor'/>
  </cpu>

  <sysinfo type='smbios'>
    <system>
      <entry name='manufacturer'>{smbios_mfr}</entry>
      <entry name='product'>{smbios_prod}</entry>
      <entry name='serial'>{smbios_serial}</entry>
    </system>
  </sysinfo>

  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>

    <disk type='file' device='disk'>
      <driver name='qemu' type='qcow2' discard='unmap'/>
      <source file='{disk}'/>
      <target dev='vda' bus='virtio'/>
      <serial>{disk_serial}</serial>
    </disk>

    <interface type='network'>
      <mac address='{mac}'/>
      <source network='default'/>
      <model type='e1000e'/>
    </interface>

    <graphics type='vnc' port='{vnc_port}' autoport='no' listen='127.0.0.1'/>
    <video>
      <model type='virtio' heads='1'/>
    </video>

    <input type='tablet' bus='virtio'/>
    <input type='keyboard' bus='virtio'/>{virtiofs}

    <channel type='unix'>
      <target type='virtio' name='org.qemu.guest_agent.0'/>
    </channel>

    <memballoon model='virtio'/>
  </devices>
</domain>"#,
            name = self.name,
            ram = self.ram_mb,
            vcpus = self.vcpus,
            smbios_mfr = xml_escape(&self.hw.smbios_manufacturer),
            smbios_prod = xml_escape(&self.hw.smbios_product),
            smbios_serial = xml_escape(&self.hw.smbios_serial),
            disk = self.disk_path,
            disk_serial = self.hw.disk_serial,
            mac = self.hw.mac_address,
            vnc_port = self.vnc_port,
            virtiofs = virtiofs_xml,
        )
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spoof;

    fn sample_config() -> VmConfig {
        let hw = spoof::generate_identity("test-vm");
        VmConfig {
            name: "test-vm".into(),
            ram_mb: 2048,
            vcpus: 2,
            disk_path: "/var/lib/vmctl/disks/test-vm.qcow2".into(),
            vnc_port: 5901,
            hw,
            virtiofs_source: Some("/opt/cs2".into()),
            virtiofs_tag: Some("cs2".into()),
        }
    }

    #[test]
    fn test_xml_contains_name() {
        let xml = sample_config().to_xml();
        assert!(xml.contains("<name>test-vm</name>"));
    }

    #[test]
    fn test_xml_contains_kvm_hidden() {
        let xml = sample_config().to_xml();
        assert!(xml.contains("<hidden state='on'/>"));
    }

    #[test]
    fn test_xml_contains_e1000e() {
        let xml = sample_config().to_xml();
        assert!(xml.contains("type='e1000e'"));
    }

    #[test]
    fn test_xml_contains_smbios() {
        let cfg = sample_config();
        let xml = cfg.to_xml();
        assert!(xml.contains(&cfg.hw.smbios_manufacturer));
        assert!(xml.contains(&cfg.hw.smbios_serial));
    }

    #[test]
    fn test_xml_contains_virtiofs() {
        let xml = sample_config().to_xml();
        assert!(xml.contains("type='virtiofs'"));
        assert!(xml.contains("dir='cs2'"));
    }

    #[test]
    fn test_xml_no_virtiofs_when_none() {
        let mut cfg = sample_config();
        cfg.virtiofs_source = None;
        cfg.virtiofs_tag = None;
        let xml = cfg.to_xml();
        assert!(!xml.contains("virtiofs"));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("A & B"), "A &amp; B");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_xml_contains_guest_agent_channel() {
        let xml = sample_config().to_xml();
        assert!(xml.contains("org.qemu.guest_agent.0"));
        assert!(xml.contains("<channel type='unix'>"));
    }

    #[test]
    fn test_xml_is_parseable() {
        let xml = sample_config().to_xml();
        // Basic well-formedness: starts and ends correctly
        assert!(xml.starts_with("<domain type='kvm'>"));
        assert!(xml.trim().ends_with("</domain>"));
    }
}
