# Justext

> Just a text editor. No bloat, no lag, just text. 🦀

![Rust](https://img.shields.io/badge/Language-Rust-orange)
![License](https://img.shields.io/badge/License-MIT-red)
![Speed](https://img.shields.io/badge/Speed-Blazing_Fast-purple)

| Platform | Arch   | Bundle                                                         |
| -------- | ------ | -------------------------------------------------------------- |
| Linux    | x86_64 | .rpm                                                           |
| Linux    | x86_64 | .tar.gz                                                        |
| Windows  | x86_64 | To be compiled                                                 |
| MacOS    | -      | Build from source (I don't deal with Apple's closed ecosystem) |

## Screenshot

<img src="https://github.com/sunaipa5/justext/blob/main/assets/ss1.png?raw=true" width="450">

## Build

Requires [Rust](https://rust-lang.org/)

### With [task](https://taskfile.dev/)

**Binary only:**

```sh
task build
```

#### Bundle

**As .tar.gz bundle:**

```sh
task build-gz
```

**As .rpm package:**

> Requires [rpmdude](https://github.com/sunaipa5/rpmdude)

```sh
task build-rpm
```

## Manual

```sh
cargo build --release
```
