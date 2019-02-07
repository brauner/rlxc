use clap::{App, Arg};

pub fn build_cli() -> App<'static, 'static> {
    App::new("lxc-run")
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
}
