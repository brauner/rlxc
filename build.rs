use std::env;

use clap::Shell;

include!("src/cli/rlxc.rs");

fn main() {
    let outdir = match env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };

    let mut app = build_cli();
    app.gen_completions("rlxc", Shell::Bash, outdir);
    //app.gen_completions("rlxc", Shell::Zsh, outdir);
}
