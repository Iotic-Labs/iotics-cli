# IOTICS CLI

## Prerequisites

- [Rust][toolchain]
- [Golang][golang]
- [Clang][clang]

## Usage

Only from this repository currently. \
TODO: publish to crates.io so that it can be installed as a standalone command and be easier to use.

### Config

Rename/copy `configuration/sample.yaml` and fill it in. You can have as many configuration files you like.

### Help

```bash
cargo run -- --help
```

```bash
cargo run -- delete-twins-by-model --help
cargo run -- delete-all-twins --help
cargo run -- follow-by-model --help
```

[toolchain]: https://rustup.rs
[golang]: https://golang.org/doc/install
[clang]: https://clang.llvm.org/get_started.html
