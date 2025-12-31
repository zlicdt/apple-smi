// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2025 zlicdt@ReSpringClipsNeko
 * render.rs
 * Render the output.
*/
use crate::{syspf, utils};
use anyhow::Result;

// Whoh a geek print function, I'm genius!
fn print_div_str(dtype: i32) {
    let (segments, start, sep, end, fill): (&[usize], char, char, char, char) = match dtype {
        0 => (&[89], '+', '+', '+', '-'),
        1 => (&[41, 24, 22], '+', '+', '+', '-'),
        // "|===+===+===|" style divider
        2 => (&[41, 24, 22], '|', '+', '|', '='),
        _ => (&[], '+', '+', '+', '-'),
    };

    if segments.is_empty() {
        return;
    }

    // Build divider like "+---+---+" or "|===+===|" based on config.
    let line: String = std::iter::once(start)
        .chain(
            segments.iter().enumerate().flat_map(|(idx, n)| {
                let fill_iter = std::iter::repeat(fill).take(*n);
                let tail = if idx + 1 == segments.len() { end } else { sep };
                fill_iter.chain(std::iter::once(tail))
            }),
        )
        .collect();

    println!("{}", line);
}

fn print_header_line(os_version: &str, metal_version: &str) {
    // Columns must align with type 1 divider: segments 41, 24, 22.
    const SEGMENTS: [usize; 3] = [36, 30, 23];
    let col1: String = format!(" Apple-SMI {}", utils::project_version()); // leading space per requirement
    let col2 = format!("macOS Version: {}", os_version);
    let col3 = format!("Metal Version: {}", metal_version);

    fn pad(s: &str, width: usize) -> String {
        if s.len() >= width {
            s[..width].to_string()
        } else {
            let mut out = String::with_capacity(width);
            out.push_str(s);
            out.extend(std::iter::repeat(' ').take(width - s.len()));
            out
        }
    }

    let mut line = String::from("|");
    line.push_str(&pad(&col1, SEGMENTS[0]));
    line.push_str(&pad(&col2, SEGMENTS[1]));
    line.push_str(&pad(&col3, SEGMENTS[2]));
    line.push('|');

    println!("{}", line);
}

pub fn render() -> Result<()> {
    let (gpu_json, os_json) = syspf::run_syspf()?;
    let root: syspf::Root = serde_json::from_str(&gpu_json)?;
    let os_ver: syspf::SysProf = serde_json::from_str(&os_json)?;
    print_div_str(0);
    let os_label = os_ver
        .system
        .get(0)
        .map(|s| s.os_version_label())
        .unwrap_or("");
    let metal_ver = root
        .gpus
        .get(0)
        .map(|g| g.metal_lable())
        .unwrap_or("");

    print_header_line(os_label, metal_ver);
    print_div_str(1);

    for (i, g) in root.gpus.iter().enumerate() {
        let name: &str = g.name.as_str();
        let cores: &str = g.sppci_cores.as_str();
        let bus: &str = g.bus_label();
        let vendor: &str = g.vendor_label();
        let metal: &str = g.metal_lable();

        // TODO: apple-smi format
        println!(
            "GPU {i}: name={name}, cores={cores}, bus={bus}, vendor={vendor}, metal={metal}"
        );
    }

    Ok(())
}