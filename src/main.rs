use std::ffi::CString;

fn main() {
    let name = CString::new("c1").unwrap();
    let path = CString::new("/home/brauner/.local/share/lxc").unwrap();
    let container =
        unsafe { lxc_sys::lxc_container_new(name.as_ptr(), path.as_ptr()) };
    unsafe {
        (*container).daemonize = true;
        //container.is_running(container);
    }
}
