// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * render.rs
 * Render the output.
 */
use crate::ioreg;
use crate::pwrmtcs;
use crate::smc;
use crate::syspf;
use crate::utils;
mod ui;
use anyhow::Result;

pub fn render() -> Result<()> {
    let (gpu_json, os_json) = syspf::run_syspf()?;
    let root: syspf::Root = serde_json::from_str(&gpu_json)?;
    let os_ver: syspf::SysProf = serde_json::from_str(&os_json)?;
    let pwrmtcs_outs: pwrmtcs::GpuMetrics = if utils::is_root() {
        pwrmtcs::run_pwrmtcs()?
    } else {
        pwrmtcs::GpuMetrics {
            gpu_hw_freq: None,
            gpu_hw_residency: None,
            gpu_sw_state: None,
            // gpu_pwr: None,
        }
    };
    let ioreg_outs = ioreg::run_ioreg()?;
    let smc_outs = smc::read_smc_snapshot()?;
    let os_label = os_ver
        .system
        .get(0)
        .map(|s| s.os_version_label())
        .unwrap_or("");
    let metal_ver = root.gpus.get(0).map(|g| g.metal_lable()).unwrap_or("");

    ui::print_div_str(0);
    ui::print_header_line(os_label, metal_ver);
    ui::print_div_str(1);
    ui::print_title();
    ui::print_div_str(2);
    for (i, g) in root.gpus.iter().enumerate() {
        ui::print_card(i, g, &pwrmtcs_outs, &ioreg_outs, &smc_outs);
    }
    ui::print_empty_line();
    ui::print_tprocess_header();
    ui::print_processes();
    Ok(())
}

pub fn list_gpus() -> Result<()> {
    let (gpu_json, _) = syspf::run_syspf()?;
    let root: syspf::Root = serde_json::from_str(&gpu_json)?;
    // Outs like GPU 0: Apple M4 [Built-in] (Metal 4)
    for (idx, gpu) in root.gpus.iter().enumerate() {
        println!(
            "GPU {}: {} [{}] (Metal {})",
            idx,
            gpu.name,
            gpu.bus_label(),
            gpu.metal_lable()
        );
    }

    Ok(())
}