// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2025 zlicdt@ReSpringClipsNeko
 * render.rs
 * Render the output.
*/
use crate::syspf;
use anyhow::Result;

pub fn render() -> Result<()> {
    let out = syspf::run_syspf()?;
    let root: syspf::Root = serde_json::from_str(&out)?;

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