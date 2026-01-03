# Apple SMI
**Apple Silicon System Management Interface**

[![Build](https://github.com/zlicdt/apple-smi/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/zlicdt/apple-smi/actions/workflows/build.yml)
[![Release](https://img.shields.io/github/v/release/zlicdt/apple-smi?style=flat)](https://github.com/zlicdt/apple-smi/releases)
[![Downloads](https://img.shields.io/github/downloads/zlicdt/apple-smi/total?style=flat)](https://github.com/zlicdt/apple-smi/releases)
[![License](https://img.shields.io/github/license/zlicdt/apple-smi?style=flat)](https://github.com/zlicdt/apple-smi/blob/main/LICENSE)
[![Stars](https://img.shields.io/github/stars/zlicdt/apple-smi?style=flat)](https://github.com/zlicdt/apple-smi/stargazers)

`nvidia-smi` like cli tool for Apple Silicon (macOS)

## Overview

Lightweight macOS GPU inspector inspired by `nvidia-smi`.
Shows GPU frequency / power / memory usage / utilization in a familiar table.

<img width="1010" alt="image" src="https://github.com/user-attachments/assets/d7b38bdb-523f-488d-9705-03ce29b63d55" />

## Quick start
**Project is still high-speed developing, it is NOT stable now**
### Homebrew (Recommended)
If you have `brew`, you can:
```bash
brew install zlicdt/tap/apple-smi
```

### Download
Go to **Releases** and download the prebuilt binary for your Mac.

### Build from source
To build this, ensure the system have **[Rust toolchain and Cargo](https://rust-lang.org/learn/get-started/)** first.
```bash
git clone --depth=1 https://github.com/zlicdt/apple-smi
cargo build --release
sudo ./target/release/apple-smi
```

## Requirements
- Apple Silicon Macs
- Because of use of `powermetrics`, this program needs **root** permissions to measure
    - Performance state
    - Frequency
    - Power
    - GPU utilization

    **Rootless is working in progress...**

## Tips
- Using of `powermetrics` means requires root permissions.
- Developing materials and documents placed in `docs` folder.

## Test Run
```sh
cargo run
```

## How it works
It shells out to `system_profiler` `powermetrics` `ioreg`, and get information from `SMC`.

## Contributing
Issues and PRs welcome. Run `cargo fmt` and `cargo clippy` before sending changes.
