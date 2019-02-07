use std::env;

use clap::Shell;

include!("src/cli.rs");

fn main() {
    let outdir = match env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };

    let mut app = build_cli();
    app.gen_completions("lxc-run", Shell::Bash, outdir);
    //app.gen_completions("lxc-run", Shell::Zsh, outdir);
}
