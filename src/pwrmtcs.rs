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
    /// MHz
    pub gpu_hw_freq: Option<u32>,
    /// percentage points (e.g. 4.63 means 4.63%)
    pub gpu_hw_residency: Option<f64>,
    /// SW_Pn residency, index 0 unused; SW_P1 at [1]
    pub gpu_sw_state: Option<Vec<f64>>,
    /// mW
    pub gpu_pwr: Option<u32>,
}

pub fn run_pwrmtcs() -> Result<GpuMetrics> {
    let cmd = "powermetrics -s gpu_power -i 200 -n 1 | grep -E '^GPU (HW active frequency|HW active residency|SW state|Power):'";
    let output = Command::new("sh").args(["-c", cmd]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut gpu_hw_freq: Option<u32> = None;
    let mut gpu_hw_residency: Option<f64> = None;
    let mut gpu_sw_state: Vec<f64> = vec![0.0; 16]; // index 0 unused; SW_P1..SW_P15 at 1..15
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
                        if idx >= gpu_sw_state.len() {
                            gpu_sw_state.resize(idx + 1, 0.0);
                        }
                        gpu_sw_state[idx] = val_clean.parse()?;
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
        gpu_sw_state: Some(gpu_sw_state),
        gpu_pwr,
    })
}