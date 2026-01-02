// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * smc.rs
 * Read SMC data via IOKit FFI.
*/
use libc::{c_char, c_void};
use std::collections::{HashMap, BTreeMap};
use std::ffi::{CStr, CString};
use std::mem::size_of;
use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct FanReading {
    pub index: u8,
    pub rpm: f32,
    pub key: String, // e.g. "F0Ac"
    pub encoding: String, // e.g. "fpe2"
}

#[derive(Debug, Default, Clone)]
pub struct SmcSnapshot {
    pub gpu_temp_avg: Option<f32>,
    pub fans: Vec<FanReading>,
}

// -------------------- IOKit / SMC FFI --------------------

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn mach_task_self() -> u32;

    fn IOServiceMatching(name: *const c_char) -> *mut c_void;
    fn IOServiceGetMatchingServices(master: u32, matching: *mut c_void, iter: *mut u32) -> i32;
    fn IOIteratorNext(iter: u32) -> u32;
    fn IORegistryEntryGetName(entry: u32, name: *mut c_char) -> i32;
    fn IOObjectRelease(obj: u32) -> i32;

    fn IOServiceOpen(service: u32, owning_task: u32, r#type: u32, connect: *mut u32) -> i32;
    fn IOServiceClose(connect: u32) -> i32;

    fn IOConnectCallStructMethod(
        conn: u32,
        selector: u32,
        input: *const c_void,
        input_cnt: usize,
        output: *mut c_void,
        output_cnt: *mut usize,
    ) -> i32;
}

