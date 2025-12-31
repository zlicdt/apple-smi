// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2025 zlicdt@ReSpringClipsNeko
 * render.rs
 * Render the output.
*/
use crate::syspf;
mod ui;
use anyhow::Result;

pub fn render() -> Result<()> {
    let (gpu_json, os_json) = syspf::run_syspf()?;
    let root: syspf::Root = serde_json::from_str(&gpu_json)?;
    let os_ver: syspf::SysProf = serde_json::from_str(&os_json)?;
    
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

    ui::print_div_str(0);
    ui::print_header_line(os_label, metal_ver);
    ui::print_div_str(1);
    ui::print_title();
    ui::print_div_str(2);
    for (i, g) in root.gpus.iter().enumerate() {
        let name: &str = g.name.as_str();
        let cores: &str = g.sppci_cores.as_str();
        let bus: &str = g.bus_label();
        let metal: &str = g.metal_lable();

        // TODO: apple-smi format
        println!(
            "GPU {i}: name={name}, cores={cores}, bus={bus}, metal={metal}"
        );
    }

    Ok(())
}