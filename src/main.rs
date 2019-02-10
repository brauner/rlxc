use std::process::exit;
mod cli;
mod lxc;

fn main() {
    let matches = cli::build_cli().get_matches();

    if let Some(start) = matches.subcommand_matches("start") {
        let sname = start.value_of("name").unwrap();
        let spath = start.value_of("path").unwrap();

        let mut container = lxc::Lxc::new(sname, spath);

        if !container.may_control() {
            eprintln!("error: Insufficient permissions");
            exit(1);
        }

        if container.is_running() {
            eprintln!("error: Container already running");
            exit(1);
        }

        if let Err(err) = container.start(false) {
            eprintln!("error: {}", err);
            exit(1);
        }
    }
}
