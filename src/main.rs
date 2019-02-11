use std::process::exit;

use failure::*;

mod cli;
mod lxc;

fn cmd_start(args: &clap::ArgMatches) -> Result<(), Error> {
    let sname = args.value_of("name").unwrap();
    let spath = args.value_of("path").unwrap();

    let container = lxc::Lxc::new(sname, spath);

    if !container.may_control() {
        bail!("Insufficient permissions");
    }

    if !container.is_running() {
        bail!("Container not running");
    }

    container.start(false)
}

fn cmd_stop(args: &clap::ArgMatches) -> Result<(), Error> {
    let sname = args.value_of("name").unwrap();
    let spath = args.value_of("path").unwrap();
    let force = args.is_present("force");

    let container = lxc::Lxc::new(sname, spath);

    if !container.may_control() {
        bail!("Insufficient permissions");
    }

    if !container.is_running() {
        bail!("Container not running");
    }

    if !force {
        return container.shutdown(-1);
    }

    return container.stop();
}

fn main() {
    let matches = cli::build_cli().get_matches();

    if let Some(start) = matches.subcommand_matches("start") {
        if let Err(err) = cmd_start(start) {
            eprintln!("error: {}", err);
            exit(1);
        }
    } else if let Some(stop) = matches.subcommand_matches("stop") {
        if let Err(err) = cmd_stop(stop) {
            eprintln!("error: {}", err);
            exit(1);
        }
    } else if let Some(list) = matches.subcommand_matches("list") {
        let spath = list.value_of("path").unwrap();
        if let Err(err) = lxc::list_all_containers(spath) {
            eprintln!("error: {}", err);
            exit(1);
        }
    } else if matches.subcommand_matches("version").is_some() {
        let version = lxc::get_version();
        println!("driver_version: {}", version);
    } else {
        println!("{}", matches.usage())
    }
}
