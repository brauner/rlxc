// SPDX-License-Identifier: LGPL-2.1+

//! Rust wrapper for `struct lxc_container`. Implements methods to control
//! containers.

use anyhow::{bail, Error};
use std::ffi::{CStr, CString, OsStr};
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::ptr;
use std::time::Duration;

use crate::util::ffi::{StringArrayIter, ToCString};

mod attach_options;
mod log_options;
pub use attach_options::AttachOptions;
pub use log_options::LogOptions;

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
    Ok(unsafe { StringArrayIter::new(names, nr as usize) })
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
    match get_global_config_item("lxc.lxcpath") {
        Ok(s) => s,
        Err(_) => "",
    }
}

pub fn set_log(options: &mut LogOptions) -> Result<(), Error> {
    let ret = unsafe { lxc_sys::lxc_log_init(options.raw()) };

    if ret < 0 {
        bail!("failed to initialize log");
    }

    Ok(())
}

impl Lxc {
    /// Create a new container handler for the container of the given `name`
    /// residing under the provided `path`.
    pub fn new<S: AsRef<OsStr>, T: AsRef<OsStr>>(
        name: S,
        path: T,
    ) -> Result<Lxc, Error> {
        let cname = name.as_ref().to_c_string()?;
        let cpath = path.as_ref().to_c_string()?;
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
    pub fn start(&self, stub: bool, argv: Vec<&str>) -> Result<(), Error> {
        let useinit = if stub { 1 } else { 0 };
        let cargv: Vec<_> =
            argv.iter().map(|arg| CString::new(*arg).unwrap()).collect();
        let mut args: Vec<_> = cargv.iter().map(|arg| arg.as_ptr()).collect();
        if args.is_empty() {
            args.push(std::ptr::null());
        }

        let started = unsafe {
            if args.is_empty() {
                // LXC doesn't alter char *const argv[] so the cast is safe.
                (*self.handle).start.unwrap()(
                    self.handle,
                    useinit,
                    args.as_ptr() as *const *mut i8,
                )
            } else {
                (*self.handle).start.unwrap()(self.handle, useinit, ptr::null())
            }
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

        if !names.is_null() {
            unsafe {
                for i in 0.. {
                    if (*names.add(i)).is_null() {
                        break;
                    }
                    len += 1;
                }
            };
        }
        unsafe { StringArrayIter::new(names, len) }
    }

    /// Get ip addresses of an interface.
    pub fn get_ipv4(&self, interface: &str) -> StringArrayIter {
        let iface = CString::new(interface).unwrap();

        let mut len = 0;
        let addresses: *mut *mut c_char = unsafe {
            (*self.handle).get_ips.unwrap()(
                self.handle,
                iface.as_ptr(),
                c_str!("inet").as_ptr(),
                0,
            )
        };

        if !addresses.is_null() {
            unsafe {
                for i in 0.. {
                    if (*addresses.add(i)).is_null() {
                        // Since the string array is NULL-terminated so free the last element here.
                        libc::free(*addresses.add(i) as *mut _);
                        break;
                    }
                    len += 1;
                }
            };
        }
        unsafe { StringArrayIter::new(addresses, len) }
    }

    /// Get ip addresses of an interface.
    pub fn get_ipv6(&self, interface: &str) -> StringArrayIter {
        let iface = CString::new(interface).unwrap();

        let mut len = 0;
        let addresses: *mut *mut c_char = unsafe {
            (*self.handle).get_ips.unwrap()(
                self.handle,
                iface.as_ptr(),
                c_str!("inet6").as_ptr(),
                0,
            )
        };

        if !addresses.is_null() {
            unsafe {
                for i in 0.. {
                    if (*addresses.add(i)).is_null() {
                        // Since the string array is NULL-terminated so free the last element here.
                        libc::free(*addresses.add(i) as *mut _);
                        break;
                    }
                    len += 1;
                }
            };
        }
        unsafe { StringArrayIter::new(addresses, len) }
    }

    pub fn daemonize(&self, daemonize: bool) {
        unsafe {
            (*self.handle).want_daemonize.unwrap()(self.handle, daemonize)
        };
    }

    pub fn terminal(&self) -> Result<(), Error> {
        let ret = unsafe {
            (*self.handle).console.unwrap()(self.handle, 0, 0, 1, 2, 1)
        };

        if ret < 0 {
            bail!("failed to attach to terminal");
        }
        Ok(())
    }
}
