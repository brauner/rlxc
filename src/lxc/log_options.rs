// SPDX-License-Identifier: LGPL-2.1+

use crate::util::ffi::ToCString;
use std::ffi::{CString, NulError, OsStr};
use std::ptr;

/// Type representing options to initialize a log for the container.
pub struct LogOptions {
    raw: lxc_sys::lxc_log,
    name: Option<CString>,
    path: Option<CString>,
    file: Option<CString>,
    level: Option<CString>,
    prefix: Option<CString>,
    quiet: bool,
}

impl LogOptions {
    pub fn new() -> Self {
        Self {
            raw: unsafe { std::mem::zeroed() },
            name: None,
            path: None,
            file: None,
            level: None,
            prefix: None,
            quiet: true,
        }
    }
}

impl Default for LogOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl LogOptions {
    pub fn set_log_name<T: AsRef<OsStr>>(
        mut self,
        v: T,
    ) -> Result<Self, NulError> {
        let cv = v.as_ref().to_c_string()?;

        self.name = Some(cv.into_owned());
        Ok(self)
    }

    pub fn set_log_path<T: AsRef<OsStr>>(
        mut self,
        v: T,
    ) -> Result<Self, NulError> {
        let cv = v.as_ref().to_c_string()?;

        self.path = Some(cv.into_owned());
        Ok(self)
    }

    pub fn set_log_file<T: AsRef<OsStr>>(
        mut self,
        v: T,
    ) -> Result<Self, NulError> {
        let cv = v.as_ref().to_c_string()?;

        self.file = Some(cv.into_owned());
        Ok(self)
    }

    pub fn set_log_level<T: AsRef<OsStr>>(
        mut self,
        v: T,
    ) -> Result<Self, NulError> {
        let cv = v.as_ref().to_c_string()?;

        self.level = Some(cv.into_owned());
        Ok(self)
    }

    pub fn set_log_prefix<T: AsRef<OsStr>>(
        mut self,
        v: T,
    ) -> Result<Self, NulError> {
        let cv = v.as_ref().to_c_string()?;

        self.prefix = Some(cv.into_owned());
        Ok(self)
    }

    pub fn set_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    fn finish(&mut self) {
        self.raw.name = self
            .name
            .as_ref()
            .map(|s| s.as_ptr() as *mut _)
            .unwrap_or(ptr::null_mut());
        self.raw.lxcpath = self
            .path
            .as_ref()
            .map(|s| s.as_ptr() as *mut _)
            .unwrap_or(ptr::null_mut());
        self.raw.file = self
            .file
            .as_ref()
            .map(|s| s.as_ptr() as *mut _)
            .unwrap_or(ptr::null_mut());
        self.raw.level = self
            .level
            .as_ref()
            .map(|s| s.as_ptr() as *mut _)
            .unwrap_or(ptr::null_mut());
        self.raw.prefix = self
            .prefix
            .as_ref()
            .map(|s| s.as_ptr() as *mut _)
            .unwrap_or(ptr::null_mut());
        self.raw.quiet = self.quiet;
    }

    pub fn raw(&mut self) -> &mut lxc_sys::lxc_log {
        self.finish();
        &mut self.raw
    }
}
