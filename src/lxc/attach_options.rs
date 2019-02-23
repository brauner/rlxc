#![allow(dead_code)]

use std::ffi::{CString, NulError};
use std::os::raw::{c_char, c_int, c_long};
use std::os::unix::io::AsRawFd;
use std::ptr;

use crate::util::ffi::CStringVec;

/// Type representing options for how to attach to a container.
pub struct AttachOptions<'t, 'u, 'v, 'w> {
    raw: lxc_sys::lxc_attach_options_t,
    extra_env_vars: CStringVec,
    extra_keep_env: CStringVec,
    initial_cwd: Option<CString>,
    stdin: Option<&'t dyn AsRawFd>,
    stdout: Option<&'u dyn AsRawFd>,
    stderr: Option<&'v dyn AsRawFd>,
    log_file: Option<&'w dyn AsRawFd>,
}

impl AttachOptions<'static, 'static, 'static, 'static> {
    pub fn new() -> Self {
        Self {
            raw: unsafe { std::mem::zeroed() },
            extra_env_vars: CStringVec::new(),
            extra_keep_env: CStringVec::new(),
            initial_cwd: None,
            stdin: None,
            stdout: None,
            stderr: None,
            log_file: None,
        }
        .set_default()
    }
}

impl<'t, 'u, 'v, 'w> AttachOptions<'t, 'u, 'v, 'w> {
    #[inline(always)]
    fn set_default(mut self) -> Self {
        self.raw.attach_flags = lxc_sys::LXC_ATTACH_DEFAULT as c_int;
        self.raw.namespaces = -1;
        self.raw.personality = -1;
        self.raw.uid = !0;
        self.raw.gid = !0;
        self.raw.env_policy =
            lxc_sys::lxc_attach_env_policy_t_LXC_ATTACH_KEEP_ENV;
        self
    }

    pub fn attach_flag(mut self, flag: c_int, on: bool) -> Self {
        if on {
            self.raw.attach_flags |= flag;
        } else {
            self.raw.attach_flags &= !flag;
        }
        self
    }

    pub fn move_to_cgroup(self, on: bool) -> Self {
        self.attach_flag(lxc_sys::LXC_ATTACH_MOVE_TO_CGROUP as _, on)
    }

    pub fn drop_capabilities(self, on: bool) -> Self {
        self.attach_flag(lxc_sys::LXC_ATTACH_DROP_CAPABILITIES as _, on)
    }

    pub fn set_personality(self, on: bool) -> Self {
        self.attach_flag(lxc_sys::LXC_ATTACH_SET_PERSONALITY as _, on)
    }

    pub fn lsm_exec(self, on: bool) -> Self {
        self.attach_flag(lxc_sys::LXC_ATTACH_LSM_EXEC as _, on)
    }

    pub fn remount_proc_sys(self, on: bool) -> Self {
        self.attach_flag(lxc_sys::LXC_ATTACH_REMOUNT_PROC_SYS as _, on)
    }

    pub fn lsm_now(self, on: bool) -> Self {
        self.attach_flag(lxc_sys::LXC_ATTACH_LSM_NOW as _, on)
    }

    pub fn no_new_privs(self, on: bool) -> Self {
        self.attach_flag(lxc_sys::LXC_ATTACH_NO_NEW_PRIVS as _, on)
    }

    pub fn terminal(self, on: bool) -> Self {
        self.attach_flag(lxc_sys::LXC_ATTACH_TERMINAL as _, on)
    }

    pub fn namespaces(mut self, v: c_int) -> Self {
        self.raw.namespaces = v;
        self
    }

    /// Pass `None` to autodetect (which is the default).
    pub fn personality(mut self, v: Option<c_long>) -> Self {
        self.raw.personality = v.unwrap_or(!0);
        self
    }

    pub fn set_initial_cwd<T>(mut self, v: T) -> Result<Self, NulError>
    where
        T: Into<Vec<u8>>,
    {
        self.initial_cwd = Some(CString::new(v)?);
        Ok(self)
    }

