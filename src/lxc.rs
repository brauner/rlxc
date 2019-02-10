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

    pub fn start(&self, stub: bool) -> Result<(), Error> {
        let useinit = if stub { 1 } else { 0 };
        let err = unsafe {
            (*self.handle).start.unwrap()(self.handle, useinit, ptr::null())
        };
        if !err {
            bail!("failed to start container");
        }
        Ok(())
    }

    pub fn shutdown(&self, timeout: i32) -> bool {
        unsafe { (*self.handle).shutdown.unwrap()(self.handle, timeout) }
    }

    pub fn stop(&self) -> bool {
        unsafe { (*self.handle).stop.unwrap()(self.handle) }
    }

    pub fn may_control(&self) -> bool {
        unsafe { (*self.handle).may_control.unwrap()(self.handle) }
    }

    pub fn is_running(&self) -> bool {
        unsafe { (*self.handle).is_running.unwrap()(self.handle) }
    }
}
