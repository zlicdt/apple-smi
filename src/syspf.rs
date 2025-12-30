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
    #[serde(default)]
    pub sppci_model: String,
    #[serde(default)]
    pub spdisplays_vendor: String,
    #[serde(default)]
    pub sppci_bus: String,
    #[serde(default)]
    pub sppci_cores: String,
    #[serde(default)]
    pub spdisplays_mtlgpufamilysupport: String,
}

impl GpuEntry {
    pub fn vendor_label(&self) -> &str {
        match self.spdisplays_vendor.strip_prefix("sppci_vendor_") {
            Some(rest) => rest,
            None => &self.spdisplays_vendor,
        }
    }

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

pub fn run_syspf() -> Result<String> {
    let out = Command::new("system_profiler")
        .args(["-json", "SPDisplaysDataType"])
        .output()
        .context("is this macOS?")?;

    anyhow::ensure!(
        out.status.success(),
        "system_profiler exited with status {}",
        out.status
    );

    let s = String::from_utf8(out.stdout)?;
    Ok(s)
}