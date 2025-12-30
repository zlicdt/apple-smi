# Apple SMI

Lightweight macOS GPU inspector inspired by `nvidia-smi`. It shells out to `system_profiler`, normalizes vendor/bus/Metal labels, and prints a concise per-GPU line.

## Requirements
- macOS
- Rust toolchain (stable) and Cargo

## Test Run
```sh
cargo run
```

## Contributing
Issues and PRs welcome. Run `cargo fmt` and `cargo clippy` before sending changes.
