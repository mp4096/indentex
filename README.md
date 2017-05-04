# `indentex`: An indentation-based superset of LaTeX


[![Build Status](https://travis-ci.org/mp4096/indentex.svg?branch=master)](https://travis-ci.org/mp4096/indentex)
[![Dependency Status](https://www.versioneye.com/user/projects/590b1f9dda0c25003951c568/badge.svg)](https://www.versioneye.com/user/projects/590b1f9dda0c25003951c568)
[![Coverage Status](https://coveralls.io/repos/github/mp4096/indentex/badge.svg?branch=master)](https://coveralls.io/github/mp4096/indentex?branch=master)


## Installation

### From source
Minimal Rust version: 1.13

```sh
cargo install --git https://github.com/mp4096/indentex/
```

## Usage
Type `indentex -h` for help:

```
indentex 0.4.0
Mikhail Pak <mikhail.pak@tum.de>
Transpiler for an indentation-based superset of LaTeX

USAGE:
    indentex [FLAGS] <path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Show transpilation progress

ARGS:
    <path>    Path to a single indentex file or a directory (recursively transpile all indentex files)
```
