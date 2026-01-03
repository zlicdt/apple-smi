// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * utils.rs
 * Some magics.
*/
pub fn project_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}
