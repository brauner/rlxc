use std::process::exit;

use failure::*;

use rlxc::cli::rlxc as cli;
use rlxc::lxc::{self, Lxc};

fn cmd_start(args: &clap::ArgMatches) -> Result<(), Error> {
    let sname = args.value_of("name").unwrap();
    let spath = args.value_of("path").unwrap();

    let container = Lxc::new(sname, spath)?;

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
    let timeout = match args.value_of("timeout") {
        None => None,
        Some(value) => match value.parse::<i32>() {
            Ok(-1) => None,
            Ok(n) => {
                if n < 0 {
                    bail!("Invalid timeout (must be -1, 0 or positive)");
                }
                Some(std::time::Duration::from_secs(n as u64))
            }
            Err(e) => bail!("Invalid timeout: {:?}", e),
        },
    };

    let container = Lxc::new(sname, spath)?;

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
    let mut env: Vec<&str> = Vec::new();
    if args.is_present("env") {
        env = args.values_of("env").unwrap().collect();
    }

    let container = match Lxc::new(sname, spath) {
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

    let mut options = lxc::AttachOptions::new();
    for e in env {
        let res: Vec<_> = e.splitn(2, '=').collect();
        if res.len() != 2 {
            eprintln!("Invalid environment variable");
            return 1;
        }
        options = match options.set_env_var(res[0], res[1]) {
            Ok(opt) => opt,
            Err(_) => {
                eprintln!("Failed to set environment variable");
                return 1;
            }
        }
    }
    container.attach_run_wait(&mut options, vals[0], vals)
}

fn cmd_list(args: &clap::ArgMatches) -> Result<(), Error> {
    for name in lxc::list_all_containers(args.value_of("path").unwrap())? {
        match name.to_str() {
            Ok(name) => println!("{}", name),
            Err(_) => println!("non-utf8 container name: {:?}", name),
        }
    }
    Ok(())
}

fn do_cmd(
    args: &clap::ArgMatches,
    func: fn(args: &clap::ArgMatches) -> Result<(), Error>,
) {
    if let Err(err) = func(args) {
        eprintln!("error: {}", err);
        exit(1);
    }
}

fn main() {
    let matches = cli::build_cli().get_matches();

    if matches.subcommand_matches("version").is_some() {
        let version = lxc::get_version();
        println!("driver_version: {}", version);
        return;
    }

    // All other commands require --path!
    if matches.value_of("path").is_none() {
        eprintln!("Missin grequired argument: 'path'");
        eprintln!("{}", matches.usage());
        exit(1);
    }

    if let Some(args) = matches.subcommand_matches("start") {
        do_cmd(args, cmd_start);
    } else if let Some(args) = matches.subcommand_matches("stop") {
        do_cmd(args, cmd_stop);
    } else if let Some(args) = matches.subcommand_matches("list") {
        do_cmd(args, cmd_list);
    } else if let Some(exec) = matches.subcommand_matches("exec") {
        exit(cmd_exec(exec));
    } else {
        println!("{}", matches.usage())
    }
}
