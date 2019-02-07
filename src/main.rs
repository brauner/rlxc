use std::ffi::CString;
use std::ptr;
use std::os::unix::ffi::OsStringExt;

use clap::{App, Arg};

fn main() {
    let matches = App::new("lxc-run")
        .version("0.1")
        .author(clap::crate_authors!("\n"))
        .about("Run LXC containers")
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .help("Name of the container")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .help("Path of the container")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

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
        (*container).start.unwrap()(container, 0, ptr::null());
        (*container).is_running.unwrap()(container);
    }
}
