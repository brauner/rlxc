#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    let name = CString::new("c1").unwrap();
    let path = CString::new("/home/brauner/.local/share/lxc").unwrap();
    unsafe {
        let container = lxc_container_new(name.as_ptr(), path.as_ptr());
        (*container).daemonize = true;
        container.is_running(container);
    }
}
