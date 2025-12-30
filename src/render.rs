// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2025 zlicdt@ReSpringClipsNeko
 * render.rs
 * Render the output.
*/
use crate::syspf;
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

pub fn render() -> Result<()> {
    let out = syspf::run_syspf()?;
    let root: syspf::Root = serde_json::from_str(&out)?;

    print_div_str(0);

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