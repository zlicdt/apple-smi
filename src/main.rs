// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * main.rs
 * The 'entry'.
*/
use anyhow::Result;
use chrono::Local;
mod render;
mod syspf;
mod utils;
mod pwrmtcs;
mod mtlapi;
mod ioreg;

fn main() -> Result<()> {
    // Local time
    println!("{}", Local::now().format("%a %b %e %T %Y"));
    render::render()
}