    pub fn unset_initial_cwd(mut self) -> Self {
        self.initial_cwd = None;
        self
    }

    /// Pass `None` for the default behavior which is using the init-uid for
    /// userns containers, or 0 if auto detection fails.
    pub fn uid(mut self, v: Option<libc::uid_t>) -> Self {
        self.raw.uid = v.unwrap_or(!0);
        self
    }

    /// Pass `None` for the default behavior which is using the init-gid for
    /// userns containers, or 0 if auto detection fails.
    pub fn gid(mut self, v: Option<libc::gid_t>) -> Self {
        self.raw.gid = v.unwrap_or(!0);
        self
    }

    pub fn keep_env(mut self) -> Self {
        self.raw.env_policy =
            lxc_sys::lxc_attach_env_policy_t_LXC_ATTACH_KEEP_ENV;
        self
    }

    pub fn clear_env(mut self) -> Self {
        self.raw.env_policy =
            lxc_sys::lxc_attach_env_policy_t_LXC_ATTACH_CLEAR_ENV;
        self
    }

    pub fn set_keep_env(self, on: bool) -> Self {
        if on {
            self.keep_env()
        } else {
            self.clear_env()
        }
    }

    pub fn stdin<'a, T: AsRawFd>(
        self,
        file: &'a T,
    ) -> AttachOptions<'a, 'u, 'v, 'w> {
        AttachOptions {
            stdin: Some(file),
            ..unsafe { std::mem::transmute(self) }
        }
    }

    pub fn stdout<'a, T: AsRawFd>(
        self,
        file: &'a T,
    ) -> AttachOptions<'t, 'a, 'v, 'w> {
        AttachOptions {
            stdout: Some(file),
            ..unsafe { std::mem::transmute(self) }
        }
    }

    pub fn stderr<'a, T: AsRawFd>(
        self,
        file: &'a T,
    ) -> AttachOptions<'t, 'u, 'a, 'w> {
        AttachOptions {
            stderr: Some(file),
            ..unsafe { std::mem::transmute(self) }
        }
    }

    pub fn log_file<'a, T: AsRawFd>(
        self,
        file: &'a T,
    ) -> AttachOptions<'t, 'u, 'v, 'a> {
        AttachOptions {
            log_file: Some(file),
            ..unsafe { std::mem::transmute(self) }
        }
    }

    pub fn set_env_var(
        mut self,
        name: &str,
        value: &str,
    ) -> Result<Self, NulError> {
        self.extra_env_vars
            .push(CString::new(format!("{}={}", name, value))?);
        Ok(self)
    }

    pub fn keep_env_var(mut self, name: &str) -> Result<Self, NulError> {
        self.extra_keep_env.push(CString::new(name)?);
        Ok(self)
    }

    fn finish(&mut self) {
        self.raw.extra_env_vars =
            self.extra_env_vars.get_raw().as_ptr() as *mut *mut c_char;
        self.raw.extra_keep_env =
            self.extra_keep_env.get_raw().as_ptr() as *mut *mut c_char;
        self.raw.stdin_fd = self.stdin.map(|f| f.as_raw_fd()).unwrap_or(0);
        self.raw.stdout_fd = self.stdout.map(|f| f.as_raw_fd()).unwrap_or(1);
        self.raw.stderr_fd = self.stderr.map(|f| f.as_raw_fd()).unwrap_or(2);
        self.raw.log_fd =
            self.log_file.map(|f| f.as_raw_fd()).unwrap_or(-libc::EBADF);
        self.raw.initial_cwd = self
            .initial_cwd
            .as_ref()
            .map(|s| s.as_ptr() as *mut _)
            .unwrap_or(ptr::null_mut());
    }

    pub fn raw(&mut self) -> &mut lxc_sys::lxc_attach_options_t {
        self.finish();
        &mut self.raw
    }
}
