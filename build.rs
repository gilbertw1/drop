extern crate clap;

use std::env;
use std::fs;

use clap::Shell;

#[allow(dead_code)]
#[path = "src/cli.rs"]
mod cli;

fn main() {
  let outdir = match env::var_os("OUT_DIR") {
    None => return,
  Some(outdir) => outdir,
    };
    fs::create_dir_all(&outdir).unwrap();

    let mut app = cli::create_drop_cli_app();
    app.gen_completions("drop", Shell::Bash, &outdir);
    app.gen_completions("drop", Shell::Fish, &outdir);
    app.gen_completions("drop", Shell::Zsh, &outdir);
    app.gen_completions("drop", Shell::PowerShell, &outdir);
}
