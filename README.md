# Skye

A cli for sync setup.json of bizyair cce dockerfile.

## Requirements

Rust

## Usage

```shell
cargo run -- --help
   Compiling skye v0.1.0 (/Volumes/Tuo-APFS/workspace/skye)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.82s
     Running `target/debug/skye --help`
A cli for sync setup.json of bizyair cce dockerfile.

Usage: skye [OPTIONS] --source <SOURCE> --target <TARGET>

Options:
  -s, --source <SOURCE>     Source file path with json format
  -t, --target <TARGET>     Target file path with json format
  -o, --output <OUTPUT>     New target file saved path with json format
  -a, --append              Whether to append missing object from source to target
  -i, --ignore <IGNORE>...  Objects you want to ignore, which is identified by `path`
      --with-update-at      With(or add) update_at field
  -h, --help                Print help
  -V, --version             Print version
```

### Eg

```shell
cargo update
cargo run -- --source setup-gpu.json --target setup-musa.json --output new-setup-musa.json
```