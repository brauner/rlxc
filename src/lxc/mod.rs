// SPDX-License-Identifier: LGPL-2.1+

//! Rust wrapper for `struct lxc_container`. Implements methods to control
//! containers.

use failure::*;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::ptr;
use std::time::Duration;

use crate::util::ffi::{StringArrayIter, ToCString};

mod attach_options;
pub use attach_options::*;

/// The main container handle. This implements the methods for `struct
/// lxc_container`.
pub struct Lxc {
    handle: *mut lxc_sys::lxc_container,
}

impl Drop for Lxc {
    fn drop(&mut self) {
        unsafe {
            lxc_sys::lxc_container_put(self.handle);
        }
    }
}

/// Get an iterator over all containers defined in the given `path`. This is a
/// wrapper for liblxc's `list_all_containers` function.
pub fn list_all_containers<T: AsRef<Path>>(
    path: T,
) -> Result<StringArrayIter, Error> {
    let cpath = path.as_ref().to_c_string()?;
    let mut names: *mut *mut c_char = ptr::null_mut();

    let nr = unsafe {
        lxc_sys::list_all_containers(
            cpath.as_ptr(),
            &mut names,
            ptr::null_mut(),
        )
    };

    if nr < 0 {
        bail!("failed to list containers");
    }
    Ok(StringArrayIter::new(names, nr as usize))
}

/// Returns the currently used liblxc's version string.
pub fn get_version() -> &'static str {
    let cstr: &CStr = unsafe { CStr::from_ptr(lxc_sys::lxc_get_version()) };
    cstr.to_str().unwrap_or("unknown")
}

pub fn get_global_config_item(key: &str) -> Result<&'static str, Error> {
    let ckey = CString::new(key).unwrap();
    let cstr: &CStr = unsafe {
        CStr::from_ptr(lxc_sys::lxc_get_global_config_item(ckey.as_ptr()))
    };
    if cstr.as_ptr().is_null() {
        bail!("failed to find value of {}", key);
    }
    Ok(cstr.to_str().unwrap())
}

pub fn get_default_path() -> &'static str {
    let path = match get_global_config_item("lxc.lxcpath") {
        Ok(s) => s,
        Err(_) => return "",
    };

    path
}

impl Lxc {
    /// Create a new container handler for the container of the given `name`
    /// residing under the provided `path`.
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

    /// Attempt to start the container. If `stub` is true, the container's
    /// `lxc.execute.cmd` is executed instead of `lxc.init.cmd`.
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

    /// Atetmpt to shutdown a container with a timeout.
    pub fn shutdown(&self, timeout: Option<Duration>) -> Result<(), Error> {
        let timeout: c_int = match timeout {
            Some(to) => {
                let secs = to.as_secs();
                // seconds can be large...
                if secs > (!(0 as c_int)) as u64 {
                    bail!("timeout too large");
                }
                secs as _
            }
            None => -1,
        };
        let down =
            unsafe { (*self.handle).shutdown.unwrap()(self.handle, timeout) };
        if !down {
            bail!("failed to shutdown container");
        }
        Ok(())
    }

    /// Attempt to stop a running container.
    pub fn stop(&self) -> Result<(), Error> {
        let stopped = unsafe { (*self.handle).stop.unwrap()(self.handle) };
        if !stopped {
            bail!("failed to start container");
        }
        Ok(())
    }

    /// Determine if the caller may control the container.
    pub fn may_control(&self) -> bool {
        unsafe { (*self.handle).may_control.unwrap()(self.handle) }
    }

    /// Determine if the container is running.
    pub fn is_running(&self) -> bool {
        unsafe { (*self.handle).is_running.unwrap()(self.handle) }
    }

    /// Try to run a program inside the container.
    pub fn attach_run_wait(
        &self,
        options: &mut AttachOptions,
        program: &str,
        argv: Vec<&str>,
    ) -> i32 {
        let cprogram = CString::new(program).unwrap();
        let cargv: Vec<_> =
            argv.iter().map(|arg| CString::new(*arg).unwrap()).collect();

        let mut args: Vec<_> = cargv.iter().map(|arg| arg.as_ptr()).collect();
        args.push(std::ptr::null());

        unsafe {
            (*self.handle).attach_run_wait.unwrap()(
                self.handle,
                options.raw(),
                cprogram.as_ptr(),
                args.as_ptr(),
            )
        }
    }

    /// Determine state of container.
    pub fn state(&self) -> &'static str {
        let cstr: &CStr = unsafe {
            CStr::from_ptr((*self.handle).state.unwrap()(self.handle))
        };
        cstr.to_str().unwrap_or("UNKNOWN")
    }

    /// Get network interfaces of container.
    pub fn get_interfaces(&self) -> StringArrayIter {
        let mut len = 0;
        let names: *mut *mut c_char =
            unsafe { (*self.handle).get_interfaces.unwrap()(self.handle) };

        if names != ptr::null_mut() {
            unsafe {
                for i in 0.. {
                    if *names.add(i) == ptr::null_mut() {
                        break;
                    }
                    len += 1;
                }
            };
        }
        StringArrayIter::new(names, len)
    }

    /// Get ip addresses of an interface.
    pub fn get_ipv4(&self, interface: &str) -> StringArrayIter {
        let iface = CString::new(interface).unwrap();

        let mut len = 0;
        let addresses: *mut *mut c_char = unsafe {
            (*self.handle).get_ips.unwrap()(
                self.handle,
                iface.as_ptr(),
                CString::new("inet").unwrap().as_ptr(),
                0,
            )
        };

        if addresses != ptr::null_mut() {
            unsafe {
                for i in 0.. {
                    if *addresses.add(i) == ptr::null_mut() {
                        // Since the string array is NULL-terminated so free the last element here.
                        libc::free(*addresses.add(i) as *mut _);
                        break;
                    }
                    len += 1;
                }
            };
        }
        StringArrayIter::new(addresses, len)
    }

    /// Get ip addresses of an interface.
    pub fn get_ipv6(&self, interface: &str) -> StringArrayIter {
        let iface = CString::new(interface).unwrap();

        let mut len = 0;
        let addresses: *mut *mut c_char = unsafe {
            (*self.handle).get_ips.unwrap()(
                self.handle,
                iface.as_ptr(),
                CString::new("inet6").unwrap().as_ptr(),
                0,
            )
        };

        if addresses != ptr::null_mut() {
            unsafe {
                for i in 0.. {
                    if *addresses.add(i) == ptr::null_mut() {
                        // Since the string array is NULL-terminated so free the last element here.
                        libc::free(*addresses.add(i) as *mut _);
                        break;
                    }
                    len += 1;
                }
            };
        }
        StringArrayIter::new(addresses, len)
    }
}
