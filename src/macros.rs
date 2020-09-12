macro_rules! c_str {
    ($data:expr) => {{
        #![allow(unused_unsafe)]
        let bytes = concat!($data, "\0");
        unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(bytes.as_bytes())
        }
    }};
}
