use failure::*;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

pub struct Lxc {
    handle: *mut lxc_sys::lxc_container,
}

/// This iterates over a `char**`, consuming each contained string by returning
/// it as an owning CString. The pointer holding the list will also be freed in
/// `drop`.
struct AllocatedStringArrayIter {
    ptr: *mut *mut c_char,
    len: usize,
    at: usize,
}

impl AllocatedStringArrayIter {
    fn new(ptr: *mut *mut c_char, len: usize) -> Self {
        Self { ptr, len, at: 0 }
    }
}

impl Drop for AllocatedStringArrayIter {
    fn drop(&mut self) {
        for i in self.at..self.len {
            // If the list was consumed these pointers will be NULL and no harm
            // is done.
            unsafe {
                libc::free(self.ptr.add(i) as *mut _);
            }
        }
        unsafe {
            libc::free(self.ptr as *mut _);
        }
    }
}

impl Iterator for AllocatedStringArrayIter {
    type Item = CString;

    fn next(&mut self) -> Option<Self::Item> {
        let at = self.at;
        if at == self.len {
            None
        } else {
            self.at += 1;
            Some(unsafe { CString::from_raw(*self.ptr.add(at)) })
        }
    }
}

pub fn list_all_containers(path: &str) -> Result<(), Error> {
    let cpath = CString::new(path).unwrap();
    let mut names: *mut *mut c_char = ptr::null_mut();

    let nr = unsafe {
        lxc_sys::list_all_containers(
            cpath.as_ptr(),
            &mut names,
            ptr::null_mut(),
        )
    };

    for name in AllocatedStringArrayIter::new(names, nr as usize) {
        match name.to_str() {
            Ok(name) => println!("{}", name),
            Err(_) => println!("non-utf8 container name: {:?}", name),
        }
    }

    Ok(())
}

pub fn get_version() -> String {
    let cstr: &CStr = unsafe { CStr::from_ptr(lxc_sys::lxc_get_version()) };

    let str_slice = cstr.to_str().unwrap_or("unknown");
    str_slice.to_owned()
}

impl Lxc {
    pub fn new(name: &str, path: &str) -> Result<Lxc, Error> {
        let cname = CString::new(name).unwrap();
        let cpath = CString::new(path).unwrap();

        let handle = unsafe {
            lxc_sys::lxc_container_new(cname.as_ptr(), cpath.as_ptr())
        };

        if handle.is_null() {
            bail!("failed to allocate new container");
        }

        Ok(Lxc { handle })
    }

    pub fn start(&self, stub: bool) -> Result<(), Error> {
        let useinit = if stub { 1 } else { 0 };
        let started = unsafe {
            (*self.handle).start.unwrap()(self.handle, useinit, ptr::null())
        };
        if !started {
            bail!("failed to start container");
        }
        Ok(())
    }

    pub fn shutdown(&self, timeout: i32) -> Result<(), Error> {
        let down =
            unsafe { (*self.handle).shutdown.unwrap()(self.handle, timeout) };
        if !down {
            bail!("failed to shutdown container");
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<(), Error> {
        let stopped = unsafe { (*self.handle).stop.unwrap()(self.handle) };
        if !stopped {
            bail!("failed to start container");
        }
        Ok(())
    }

    pub fn may_control(&self) -> bool {
        unsafe { (*self.handle).may_control.unwrap()(self.handle) }
    }

    pub fn is_running(&self) -> bool {
        unsafe { (*self.handle).is_running.unwrap()(self.handle) }
    }
}
