use std::ffi::CString;
use std::ptr;
use std::os::unix::ffi::OsStringExt;

mod cli;

fn main() {
    let matches = cli::build_cli().get_matches();

    let sname = matches.value_of("name").unwrap();
    let cpath = matches
        .value_of("path")
        .map_or_else(
            || {
                CString::new(
                    xdg::BaseDirectories::with_prefix("lxc")
                        .unwrap()
                        .get_data_home()
                        .into_os_string()
                        .into_vec(),
                )
            },
            |x| CString::new(x),
        )
        .unwrap();
    let cname = CString::new(sname).unwrap();
    let container =
        unsafe { lxc_sys::lxc_container_new(cname.as_ptr(), cpath.as_ptr()) };
    unsafe {
        (*container).daemonize = true;
        let err = (*container).start.unwrap()(container, 0, ptr::null());
        if !err {
            lxc_sys::lxc_container_put(container);
            panic!("Failed to start container {}", sname);
        }
    }
}
