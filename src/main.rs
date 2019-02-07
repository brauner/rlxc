extern crate clap;

use std::ffi::CString;
use std::ptr;

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
    let spath = matches
        .value_of("path")
        .unwrap_or("/home/brauner/.local/share/lxc");
    let cname = CString::new(sname).unwrap();
    let cpath = CString::new(spath).unwrap();
    let container =
        unsafe { lxc_sys::lxc_container_new(cname.as_ptr(), cpath.as_ptr()) };
    unsafe {
        (*container).daemonize = true;
        (*container).start.unwrap()(container, 0, ptr::null());
        (*container).is_running.unwrap()(container);
    }
}
