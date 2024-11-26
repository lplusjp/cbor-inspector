# cbor-inspector

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-Edition%202021-orange)](https://www.rust-lang.org/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](#contributing)

## Install

### from pre-built binaries

Pre-built binaries are available for download on the [releases](https://github.com/lplusjp/cbor-inspector/releases) page.

### from source code

```
cargo install --git https://github.com/lplusjp/cbor-inspector.git
```

## Usage

Without any options, cbor-inspector reads CBOR data and writes the tree structure.

```
$ cbor-inspector < cbor.bin
82          -- array(0x2 = 2)
   01       -- unsigned(0x1) = 1
   62       -- tstr(2)
      4142  -- "AB"
```

The `--hex` option allows input in hex format.

```
$ cbor-inspector --hex
82 01 624142
^D
82          -- array(0x2 = 2)
   01       -- unsigned(0x1) = 1
   62       -- tstr(2)
      4142  -- "AB"
```

## License

This repository is licensed under the [MIT License](LICENSE).
