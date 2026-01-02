// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * ioreg.rs
 * Fetch data by running ioreg output and parse that.
*/
use std::process::Command;
use anyhow::Result;

pub struct VramInfo {
    pub alloc_vram: Option<u64>,
    pub inuse_vram: Option<u64>,
}

pub fn run_ioreg() -> Result<VramInfo> {
    let cmd = "ioreg -r -d 1 -w 0 -c IOAccelerator -l | grep memory | head -n 1";
    let output = Command::new("sh").args(["-c", cmd]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    fn extract(label: &str, text: &str) -> Option<u64> {
        let pat = format!("\"{}\"=", label);
        let start = text.find(&pat)? + pat.len();
        let digits: String = text[start..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        digits.parse().ok()
    }

    let alloc_vram = extract("Alloc system memory", &stdout);
    let inuse_vram = extract("In use system memory", &stdout);

    Ok(VramInfo {
        // Convert from bytes to MiB
        alloc_vram: alloc_vram.map(|v| v / (1024 * 1024)),
        inuse_vram: inuse_vram.map(|v| v / (1024 * 1024)),
    })
}