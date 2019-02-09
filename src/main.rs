use std::ffi::CString;
use std::os::unix::ffi::OsStringExt;
use std::ptr;

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
    let slogfile = matches.value_of("logfile").unwrap_or("");
    let sloglevel = matches.value_of("loglevel").unwrap_or("ERROR");

    let cname = CString::new(sname).unwrap();
    let container =
        unsafe { lxc_sys::lxc_container_new(cname.as_ptr(), cpath.as_ptr()) };
    unsafe {
        if !slogfile.is_empty() {
            let clogfile = CString::new(slogfile).unwrap();
            let cloglevel = CString::new(sloglevel).unwrap();
            let cprefix = CString::new("lxc-run").unwrap();
            let mut log = lxc_sys::lxc_log {
                name: cname.as_ptr(),
                lxcpath: cpath.as_ptr(),
                file: clogfile.as_ptr(),
                level: cloglevel.as_ptr(),
                prefix: cprefix.as_ptr(),
                quiet: false,
            };

            let err = lxc_sys::lxc_log_init(&mut log);
            if err < 0 {
                lxc_sys::lxc_container_put(container);
                panic!("Failed to initialize log");
            }
        }
        (*container).daemonize = true;
        let err = (*container).start.unwrap()(container, 0, ptr::null());
        if !err {
            lxc_sys::lxc_container_put(container);
            panic!("Failed to start container {}", sname);
        }
    }
}
