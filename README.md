<div align="center">
  <img width="40%", src="doc/images/logo.png"><br><br>
</div>


[![Travis Build Status](https://travis-ci.org/mp4096/indentex.svg?branch=master)](https://travis-ci.org/mp4096/indentex)
[![Appveyor Build status](https://ci.appveyor.com/api/projects/status/uyu5ku0e80fo6t88/branch/master?svg=true)](https://ci.appveyor.com/project/mp4096/indentex/branch/master)
[![Coverage Status](https://coveralls.io/repos/github/mp4096/indentex/badge.svg?branch=master)](https://coveralls.io/github/mp4096/indentex?branch=master)

[![Chocolatey](https://img.shields.io/chocolatey/v/indentex.svg)](https://chocolatey.org/packages/indentex/)


Indentex is an indentation-based superset of LaTeX.
An Indentex source file is more concise and visually less cluttered than an equivalent
LaTeX file. Its indentation-based syntax was inspired by Python.
Transpiling an Indentex source file yields a plain LaTeX file,
which can be used further in your toolchain, sent to a publisher or your colleagues.


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
