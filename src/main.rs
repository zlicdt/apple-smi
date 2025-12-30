use anyhow::{Context, Ok, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize)]
struct Root {
    #[serde(rename = "SPDisplaysDataType", default)]
    gpus: Vec<GpuEntry>,
}

#[derive(Deserialize)]
struct GpuEntry {
    #[serde(rename = "_name")]
    name: Option<String>,
    sppci_model: Option<String>,
    spdisplays_vendor: Option<String>,
    sppci_bus: Option<String>,
    sppci_cores: Option<String>,
    spdisplays_mtlgpufamilysupport: Option<String>,
}

impl GpuEntry {
    fn vendor_label(&self) -> Option<&str> {
        self.spdisplays_vendor
            .as_deref()
            .map(|s| s.strip_prefix("sppci_vendor_").unwrap_or(s))
    }

    fn bus_label(&self) -> Option<&str> {
        self.sppci_bus.as_deref().map(|s| match s {
            "spdisplays_builtin" => "Built-in",
            other => other,
        })
    }

    fn metal_lable(&self) -> Option<&str> {
        self.spdisplays_mtlgpufamilysupport
            .as_deref()
            .map(|s| s.strip_prefix("spdisplays_metal").unwrap_or(s))
    }
}

fn run_system_profiler() -> Result<String> {
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
fn main() -> Result<()> {
    let out = run_system_profiler()?;
    let root: Root = serde_json::from_str(&out)?;
    for (i, g) in root.gpus.iter().enumerate() {
        // Option<String> -> Option<&str> -> &str
        let name: &str = g
            .name
            .as_deref()
            .or(g.sppci_model.as_deref())
            .unwrap_or("Unknown GPU");

        let cores: &str = g.sppci_cores.as_deref().unwrap_or("N/A");
        let bus: &str = g.bus_label().unwrap_or("N/A");
        let vendor: &str = g.vendor_label().unwrap_or("N/A");
        let metal: &str = g.metal_lable().unwrap_or("N/A");
        // TODO: apple-smi format
        println!("GPU {i}: name={name}, cores={cores}, bus={bus}, vendor={vendor}, metal={metal}");
    }
    Ok(())
}
