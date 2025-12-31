// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2025 zlicdt@ReSpringClipsNeko
 * main.rs
 * The 'entry'.
*/
use anyhow::Result;
mod render;
mod syspf;
mod utils;

fn main() -> Result<()> {
    // TODO: Check root permissions, that we can get powermetrics data
    render::render()
}
