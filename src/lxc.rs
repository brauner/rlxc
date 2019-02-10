use failure::*;
use std::ffi::CString;
use std::ptr;

pub struct Lxc {
    handle: *mut lxc_sys::lxc_container,
}

impl Lxc {
    pub fn new(name: &str, path: &str) -> Lxc {
        let cname = CString::new(name).unwrap();
        let cpath = CString::new(path).unwrap();

        let handle = unsafe {
            lxc_sys::lxc_container_new(cname.as_ptr(), cpath.as_ptr())
        };

        Lxc { handle }
    }

    pub fn start(&mut self, stub: bool) -> Result<(), Error> {
        let useinit = if stub { 1 } else { 0 };
        let err = unsafe {
            (*self.handle).start.unwrap()(self.handle, useinit, ptr::null())
        };
        if !err {
            bail!("failed to start container");
        }
        Ok(())
    }
}
