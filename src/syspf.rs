// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2025 zlicdt@ReSpringClipsNeko
 * syspf.rs
 * Fetch data by running system_profiler output JSON and parse that.
*/
use serde::Deserialize;
use std::process::Command;
use anyhow::{Context, Ok, Result};

#[derive(Deserialize)]
pub struct Root {
    #[serde(rename = "SPDisplaysDataType", default)]
    pub gpus: Vec<GpuEntry>,
}

#[derive(Deserialize)]
pub struct GpuEntry {
    // I think _name and sppci_model are equivalent, but keep both for safety
    #[serde(rename = "_name", default)]
    pub name: String,
    // #[serde(default)]
    // pub sppci_model: String,
    #[serde(default)]
    pub sppci_bus: String,
    #[serde(default)]
    pub sppci_cores: String,
    #[serde(default)]
    pub spdisplays_mtlgpufamilysupport: String,
}

impl GpuEntry {
    pub fn bus_label(&self) -> &str {
        match self.sppci_bus.as_str() {
            "spdisplays_builtin" => "Built-in",
            other => other,
        }
    }

    pub fn metal_lable(&self) -> &str {
        match self
            .spdisplays_mtlgpufamilysupport
            .strip_prefix("spdisplays_metal")
        {
            Some(rest) => rest,
            None => &self.spdisplays_mtlgpufamilysupport,
        }
    }
}

#[derive(Deserialize)]
pub struct SysProf {
    #[serde(rename = "SPSoftwareDataType", default)]
    pub system: Vec<SysVersion>,
}
#[derive(Deserialize)]
pub struct SysVersion {
    #[serde(default)]
    pub os_version: String,
}

impl SysVersion {
    pub fn os_version_label(&self) -> &str {
        match self.os_version.strip_prefix("macOS ") {
            Some(rest) => rest,
            None => &self.os_version,
        }
    }
}

pub fn run_syspf() -> Result<(String, String)> {
    let gpu_out = Command::new("system_profiler")
        .args(["-json", "SPDisplaysDataType"])
        .output()
        .context("is this macOS?")?;

    let os_out = Command::new("system_profiler")
        .args(["-json", "SPSoftwareDataType"])
        .output()
        .context("is this macOS?")?;

    anyhow::ensure!(
        gpu_out.status.success(),
        "system_profiler exited with status {}",
        gpu_out.status
    );
    anyhow::ensure!(
        os_out.status.success(),
        "system_profiler exited with status {}",
        os_out.status
    );

    let s_gpu = String::from_utf8(gpu_out.stdout)?;
    let s_os = String::from_utf8(os_out.stdout)?;
    Ok((s_gpu, s_os))
}