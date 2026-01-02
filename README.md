# Apple SMI

**Apple Silicon System Management Interface**

## Overview

Lightweight macOS GPU inspector inspired by `nvidia-smi`. It shells out to `system_profiler` `powermetrics` `ioreg`, and get information from `SMC`.

<img width="1000" alt="image" src="https://github.com/user-attachments/assets/0623fe79-334f-4a3e-945e-5bf49af07fb7" />


## Requirements
- macOS
- Rust toolchain and Cargo

## Tips
- Using of `powermetrics` means requires root permissions.
- Developing materials and documents placed in `docs` folder.

## Test Run
```sh
cargo run
```

## Contributing
Issues and PRs welcome. Run `cargo fmt` and `cargo clippy` before sending changes.
