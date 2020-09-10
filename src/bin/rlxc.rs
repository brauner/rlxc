// SPDX-License-Identifier: LGPL-2.1+

use std::process::exit;

use failure::*;

use rlxc::cli::rlxc as cli;
use rlxc::lxc::{self, Lxc};
#[macro_use]
extern crate prettytable;
use prettytable::Table;

fn cmd_start(args: &clap::ArgMatches) -> Result<(), Error> {
    let sname = args.value_of("name").unwrap();
    let spath = args.value_of("path").unwrap_or(lxc::get_default_path());
    if spath.is_empty() {
        bail!("Missing required argument: 'path' and no default path set");
    }

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
    let mut sname = "";
    if args.is_present("name") {
        sname = args.value_of("name").unwrap();
    }
    let spath = args.value_of("path").unwrap_or(lxc::get_default_path());
    if spath.is_empty() {
        bail!("Missing required argument: 'path' and no default path set");
    }

    let all = args.is_present("all");

    if !all && sname.len() == 0 {
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

    if all {
        for name in lxc::list_all_containers(spath)? {
            let container = Lxc::new(&name, spath)?;

            if !container.may_control() {
                bail!("Insufficient permissions");
            }

            if !container.is_running() {
                println!("Container {:?} not running", name);
            }

            if force {
                if let Err(err) = container.stop() {
                    eprintln!("error: {}", err);
                }
            } else {
                if let Err(err) = container.shutdown(timeout) {
                    eprintln!("error: {}", err);
                }
            }
        }

        return Ok(());
    }

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
    let spath = args.value_of("path").unwrap_or(lxc::get_default_path());
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
    let spath = args.value_of("path").unwrap_or(lxc::get_default_path());
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
        let interfaces = container.get_interfaces();
        for iface in interfaces {
            // skip the loopback device
            if iface == "lo" {
                continue;
            }

            let ipv4_addresses = container.get_ipv4(&iface);
            for address in ipv4_addresses {
                ipv4.push_str(&address);
                ipv4.push_str(" (");
                ipv4.push_str(&iface);
                ipv4.push_str(")");
                ipv4.push('\n');
                // If we have multiple addresses don't bother for now.
                break;
            }

            let ipv6_addresses = container.get_ipv6(&iface);
            for address in ipv6_addresses {
                ipv6.push_str(&address);
                ipv6.push_str(" (");
                ipv6.push_str(&iface);
                ipv6.push_str(")");
                ipv6.push('\n');
                // If we have multiple addresses don't bother for now.
                break;
            }
        }

        table.add_row(row![name, container.state(), ipv4, ipv6]);
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
