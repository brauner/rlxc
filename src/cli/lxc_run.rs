use clap::{App, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    App::new("rlxc")
        .version("0.1")
        .author(clap::crate_authors!("\n"))
        .about("Run LXC containers")
        .subcommand(
            SubCommand::with_name("exec")
                .about("Execute commands in a container")
                .arg(
                    Arg::with_name("name")
                        .index(1)
                        .help("Name of the container")
                        .required(true),
                )
                .arg(
                    Arg::with_name("path")
                        .short("p")
                        .long("path")
                        .help("Path of the container")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("command")
                        .index(2)
                        .help("Command to execute")
                        .takes_value(true)
                        .required(true)
                        .multiple(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("start")
                .about("Run LXC containers")
                .arg(
                    Arg::with_name("name")
                        .index(1)
                        .help("Name of the container")
                        .required(true),
                )
                .arg(
                    Arg::with_name("path")
                        .short("p")
                        .long("path")
                        .help("Path of the container")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("logfile")
                        .long("logfile")
                        .help("Logfile for the container")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("loglevel")
                        .long("loglevel")
                        .help("Loglevel for the container")
                        .takes_value(true)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("stop")
                .about("Stop LXC containers")
                .arg(
                    Arg::with_name("name")
                        .index(1)
                        .help("Name of the container")
                        .required(true),
                )
                .arg(
                    Arg::with_name("path")
                        .short("p")
                        .long("path")
                        .help("Path of the container")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("force")
                        .short("f")
                        .long("force")
                        .help("SIGKILL the container")
                        .takes_value(false)
                        .required(false)
                        .conflicts_with("timeout"),
                )
                .arg(
                    Arg::with_name("timeout")
                        .short("t")
                        .long("timeout")
                        .help("timeout to wait for the container to stop")
                        .takes_value(true)
                        .required(false)
                        .conflicts_with("force"),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List LXC containers")
                .arg(
                    Arg::with_name("path")
                        .index(1)
                        .help("Path of the container")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("version")
                .about("Show runtime and client version")
                .arg(
                    Arg::with_name("version")
                        .index(1)
                        .help("Show runtime and client version")
                        .takes_value(false)
                        .required(false),
                ),
        )
}
