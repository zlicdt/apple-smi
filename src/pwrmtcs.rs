// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * pwrmtcs.rs
 * Fetch data by running powermetrics output and parse that.
*/

use std::process::Command;
use anyhow::Result;
#[derive(Debug, Clone)]
pub struct GpuMetrics {
    // MHz
    pub gpu_hw_freq: Option<u32>,
    // percentage points (e.g. 4.63 means 4.63%)
    pub gpu_hw_residency: Option<f64>,
    /* 
     * SW_Pn residency winner (e.g. 3 means SW_P3)
     * It is like a list, so I have no idea to choose who as the gpu_sw_state
     * Use maximum value like codes below
     */
    pub gpu_sw_state: Option<usize>,
    // mW
    pub gpu_pwr: Option<u32>,
}

pub fn run_pwrmtcs() -> Result<GpuMetrics> {
    let cmd = "powermetrics -s gpu_power -i 200 -n 1 | grep -E '^GPU (HW active frequency|HW active residency|SW state|Power):'";
    let output = Command::new("sh").args(["-c", cmd]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut gpu_hw_freq: Option<u32> = None;
    let mut gpu_hw_residency: Option<f64> = None;
    let mut max_sw_state: Option<(usize, f64)> = None; // track (idx, value)
    let mut gpu_pwr: Option<u32> = None;

    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("GPU HW active frequency:") {
            if let Some(freq_str) = rest.trim().split_whitespace().next() {
                gpu_hw_freq = Some(freq_str.parse()?);
            }
        } else if let Some(rest) = line.strip_prefix("GPU HW active residency:") {
            let percent_str = rest.split('%').next().unwrap_or("").trim();
            gpu_hw_residency = Some(percent_str.parse()?);
        } else if let Some(rest) = line.strip_prefix("GPU SW state:") {
            let mut parts = rest.split_whitespace();
            while let Some(token) = parts.next() {
                let label = token.trim_start_matches('(');
                if let Some(idx_str) = label.strip_prefix("SW_P") {
                    let _colon = parts.next();
                    if let Some(val_token) = parts.next() {
                        let val_clean = val_token.trim_end_matches(|c| c == '%' || c == ')');
                        let idx: usize = idx_str.parse()?;
                        let val: f64 = val_clean.parse()?;
                        if max_sw_state.map(|(_, prev)| val > prev).unwrap_or(true) {
                            max_sw_state = Some((idx, val));
                        }
                    }
                }
            }
        } else if let Some(rest) = line.strip_prefix("GPU Power:") {
            if let Some(pwr_str) = rest.trim().split_whitespace().next() {
                gpu_pwr = Some(pwr_str.parse()?);
            }
        }
    }

    Ok(GpuMetrics {
        gpu_hw_freq,
        gpu_hw_residency,
        gpu_sw_state: max_sw_state.map(|(idx, _)| idx),
        gpu_pwr,
    })
}