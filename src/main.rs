use std::process::exit;

use failure::*;

mod cli;
mod lxc;

fn cmd_start(args: &clap::ArgMatches) -> Result<(), Error> {
    let sname = args.value_of("name").unwrap();
    let spath = args.value_of("path").unwrap();

    let container = lxc::Lxc::new(sname, spath)?;

    if !container.may_control() {
        bail!("Insufficient permissions");
    }

    if container.is_running() {
        bail!("Container already running");
    }

    container.start(false)
}

fn cmd_stop(args: &clap::ArgMatches) -> Result<(), Error> {
    let sname = args.value_of("name").unwrap();
    let spath = args.value_of("path").unwrap();
    let force = args.is_present("force");
    let timeout = match args.value_of("timeout").unwrap_or("-1").parse::<i32>()
    {
        Ok(n) => n,
        Err(n) => bail!("Invalid timeout: {:?}", n),
    };

    let container = lxc::Lxc::new(sname, spath)?;

    if !container.may_control() {
        bail!("Insufficient permissions");
    }

    if !container.is_running() {
        bail!("Container not running");
    }

    if force {
        return container.stop();
    }

    return container.shutdown(timeout);
}

fn cmd_exec(args: &clap::ArgMatches) -> i32 {
    let sname = args.value_of("name").unwrap();
    let spath = args.value_of("path").unwrap();
    let vals: Vec<&str> = args.values_of("command").unwrap().collect();

    let container = match lxc::Lxc::new(sname, spath) {
        Ok(c) => c,
        Err(_) => return 1,
    };

    if !container.may_control() {
        eprintln!("Insufficient permissions");
        return 1;
    }

    if !container.is_running() {
        eprintln!("Container not running");
        return 1;
    }

    container.attach_run_wait(vals[0], vals)
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
    } else if let Some(exec) = matches.subcommand_matches("exec") {
        exit(cmd_exec(exec));
    } else {
        println!("{}", matches.usage())
    }
}
