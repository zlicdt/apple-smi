// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * main.rs
 * The 'entry'.
*/
use anyhow::Result;
mod render;
mod syspf;
mod utils;
mod pwrmtcs;
mod mtlapi;
mod ioreg;

fn main() -> Result<()> {
    render::render()
}
