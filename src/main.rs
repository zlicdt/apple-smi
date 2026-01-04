// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * main.rs
 * The 'entry'.
 */
use anyhow::Result;
use chrono::Local;
use clap::{Arg, ArgAction, Command};
mod ioreg;
mod ioreport;
mod mtlapi;
mod pwrmtcs;
mod render;
mod smc;
mod syspf;
mod utils;
fn main() -> Result<()> {
    /*
     * For argument, display help & version
     * clap will auto-provide -h/--help and -V/--version
     */
    let matches = Command::new("apple-smi")
        .about("Apple Silicon System Management Interface")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("list-gpus")
                .short('L')
                .long("list-gpus")
                .help("Display a list of GPUs connected to the system.")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("list-gpus") {
        render::list_gpus()?;
        return Ok(());
    }

    // Local time
    println!("{}", Local::now().format("%a %b %e %T %Y"));
    render::render()
}
