// SPDX-License-Identifier: LGPL-2.1+

//! FFI utilities to help communicating with the C library.

use std::borrow::Cow;
use std::ffi::{CStr, CString, NulError};
use std::os::raw::c_char;
use std::os::unix::ffi::OsStrExt;

/// Helper to create a C string array (`char**`) variable with the ownership
/// still in rust code. The raw version of this will contain a trailing `NULL`
/// pointer.
#[derive(Debug, Default)]
pub struct CStringVec {
    owned: Vec<CString>,
    ffi: Vec<*const c_char>,
}

impl CStringVec {
    /// Create a new empty vector.
    pub fn new() -> Self {
        Self {
            owned: Vec::new(),
            ffi: Vec::new(),
        }
    }

    /// Update the inner `ffi` vector.
    fn update(&mut self) {
        self.ffi.truncate(0);
        self.ffi.reserve(self.owned.len() + 1);
        for cstr in self.owned.iter() {
            self.ffi.push(cstr.as_ptr());
        }
        self.ffi.push(std::ptr::null());
    }

    /// Get a reference to the ffi vector. We return a reference to the `Vec`
    /// type instead of returning a `*const *const c_char` to explicitly show
    /// that this borrows `self`!
    pub fn get_raw<'a>(&'a mut self) -> &'a Vec<*const c_char> {
        self.update();
        &self.ffi
    }

    //pub fn into_inner(self) -> Vec<CString> {
    //    self.owned
    //}
}

// Implement `Deref<Vec<CString>>` so we can use this type exactly as if
// it actually were just the inner `Vec<CString>`.
impl std::ops::Deref for CStringVec {
    type Target = Vec<CString>;

    fn deref(&self) -> &Self::Target {
        &self.owned
    }
}

impl std::ops::DerefMut for CStringVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.owned
    }
}

/// This iterates over a `char**`, consuming each contained string by returning
/// it as an owning CString. The pointer holding the list will also be freed in
/// `drop`.
#[derive(Debug)]
pub struct StringArrayIter {
    ptr: *mut *mut c_char,
    len: usize,
    at: usize,
}

impl StringArrayIter {
    /// Get the contents as a slice to work with more easily.
    fn as_slice(&self) -> &[*mut c_char] {
        unsafe {
            std::slice::from_raw_parts(self.ptr as *const *mut c_char, self.len)
        }
    }

    /// Create a new string array iterator.
    ///
    /// # Safety
    ///
    /// `ptr` must point to an allocated array of valid pointers to C strings.
    pub unsafe fn new(ptr: *mut *mut c_char, len: usize) -> Self {
        let mut this = Self { ptr, len, at: 0 };
        // Try to find any early NULLs.
        this.len = this
            .as_slice()
            .iter()
            .position(|p| p.is_null())
            .unwrap_or(this.len);
        this
    }
}

impl Iterator for StringArrayIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = *self.as_slice().get(self.at)?;
        self.at += 1;
        let cstr = unsafe { CStr::from_ptr(ptr) };
        Some(
            cstr.to_str()
                .expect("liblxc returned non-utf8 string")
                .to_string(),
        )
    }
}

impl Drop for StringArrayIter {
    fn drop(&mut self) {
        unsafe {
            for ptr in self.as_slice() {
                libc::free(*ptr as *mut libc::c_void);
            }
            libc::free(self.ptr as *mut _);
        }
    }
}

/// Helper trait allowing faster conversion from various string types to
/// `CStrings`.
///
/// Some types in rust don't have convenient conversion paths towards to
/// `CString` (due to rust methods having to be safe for multiple platforms).
/// `CString::new` takes a value which is `Into<Vec<u8>>`. When you have a
/// `Path` you can take it `AsRef<OsStr>` which can provide an `as_bytes()`
/// method, but only if the `std::os::unix::ffi::OsStrExt` trait is visible.
///
/// This trait connects all those missing paths for the most common types. Use
/// it as:
///
/// ```
// use rlxc::util::ffi::ToCString;
//
// fn foo<T: ?Sized + ToCString>(path: &T) -> Result<()> {
//     let cpath = path.to_c_string()?;
//     unsafe { c_function(cpath.as_ptr()) };
//     Ok(())
// }
// ```
pub trait ToCString {
    fn to_c_string(&self) -> Result<Cow<CStr>, NulError>;
}

impl ToCString for CStr {
    fn to_c_string(&self) -> Result<Cow<CStr>, NulError> {
        Ok(Cow::Borrowed(self))
    }
}

impl ToCString for CString {
    fn to_c_string(&self) -> Result<Cow<CStr>, NulError> {
        Ok(Cow::Borrowed(&self))
    }
}

impl ToCString for str {
    fn to_c_string(&self) -> Result<Cow<CStr>, NulError> {
        Ok(Cow::Owned(CString::new(self)?))
    }
}

impl ToCString for [u8] {
    fn to_c_string(&self) -> Result<Cow<CStr>, NulError> {
        Ok(Cow::Owned(CString::new(self)?))
    }
}

impl ToCString for String {
    fn to_c_string(&self) -> Result<Cow<CStr>, NulError> {
        Ok(Cow::Owned(CString::new(self.as_bytes())?))
    }
}

impl ToCString for std::ffi::OsStr {
    fn to_c_string(&self) -> Result<Cow<CStr>, NulError> {
        Ok(Cow::Owned(CString::new(self.as_bytes())?))
    }
}

impl ToCString for std::ffi::OsString {
    fn to_c_string(&self) -> Result<Cow<CStr>, NulError> {
        Ok(Cow::Owned(CString::new(self.as_bytes())?))
    }
}

impl ToCString for std::path::Path {
    fn to_c_string(&self) -> Result<Cow<CStr>, NulError> {
        AsRef::<std::ffi::OsStr>::as_ref(self).to_c_string()
    }
}

impl ToCString for std::path::PathBuf {
    fn to_c_string(&self) -> Result<Cow<CStr>, NulError> {
        AsRef::<std::ffi::OsStr>::as_ref(self).to_c_string()
    }
}
