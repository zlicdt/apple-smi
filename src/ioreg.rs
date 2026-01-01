// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * ioreg.rs
 * Fetch data by running ioreg output and parse that.
*/
use std::process::Command;
use anyhow::Result;