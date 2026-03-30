Title: virt - Rust

URL Source: https://docs.rs/virt/latest/virt/

Markdown Content:
Expand description

A Rust bindings for libvirt.

Libvirt is a portable toolkit to interact with the virtualisation capabilities of Linux, Solaris and other operating systems.

The binding tries to be a fairly direct mapping of the underling C API with some differences to respect Rust conventions. So for example C functions related to a domain like: `virDomainCreate` will be mapped in the binding like `dom.create()` or `virDomainPinVcpu` as `dom.pin_vcpu`.

The binding uses standard errors handling from Rust. Each method (there are some exceptions) is returning a type `Option` or `Result`.

```
use virt::connect::Connect;

if let Ok(mut conn) = Connect::open(Some("test:///default")) {
  assert_eq!(Ok(0), conn.close());
}
```

Most of the structs are automatically release their references by implemementing `Drop` trait but for structs which are reference counted at C level, it is still possible to explicitly release the reference at Rust level. For instance if a Rust method returns a *Domain, it is possible to call `free` on it when no longer required.

```
use virt::connect::Connect;
use virt::domain::Domain;

if let Ok(mut conn) = Connect::open(Some("test:///default")) {
  if let Ok(mut dom) = Domain::lookup_by_name(&conn, "myguest") {
      assert_eq!(Ok(()), dom.free());   // Explicitly releases memory at Rust level.
      assert_eq!(Ok(0), conn.close());
  }
}
```

For each methods accepting or returning a virTypedParameter array a new Rust struct has been defined where each attribute is handling a type Option.

```
use virt::connect::Connect;
use virt::domain::Domain;

if let Ok(mut conn) = Connect::open(Some("test://default")) {
  if let Ok(dom) = Domain::lookup_by_name(&conn, "myguest") {
    if let Ok(memp) = dom.get_memory_parameters(0) {
      if memp.hard_limit.is_some() {
        println!("hard limit: {}", memp.hard_limit.unwrap())
      }
    }
  }
  assert_eq!(Ok(0), conn.close());
}
```

`pub extern crate virt_sys as sys;`[connect](https://docs.rs/virt/latest/virt/connect/index.html "mod virt::connect")[domain](https://docs.rs/virt/latest/virt/domain/index.html "mod virt::domain")[domain_ snapshot](https://docs.rs/virt/latest/virt/domain_snapshot/index.html "mod virt::domain_snapshot")[error](https://docs.rs/virt/latest/virt/error/index.html "mod virt::error")[event](https://docs.rs/virt/latest/virt/event/index.html "mod virt::event")[interface](https://docs.rs/virt/latest/virt/interface/index.html "mod virt::interface")[network](https://docs.rs/virt/latest/virt/network/index.html "mod virt::network")[nodedev](https://docs.rs/virt/latest/virt/nodedev/index.html "mod virt::nodedev")[nwfilter](https://docs.rs/virt/latest/virt/nwfilter/index.html "mod virt::nwfilter")[secret](https://docs.rs/virt/latest/virt/secret/index.html "mod virt::secret")[storage_ pool](https://docs.rs/virt/latest/virt/storage_pool/index.html "mod virt::storage_pool")[storage_ vol](https://docs.rs/virt/latest/virt/storage_vol/index.html "mod virt::storage_vol")[stream](https://docs.rs/virt/latest/virt/stream/index.html "mod virt::stream")[param_ field_ in](https://docs.rs/virt/latest/virt/macro.param_field_in.html "macro virt::param_field_in")[param_ field_ out](https://docs.rs/virt/latest/virt/macro.param_field_out.html "macro virt::param_field_out")
