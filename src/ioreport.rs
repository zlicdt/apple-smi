// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * ioreport.rs
 * Fetch GPU power data via IOReport private API.
 */

use anyhow::{Result, anyhow};
use libc::c_void;
use std::{
    ffi::{CStr, CString},
    ptr::{null, null_mut},
    time::{Duration, Instant},
};

// ---- Minimal CoreFoundation types ----
type CFTypeRef = *const c_void;
type CFStringRef = *const c_void;
type CFDictionaryRef = *const c_void;
type CFMutableDictionaryRef = *mut c_void;
type CFArrayRef = *const c_void;
type CFAllocatorRef = *const c_void;
type IOReportSubscriptionRef = *mut c_void;

const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFAllocatorDefault: CFAllocatorRef;

    fn CFRelease(obj: CFTypeRef);

    fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        c_str: *const i8,
        encoding: u32,
    ) -> CFStringRef;

    fn CFStringGetCString(
        the_string: CFStringRef,
        buffer: *mut i8,
        buffer_size: isize,
        encoding: u32,
    ) -> i32;

    fn CFDictionaryGetValue(dict: CFDictionaryRef, key: *const c_void) -> *const c_void;
    fn CFDictionaryGetCount(dict: CFDictionaryRef) -> isize;
    fn CFDictionaryCreateMutableCopy(
        alloc: CFAllocatorRef,
        capacity: isize,
        dict: CFDictionaryRef,
    ) -> CFMutableDictionaryRef;

    fn CFArrayGetCount(array: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, idx: isize) -> *const c_void;
}

// ---- IOReport (private API) ----
#[link(name = "IOReport", kind = "dylib")]
unsafe extern "C" {
    fn IOReportCopyChannelsInGroup(
        group: CFStringRef,
        subgroup: CFStringRef,
        a: u64,
        b: u64,
        c: u64,
    ) -> CFDictionaryRef;

    fn IOReportCreateSubscription(
        a: *const c_void,
        b: CFMutableDictionaryRef,
        c: *mut CFMutableDictionaryRef,
        d: u64,
        e: CFTypeRef,
    ) -> IOReportSubscriptionRef;

    fn IOReportCreateSamples(
        subs: IOReportSubscriptionRef,
        channels: CFMutableDictionaryRef,
        a: CFTypeRef,
    ) -> CFDictionaryRef;

    fn IOReportCreateSamplesDelta(
        s1: CFDictionaryRef,
        s2: CFDictionaryRef,
        a: CFTypeRef,
    ) -> CFDictionaryRef;

    fn IOReportChannelGetChannelName(chan: CFDictionaryRef) -> CFStringRef;
    fn IOReportChannelGetUnitLabel(chan: CFDictionaryRef) -> CFStringRef;

    fn IOReportSimpleGetIntegerValue(chan: CFDictionaryRef, idx: i32) -> i64;
}

fn cfstr(s: &str) -> Result<CFStringRef> {
    let c = CString::new(s)?;
    let r = unsafe {
        CFStringCreateWithCString(kCFAllocatorDefault, c.as_ptr(), K_CF_STRING_ENCODING_UTF8)
    };
    if r.is_null() {
        Err(anyhow!("CFStringCreateWithCString failed for {s}"))
    } else {
        Ok(r)
    }
}

fn from_cfstr(s: CFStringRef) -> String {
    if s.is_null() {
        return String::new();
    }
    unsafe {
        let mut buf = vec![0i8; 256];
        let ok = CFStringGetCString(
            s,
            buf.as_mut_ptr(),
            buf.len() as isize,
            K_CF_STRING_ENCODING_UTF8,
        );
        if ok == 0 {
            return String::new();
        }
        CStr::from_ptr(buf.as_ptr()).to_string_lossy().to_string()
    }
}

fn dict_get(dict: CFDictionaryRef, key: &str) -> Option<*const c_void> {
    let k = cfstr(key).ok()?;
    let v = unsafe { CFDictionaryGetValue(dict, k as *const c_void) };
    unsafe { CFRelease(k as CFTypeRef) };
    if v.is_null() { None } else { Some(v) }
}

fn energy_delta_to_watts(energy_delta: f32, unit: &str, dt_ms: u64) -> Result<f32> {
    // P(W) = E(J) / t(s)
    let per_sec = energy_delta / (dt_ms as f32 / 1000.0);
    match unit.trim() {
        "mJ" => Ok(per_sec / 1e3),
        "uJ" => Ok(per_sec / 1e6),
        "nJ" => Ok(per_sec / 1e9),
        other => Err(anyhow!("Unknown energy unit: {other}")),
    }
}

/// Persistent subscription + previous sample caching (macmon-style).
struct EnergyModelSampler {
    subs: IOReportSubscriptionRef,
    chan: CFMutableDictionaryRef,
    prev: Option<(CFDictionaryRef, Instant)>,
}

