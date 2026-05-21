# Skye

A cli for sync setup.json of bizyair cce dockerfile.

## Requirements

Rust

## Usage

```shell
cargo run -- --help # for help

Usage: skye [OPTIONS] --source <SOURCE> --target <TARGET>

Options:
-s, --source <SOURCE>  Source file path with json format
-t, --target <TARGET>  Target file path with json format
-o, --output <OUTPUT>  New target file saved path with json format
-a, --append           Whether to append missing object from source to target
-h, --help             Print help
-V, --version          Print version
```

### Eg

```shell
cargo update
cargo run -- --source setup-gpu.json --target setup-musa.json --output new-setup-musa.json
```