# Apple SMI

Lightweight macOS GPU inspector inspired by `nvidia-smi`. It shells out to `system_profiler` and `powermetrics`, normalizes vendor/bus/Metal labels, and prints a concise per-GPU line.

## Requirements
- macOS
- Rust toolchain (stable) and Cargo

## Tips
- `powermetrics` requires root permissions.
- Developing materials and documents placed in `docs` folder.

## Test Run
```sh
cargo run
```

## Contributing
Issues and PRs welcome. Run `cargo fmt` and `cargo clippy` before sending changes.
