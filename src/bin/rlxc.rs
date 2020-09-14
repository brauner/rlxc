// SPDX-License-Identifier: LGPL-2.1+

use std::process::exit;

use anyhow::{bail, Error};

use rlxc::cli::rlxc as cli;
use rlxc::lxc::{self, Lxc};
#[macro_use]
extern crate prettytable;
use prettytable::Table;
use rayon::prelude::*;

fn cmd_start(args: &clap::ArgMatches) -> Result<(), Error> {
    let sname = args.value_of_os("name").unwrap();
    let spath = args
        .value_of_os("path")
        .unwrap_or_else(|| lxc::get_default_path().as_ref());
    if spath.is_empty() {
        bail!("Missing required argument: 'path' and no default path set");
    }

    let logfile = args.value_of_os("logfile").unwrap_or("none".as_ref());
    let loglevel = args.value_of_os("loglevel").unwrap_or("ERROR".as_ref());

    let vals: Vec<&str> = match args.values_of("command") {
        None => Vec::new(),
        Some(v) => v.collect(),
    };

    if !logfile.is_empty() {
        let mut options = lxc::LogOptions::new();
        options = options.set_log_name(sname)?;
        options = options.set_log_path(spath)?;
        options = options.set_log_file(logfile)?;
        options = options.set_log_level(loglevel)?;
        lxc::set_log(&mut options)?
    }

    let container = Lxc::new(sname, spath)?;

    if !container.may_control() {
        bail!("Insufficient permissions");
    }

    if container.is_running() {
        bail!("Container already running");
    }

    if args.is_present("terminal") {
        container.daemonize(false);
    }

    if !vals.is_empty() {
        return container.start(true, vals);
    }

    container.start(false, vals)
}

fn cmd_stop(args: &clap::ArgMatches) -> Result<(), Error> {
    let sname = args.value_of_os("name").unwrap_or("".as_ref());

    let spath = args
        .value_of_os("path")
        .unwrap_or_else(|| lxc::get_default_path().as_ref());
    if spath.is_empty() {
        bail!("Missing required argument: 'path' and no default path set");
    }

    let all = args.is_present("all");

    if !all && sname.is_empty() {
        bail!("Either a single container or all containers must be stopped");
    }

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

    let stop_function = |name| {
        let container = Lxc::new(name, spath)?;

        if !container.may_control() {
            bail!("Insufficient permissions");
        }

        if !container.is_running() {
            println!("Container {:?} not running", name);
            return Ok(());
        }

        if force {
            return container.stop();
        }

        container.shutdown(timeout)
    };

    if !all {
        return stop_function(sname);
    }

    let bulk: Vec<String> = lxc::list_all_containers(spath)?.collect();
    let errors: Vec<_> = bulk
        .par_iter()
        .map(|name| stop_function(name.as_ref()))
        .filter_map(Result::err)
        .collect();

    if !errors.is_empty() {
        bail!("Failed to stop some containers");
    }
    Ok(())
}

fn cmd_exec(args: &clap::ArgMatches) -> i32 {
    let sname = args.value_of_os("name").unwrap();
    let spath = args
        .value_of_os("path")
        .unwrap_or_else(|| lxc::get_default_path().as_ref());
    if spath.is_empty() {
        eprintln!("Missing required argument: 'path' and no default path set");
        return 1;
    }
    let vals: Vec<&str> = args.values_of("command").unwrap().collect();
    let env: Vec<&str> = args
        .values_of("env")
        .map_or_else(Vec::new, |matches| matches.collect());

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
    let spath = args
        .value_of_os("path")
        .unwrap_or_else(|| lxc::get_default_path().as_ref());
    if spath.is_empty() {
        bail!("Missing required argument: 'path' and no default path set");
    }

    let mut table = Table::new();
    table.add_row(row!["NAME", "STATE", "IPV4", "IPV6"]);
    for name in lxc::list_all_containers(spath)? {
        let container = Lxc::new(&name, spath)?;

        if !container.may_control() {
            continue;
        }

        let mut ipv4 = String::new();
        let mut ipv6 = String::new();
        if container.is_running() {
            let interfaces = container.get_interfaces();
            for iface in interfaces {
                // skip the loopback device
                if iface == "lo" {
                    continue;
                }

                for ipv4_addr in container.get_ipv4(&iface) {
                    ipv4.push_str(&format!("{} ({})\n", ipv4_addr, iface));
                }
                for ipv6_addr in container.get_ipv6(&iface) {
                    ipv6.push_str(&format!("{} ({})\n", ipv6_addr, iface));
                }
            }
        }

        table.add_row(row![&name, container.state(), ipv4, ipv6]);
    }
    table.printstd();
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

    match matches.subcommand() {
        ("start", Some(args)) => do_cmd(args, cmd_start),
        ("stop", Some(args)) => do_cmd(args, cmd_stop),
        ("list", Some(args)) => do_cmd(args, cmd_list),
        ("exec", Some(args)) => exit(cmd_exec(args)),
        _ => {
            println!("{}", matches.usage());
            exit(1);
        }
    }
}
