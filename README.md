# `indentex`: An indentation-based superset of LaTeX


[![Build Status](https://travis-ci.org/mp4096/indentex.svg?branch=master)](https://travis-ci.org/mp4096/indentex)


## Installation

### From source
Minimal Rust version: 1.13

```sh
cargo install --git https://github.com/mp4096/indentex/
```

## Usage
Type `indentex -h` for help:

```
indentex 0.3.2
Mikhail Pak <mikhail.pak@tum.de>
Transpiler for an indentation-based superset of LaTeX

USAGE:
    indentex [FLAGS] [OPTIONS] <path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Show transpilation progress

OPTIONS:
    -j, --jobs <jobs>    Use multithreading to speed-up file transpiling

ARGS:
    <path>    Path to a single indentex file or a directory (recursively transpile all indentex files)
```