struct IOServiceIter {
    iter: u32,
}
impl IOServiceIter {
    fn new(class_name: &str) -> Result<Self> {
        let cname = CString::new(class_name)?;
        let matching = unsafe { IOServiceMatching(cname.as_ptr()) };
        if matching.is_null() {
            return Err(anyhow!("IOServiceMatching returned null"));
        }

        let mut iter: u32 = 0;
        let kr = unsafe { IOServiceGetMatchingServices(0, matching, &mut iter) };
        if kr != 0 {
            return Err(anyhow!("IOServiceGetMatchingServices: {}", kr));
        }
        Ok(Self { iter })
    }
}
impl Iterator for IOServiceIter {
    type Item = (u32, String); // (io_service_t, name)
    fn next(&mut self) -> Option<Self::Item> {
        let obj = unsafe { IOIteratorNext(self.iter) };
        if obj == 0 {
            return None;
        }
        let mut buf = [0 as c_char; 128];
        let kr = unsafe { IORegistryEntryGetName(obj, buf.as_mut_ptr()) };
        let name = if kr == 0 {
            unsafe { CStr::from_ptr(buf.as_ptr()) }.to_string_lossy().to_string()
        } else {
            String::new()
        };
        Some((obj, name))
    }
}
impl Drop for IOServiceIter {
    fn drop(&mut self) {
        unsafe { IOObjectRelease(self.iter) };
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct KeyDataVer {
    pub major: u8,
    pub minor: u8,
    pub build: u8,
    pub reserved: u8,
    pub release: u16,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct PLimitData {
    pub version: u16,
    pub length: u16,
    pub cpu_p_limit: u32,
    pub gpu_p_limit: u32,
    pub mem_p_limit: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct KeyInfo {
    pub data_size: u32,
    pub data_type: u32, // FourCC
    pub data_attributes: u8,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct KeyData {
    pub key: u32,
    pub vers: KeyDataVer,
    pub p_limit_data: PLimitData,
    pub key_info: KeyInfo,
    pub result: u8,
    pub status: u8,
    pub data8: u8,
    pub data32: u32,
    pub bytes: [u8; 32],
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct SensorVal {
    pub name: String,
    pub unit: String, // FourCC as string, e.g. "flt "
    pub data: Vec<u8>,
}

#[allow(clippy::upper_case_acronyms)]
pub struct SMC {
    conn: u32,
    cache: HashMap<u32, KeyInfo>,
}

impl SMC {
    pub fn new() -> Result<Self> {
        let mut conn = 0u32;

        // Same idea as macmon: match "AppleSMC" and open the "AppleSMCKeysEndpoint" entry.
        for (service, name) in IOServiceIter::new("AppleSMC")? {
            if name == "AppleSMCKeysEndpoint" {
                let kr = unsafe { IOServiceOpen(service, mach_task_self(), 0, &mut conn) };
                unsafe { IOObjectRelease(service) };
                if kr != 0 {
                    return Err(anyhow!("IOServiceOpen: {}", kr));
                }
                break;
            } else {
                unsafe { IOObjectRelease(service) };
            }
        }

        if conn == 0 {
            return Err(anyhow!("AppleSMCKeysEndpoint not found / open failed"));
        }

        Ok(Self { conn, cache: HashMap::new() })
    }

    fn read_call(&self, input: &KeyData) -> Result<KeyData> {
        let mut out = KeyData::default();
        let mut out_len = size_of::<KeyData>();

        // selector=2 is what macmon uses for AppleSMCKeysEndpoint struct method calls.
        let kr = unsafe {
            IOConnectCallStructMethod(
                self.conn,
                2,
                input as *const _ as *const c_void,
                size_of::<KeyData>(),
                &mut out as *mut _ as *mut c_void,
                &mut out_len,
            )
        };

        if kr != 0 {
            return Err(anyhow!("IOConnectCallStructMethod: {}", kr));
        }
        if out.result == 132 {
            return Err(anyhow!("SMC key not found"));
        }
        if out.result != 0 {
            return Err(anyhow!("SMC error: {}", out.result));
        }
        Ok(out)
    }

    pub fn key_by_index(&self, index: u32) -> Result<String> {
        let indata = KeyData { data8: 8, data32: index, ..Default::default() };
        let out = self.read_call(&indata)?;
        Ok(std::str::from_utf8(&out.key.to_be_bytes())?.to_string())
    }

    pub fn read_key_info(&mut self, key: &str) -> Result<KeyInfo> {
        if key.len() != 4 {
            return Err(anyhow!("SMC key must be 4 chars"));
        }
        let k = fourcc_str_to_u32(key);
        if let Some(ki) = self.cache.get(&k) {
            return Ok(*ki);
        }

        let indata = KeyData { data8: 9, key: k, ..Default::default() };
        let out = self.read_call(&indata)?;
        self.cache.insert(k, out.key_info);
        Ok(out.key_info)
    }

    pub fn read_val(&mut self, key: &str) -> Result<SensorVal> {
        let name = key.to_string();
        let key_info = self.read_key_info(key)?;
        let k = fourcc_str_to_u32(key);

        let indata = KeyData { data8: 5, key: k, key_info, ..Default::default() };
        let out = self.read_call(&indata)?;

        let unit = std::str::from_utf8(&key_info.data_type.to_be_bytes())?.to_string();
        let n = key_info.data_size as usize;
        Ok(SensorVal { name, unit, data: out.bytes[0..n.min(out.bytes.len())].to_vec() })
    }

    pub fn read_all_keys(&mut self) -> Result<Vec<String>> {
        let val = self.read_val("#KEY")?;
        let count = u32::from_be_bytes(val.data[0..4].try_into()?);

        let mut keys = Vec::with_capacity(count as usize);
        for i in 0..count {
            let key = match self.key_by_index(i) {
                Ok(k) => k,
                Err(_) => continue,
            };
            if self.read_val(&key).is_ok() {
                keys.push(key);
            }
        }
        Ok(keys)
    }
}

impl Drop for SMC {
    fn drop(&mut self) {
        unsafe { IOServiceClose(self.conn) };
    }
}

// -------------------- Decoding helpers --------------------

fn fourcc_str_to_u32(s: &str) -> u32 {
    s.bytes().fold(0u32, |acc, b| (acc << 8) | (b as u32))
}

/// Try decode common SMC numeric encodings into f32.
/// - "flt " : 4 bytes little-endian f32 (what macmon uses for temps on macOS 14+)
/// - "fpe2" : 2 bytes big-endian fixed point (value = raw / 4.0) often used for fan RPM
/// - "sp78" : 2 bytes big-endian signed fixed point (value = raw / 256.0) common temp encoding on older Macs
fn decode_numeric(v: &SensorVal) -> Option<f32> {
    match (v.unit.as_str(), v.data.as_slice()) {
        ("flt ", d) if d.len() >= 4 => {
            let raw: [u8; 4] = d[0..4].try_into().ok()?;
            Some(f32::from_le_bytes(raw))
        }
        ("fpe2", d) if d.len() >= 2 => {
            let raw: [u8; 2] = d[0..2].try_into().ok()?;
            let x = u16::from_be_bytes(raw) as f32;
            Some(x / 4.0)
        }
        ("sp78", d) if d.len() >= 2 => {
            let raw: [u8; 2] = d[0..2].try_into().ok()?;
            let x = i16::from_be_bytes(raw) as f32;
            Some(x / 256.0)
        }
        _ => None,
    }
}

fn avg(vals: &[f32]) -> Option<f32> {
    if vals.is_empty() { return None; }
    Some(vals.iter().sum::<f32>() / (vals.len() as f32))
}

// fan key pattern: "F?Ac" (Actual speed). '?' is usually 0..9 (sometimes A..F).
fn fan_index_from_key(k: &str) -> Option<u8> {
    if k.len() != 4 { return None; }
    let b = k.as_bytes();
    if b[0] != b'F' || b[2] != b'A' || b[3] != b'c' { return None; }
    let idx = b[1];
    match idx {
        b'0'..=b'9' => Some(idx - b'0'),
        b'A'..=b'F' => Some(10 + (idx - b'A')),
        b'a'..=b'f' => Some(10 + (idx - b'a')),
        _ => None,
    }
}

// -------------------- Public API: read temps + fans --------------------

/// Read CPU/GPU temps (Tp/Te/Tg) + fan RPM (F?Ac) via SMC.
/// This is intentionally "multi-machine" friendly: it discovers keys at runtime instead of hardcoding a model list.
pub fn read_smc_snapshot() -> Result<SmcSnapshot> {
    let mut smc = SMC::new()?;
    let keys = smc.read_all_keys().unwrap_or_default();

    // macmon filters temps by: data_size==4 && data_type=="flt " and key prefix Tp/Te/Tg
    // We'll keep that as the primary path, but decode a few common alternatives too.
    const FLOAT_TYPE: u32 = 0x666C7420; // "flt "

    let mut gpu_temps = Vec::<f32>::new();

    // fan readings indexed for stable ordering
    let mut fans_map: BTreeMap<u8, FanReading> = BTreeMap::new();

    for k in keys {
        // Fans: detect first (no type restriction; decode_numeric will decide)
        if let Some(idx) = fan_index_from_key(&k) {
            if let Ok(v) = smc.read_val(&k) {
                if let Some(rpm) = decode_numeric(&v) {
                    fans_map.insert(idx, FanReading {
                        index: idx,
                        rpm,
                        key: k.clone(),
                        encoding: v.unit.clone(),
                    });
                }
            }
            continue;
        }

        // Temps: we only care about GPU temps (keys starting with "Tg").
        if !k.starts_with("Tg") {
            continue;
        }

        let ki = match smc.read_key_info(&k) {
            Ok(x) => x,
            Err(_) => continue,
        };

        // Primary path: macmon's flt/4B filter
        let val = if ki.data_size == 4 && ki.data_type == FLOAT_TYPE {
            match smc.read_val(&k) {
                Ok(v) => {
                    let raw = v.data.get(0..4).and_then(|b| b.try_into().ok());
                    raw.map(|arr| (f32::from_le_bytes(arr), v.unit))
                }
                Err(_) => None,
            }
        } else {
            // fallback: attempt decode_numeric for other encodings (sp78, etc.)
            match smc.read_val(&k) {
                Ok(v) => decode_numeric(&v).map(|f| (f, v.unit)),
                Err(_) => None,
            }
        };

        if let Some((temp_c, _unit)) = val {
            if temp_c == 0.0 { continue; }
            gpu_temps.push(temp_c);
        }
    }

    let gpu_avg = avg(&gpu_temps);

    Ok(SmcSnapshot {
        gpu_temp_avg: gpu_avg,
        fans: fans_map.into_values().collect(),
    })
}