impl EnergyModelSampler {
    /// Subscribe to IOReport group: "Energy Model".
    fn new() -> Result<Self> {
        let group = cfstr("Energy Model")?;
        let channels = unsafe { IOReportCopyChannelsInGroup(group, null(), 0, 0, 0) };
        unsafe { CFRelease(group as CFTypeRef) };

        if channels.is_null() {
            return Err(anyhow!(
                "IOReportCopyChannelsInGroup(\"Energy Model\") failed"
            ));
        }

        let chan = unsafe {
            let count = CFDictionaryGetCount(channels);
            let m = CFDictionaryCreateMutableCopy(kCFAllocatorDefault, count, channels);
            CFRelease(channels as CFTypeRef);
            m
        };
        if chan.is_null() {
            return Err(anyhow!("CFDictionaryCreateMutableCopy failed"));
        }

        let mut subbed: CFMutableDictionaryRef = null_mut();
        let subs = unsafe { IOReportCreateSubscription(null(), chan, &mut subbed, 0, null()) };
        unsafe {
            if !subbed.is_null() {
                CFRelease(subbed as CFTypeRef);
            }
        }

        if subs.is_null() {
            unsafe { CFRelease(chan as CFTypeRef) };
            return Err(anyhow!("IOReportCreateSubscription failed"));
        }

        Ok(Self {
            subs,
            chan,
            prev: None,
        })
    }

    fn raw_sample(&self) -> (CFDictionaryRef, Instant) {
        let s = unsafe { IOReportCreateSamples(self.subs, self.chan, null()) };
        (s, Instant::now())
    }

    /// Sample GPU power in Watts over `window_ms`, averaging over `slices` sub-samples.
    ///
    /// macmon splits each refresh window into 4 slices by default to reduce jitter.
    fn sample_gpu_power_w(&mut self, window_ms: u64, slices: usize) -> Result<Option<f32>> {
        let slices = slices.clamp(1, 32);
        let step = window_ms / slices as u64;

        let mut prev = self.prev.take().unwrap_or_else(|| self.raw_sample());
        let mut acc_watts = 0.0f32;
        let mut got_any = false;

        for _ in 0..slices {
            std::thread::sleep(Duration::from_millis(step));
            let next = self.raw_sample();

            // Use *real* elapsed time (macmon does this), avoid division by zero.
            let dt_ms = next.1.duration_since(prev.1).as_millis().max(1) as u64;

            let delta = unsafe { IOReportCreateSamplesDelta(prev.0, next.0, null()) };
            unsafe { CFRelease(prev.0 as CFTypeRef) };
            prev = next;

            if delta.is_null() {
                continue;
            }

            let arr = match dict_get(delta, "IOReportChannels") {
                Some(v) => v as CFArrayRef,
                None => {
                    unsafe { CFRelease(delta as CFTypeRef) };
                    continue;
                }
            };

            let n = unsafe { CFArrayGetCount(arr) };
            for i in 0..n {
                let item = unsafe { CFArrayGetValueAtIndex(arr, i) } as CFDictionaryRef;
                if item.is_null() {
                    continue;
                }

                let chan_name = from_cfstr(unsafe { IOReportChannelGetChannelName(item) });

                // macmon targets "GPU Energy" (some models may have a prefix; suffix match is safer).
                if chan_name != "GPU Energy" && !chan_name.ends_with("GPU Energy") {
                    continue;
                }

                let unit = from_cfstr(unsafe { IOReportChannelGetUnitLabel(item) });
                let val = unsafe { IOReportSimpleGetIntegerValue(item, 0) } as f32;

                acc_watts += energy_delta_to_watts(val, &unit, dt_ms)?;
                got_any = true;
            }

            unsafe { CFRelease(delta as CFTypeRef) };
        }

        self.prev = Some(prev);

        if got_any {
            Ok(Some(acc_watts / slices as f32))
        } else {
            Ok(None)
        }
    }

    /// Convenience: macmon-like default (4 slices).
    fn sample_gpu_power_w_default(&mut self, window_ms: u64) -> Result<Option<f32>> {
        self.sample_gpu_power_w(window_ms, 4)
    }
}

impl Drop for EnergyModelSampler {
    fn drop(&mut self) {
        unsafe {
            if let Some((p, _)) = self.prev.take() {
                if !p.is_null() {
                    CFRelease(p as CFTypeRef);
                }
            }
            if !self.chan.is_null() {
                CFRelease(self.chan as CFTypeRef);
            }
            if !self.subs.is_null() {
                CFRelease(self.subs as CFTypeRef);
            }
        }
    }
}

/// One-shot helper (creates subscription each call).
/// Prefer using `EnergyModelSampler` for repeated sampling.
pub fn sample_gpu_power_once(window_ms: u64) -> Result<Option<f32>> {
    let mut s = EnergyModelSampler::new()?;
    s.sample_gpu_power_w_default(window_ms)
}
