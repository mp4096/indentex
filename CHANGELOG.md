:warning: Before the 1.0 release, we only document minor releases here!

# 0.5.0-beta
[PR #49](https://github.com/mp4096/indentex/pull/49)

### :warning: Breaking changes

* No backslashes are now allowed in the command name part.

| Indentex source  | Transpiled with `0.4.0`         | Transpiled with `0.5.0`     |
|:-----------------|:--------------------------------|:----------------------------|
| `# foo\bar: baz` | `\foo\bar{baz}`                 | `\foo\bar{baz}`             |
| `# foo\bar:`     | `\begin{foo\bar}⏎\end{foo\bar}` | `\begin{foo}\bar⏎\end{foo}` |

### Non-breaking changes

* Migrated to nom 5
* Changed parsing logic
* Refactor code
* Add unit tests

# 0.4.0 (2017-02-18)
Commit 2adba137618d72c0251ddedfbbe01cc076536ce7

* (not really breaking) implemented multithreading using `rayon`

# 0.3.0 (2017-01-07)
Commit f8f15fb9581b77310ffabac54525c75c5d0d7be6

* Fixed a bug in comments handling when a percent sign is interpreted as an option:
`# section % baz: foo bar` → `\section% baz{foo bar}`

# 0.2.0 (2017-01-07)
Commit 4b71c5170804555d8820daeb365c4c86505cb20b
* Indentex is now able to handle comments in hashlines

# 0.1.0 (2016-11-06)
Commit d39ff770f8006504d2902ff89e4620965b952f0b

* First working version

# Versioning convention (from 0.4.0 onwards)

Indentex is versioned according to [SemVer 2.0](http://semver.org/spec/v2.0.0.html).
We define public API as the
* language specification and transpilation behaviour
* and the CLI interface, e.g. flags, options etc.

Since Indentex is still in the initial development phase (`0.y.z`),
the public API should be treated as generally unstable.
Although the core Indentex syntax has not changed much,
handling of edge cases can be changed at any time.

We try to denote a breaking language change with an increase in the _minor version_,
e.g `0.1.0` → `0.2.0`.
An increase in the _patch version_ (`0.1.0` → `0.2.0`) denotes any non-breaking improvement,
e.g. bug fixing or new features.
