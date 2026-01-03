// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * mtlapi.rs
 * Check is the gpu initialized via Metal API.
*/
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {}

use objc2_metal::{
    MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandQueue, MTLCreateSystemDefaultDevice,
    MTLDevice,
};

pub fn gpu_initialized_probe() -> bool {
    // Get default Metal device
    let device = match MTLCreateSystemDefaultDevice() {
        Some(d) => d,
        None => return false,
    };

    let queue = match device.newCommandQueue() {
        Some(q) => q,
        None => return false,
    };

    // Command buffer
    let cb = match queue.commandBuffer() {
        Some(cb) => cb,
        None => return false,
    };

    cb.commit();
    cb.waitUntilCompleted();

    // Return status
    let ok = matches!(cb.status(), MTLCommandBufferStatus::Completed) && cb.error().is_none();
    ok
}
