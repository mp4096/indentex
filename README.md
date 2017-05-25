# `indentex`: An indentation-based superset of LaTeX


[![Travis Build Status](https://travis-ci.org/mp4096/indentex.svg?branch=master)](https://travis-ci.org/mp4096/indentex)
[![Appveyor Build status](https://ci.appveyor.com/api/projects/status/uyu5ku0e80fo6t88/branch/master?svg=true)](https://ci.appveyor.com/project/mp4096/indentex/branch/master)
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
        --disable-do-not-edit    Disable prepending the 'DO NOT EDIT' notice
        --flatten-output         Remove all indentation from the output
    -h, --help                   Prints help information
    -V, --version                Prints version information
    -v, --verbose                Show transpilation progress

ARGS:
    <path>    Path to a single indentex file or a directory (recursively transpile all indentex files)
```
