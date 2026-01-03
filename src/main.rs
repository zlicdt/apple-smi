// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * main.rs
 * The 'entry'.
*/
use anyhow::Result;
use chrono::Local;
use clap::Command;
mod ioreg;
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
    let _matches = Command::new("apple-smi")
        .about("Apple Silicon System Management Interface")
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    // Local time
    println!("{}", Local::now().format("%a %b %e %T %Y"));
    render::render()
}
