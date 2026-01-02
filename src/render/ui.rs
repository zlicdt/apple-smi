// SPDX-License-Identifier: MIT
/*
 * apple-smi: Apple Silicon System Management Interface
 * Copyright (C) 2026 zlicdt@ReSpringClipsNeko
 * ui.rs
 * Construct output text.
*/
use crate::utils;
use crate::syspf;
use crate::pwrmtcs;
use crate::mtlapi;
use crate::ioreg;

fn pad(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        let mut out = String::with_capacity(width);
        out.push_str(s);
        out.extend(std::iter::repeat(' ').take(width - s.len()));
        out
    }
}

// Whoh a geek print function, I'm genius!
pub fn print_div_str(dtype: i32) {
    /*
     * dtype:
     * 0: "+-----------------------------------------------------------------------------------------+"
     * 1: "+-----------------------------------------+------------------------+----------------------+"
     * 2: "|=========================================+========================+======================|"
     * 2: "|=========================================================================================|"
     */
    let (segments, start, sep, end, fill): (&[usize], char, char, char, char) = match dtype {
        0 => (&[89], '+', '+', '+', '-'),
        1 => (&[41, 24, 22], '+', '+', '+', '-'),
        // "|===+===+===|" style divider
        2 => (&[41, 24, 22], '|', '+', '|', '='),
        3 => (&[89], '|', '|', '|', '='),
        _ => (&[], '+', '+', '+', '-'),
    };

    if segments.is_empty() {
        return;
    }

    // Build divider like "+---+---+" or "|===+===|" based on config.
    let line: String = std::iter::once(start)
        .chain(segments.iter().enumerate().flat_map(|(idx, n)| {
            let fill_iter = std::iter::repeat(fill).take(*n);
            let tail = if idx + 1 == segments.len() { end } else { sep };
            fill_iter.chain(std::iter::once(tail))
        }))
        .collect();

    println!("{}", line);
}

pub fn print_header_line(os_version: &str, metal_version: &str) {
    // Columns must align with type 1 divider: segments 41, 24, 22.
    const SEGMENTS: [usize; 3] = [36, 30, 23];
    let container: [String; 3] = [
        format!(" Apple-SMI {}", utils::project_version()), // leading space per requirement
        format!("macOS Version: {}", os_version),
        format!("Metal Version: {}", metal_version),
    ];

    let mut line = String::from("|");
    for col in 0..3 {
        line.push_str(&pad(container[col].as_str(), SEGMENTS[col]));
    }
    line.push('|');

    println!("{}", line);
}

pub fn print_title() {
    // Columns must align with type 1 divider: segments 41, 24, 22.
    const SEGMENTS: [[usize; 3]; 3] = [[41, 25, 23], [41, 25, 23], [41, 25, 23]];
    let container: [[String; 3]; 3] = [
        [
            String::from(" GPU  Name                     Frequency"), // leading space per requirement
            String::from("| Bus-Id          Disp.A"),
            String::from("|"),
        ],
        [
            String::from(" Fan  Temp  Perf               Pwr:Usage"),
            String::from("|           Memory-Usage"),
            String::from("| GPU-Util  Compute M."),
        ],
        [String::from(""), String::from("|"), String::from("|")],
    ];

    let mut line = String::from("|");
    for row in 0..3 {
        for col in 0..3 {
            line.push_str(&pad(&container[row][col], SEGMENTS[row][col]));
        }
        line.push('|');
        println!("{}", line);
        line.clear();
        line.push('|');
    }
}

pub fn print_card(i: usize, g: &syspf::GpuEntry, p: &pwrmtcs::GpuMetrics, v: &ioreg::VramInfo) {
    let name: &str = g.name.as_str();
    let bus: &str = g.bus_label();
    let freq = match p.gpu_hw_freq {
        Some(v) => format!("{:>4}", v), // right-align to 4 chars, fill leading spaces as needed
        None => String::from(" N/A"),
    };
    let status = match mtlapi::gpu_initialized_probe() {
        true => "On",
        false => "Off",
    };
    let disp_a = format!("{:>3}", status);
    let pwr = match p.gpu_pwr {
        Some(v) => format!("{:>6}", v),
        None => format!("{:>6}", "N/A"),
    };
    let gpu_sw_state = match p.gpu_sw_state {
        Some(idx) => format!("P{}", idx),
        None => String::from("N/A"),
    };
    let vram_status = match (v.inuse_vram, v.alloc_vram) {
        (Some(inuse), Some(alloc)) => format!("{:>22}", format!("{}MiB / {}MiB", inuse, alloc)),
        _ => String::from("N/A"),
    };
    const SEGMENTS: [[usize; 3]; 3] = [[32, 30, 27], [31, 12, 46], [41, 25, 23]];
    let container: [[String; 3]; 3] = [
        [
            format!("   {}  {}", i, name), // leading space per requirement
            format!("{} MHz |   {}", freq, bus), // TODO: display status from syspf display list
            format!("{} |", disp_a),
        ],
        // TODO: Fill real data by powermetrics
        [
            format!(" N/A  Temp  {}", gpu_sw_state), // Fan speed and Temp not available
            format!("{} mW |", pwr),
            format!("{} | GPU-Util     Default", vram_status),
        ],
        // Line 3! empty line
        [String::from(""), String::from("|"), String::from("|")],
    ];

    let mut line = String::from("|");
    for row in 0..3 {
        for col in 0..3 {
            line.push_str(&pad(&container[row][col], SEGMENTS[row][col]));
        }
        line.push('|');
        println!("{}", line);
        line.clear();
        line.push('|');
    }
    print_div_str(1);
}

pub fn print_empty_line() {
    println!();
}

pub fn print_tprocess_header() {
    print_div_str(0);
    let container: [String; 3] = [
        String::from(" Processes:"),
        String::from(
            "   GPU   GI   CI              PID   Type   Process name                       GPU Memory",
        ),
        String::from(
            "        ID   ID                                                               Usage",
        ),
    ];
    let mut line = String::from("|");
    for row in 0..3 {
        line.push_str(&pad(&container[row], 89));
        line.push('|');
        println!("{}", line);
        line.clear();
        line.push('|');
    }
    print_div_str(3);
}
