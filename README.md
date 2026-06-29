[![Rust](https://img.shields.io/badge/Rust-1.95.0-blue?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![uv](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/uv/main/assets/badge/v0.json&style=flat-square)](https://docs.astral.sh/uv/)
[![CI](https://img.shields.io/github/actions/workflow/status/javidahmed64592/uv-align/ci.yml?style=flat-square&label=CI&logo=github)](https://github.com/javidahmed64592/uv-align/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/github/actions/workflow/status/javidahmed64592/uv-align/docs.yml?style=flat-square&label=Docs&logo=github)](https://github.com/javidahmed64592/uv-align/actions/workflows/docs.yml)
[![Publish](https://img.shields.io/github/actions/workflow/status/javidahmed64592/uv-align/publish.yml?style=flat-square&label=Publish&logo=github)](https://github.com/javidahmed64592/uv-align/actions/workflows/publish.yml)
[![Release](https://img.shields.io/github/actions/workflow/status/javidahmed64592/uv-align/release.yml?style=flat-square&label=Release&logo=github)](https://github.com/javidahmed64592/uv-align/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

<!-- omit from toc -->
# uv-align

`uv-align` is a command-line tool that helps you keep your Python dependencies in sync between `pyproject.toml` and `uv.lock`.
It checks for out-of-sync dependencies and can update version constraints in `pyproject.toml` accordingly.

It is intended to be used with Python projects that are managed with `uv`.
When executing `uv lock --upgrade`, `uv` updates `uv.lock` with the latest resolved dependecies, but it does not update the version constraints in `pyproject.toml` accordingly.
`uv-align` bridges this gap by checking for out-of-sync dependencies and updating the version constraints in `pyproject.toml` to match the resolved versions in `uv.lock`.

<!-- omit from toc -->
## Table of Contents
- [Quick Guide](#quick-guide)
- [Usage](#usage)
- [License](#license)


## Quick Guide

```sh
uv-align -h      # Show help message
uv-align --check # Check for out-of-sync dependencies between `pyproject.toml` and `uv.lock`
uv-align -y      # Update any out-of-sync version constraints in `pyproject.toml`
uv-align -yu     # Upgrade dependencies and update version constraints in `pyproject.toml`
```

Example output:

![Example Usage](https://github.com/javidahmed64592/uv-align/blob/main/docs/usage.png)

## Usage

```sh
Update dependency constraints using versions resolved by `uv`

Usage: uv-align [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to folder containing `pyproject.toml` and `uv.lock` files [default: .]

Options:
      --check    Show a diff of dependency updates without applying them
  -y, --yes      Automatically apply all changes without prompting
  -u, --upgrade  Upgrade dependencies in `uv.lock` with `uv`
  -v, --verbose  Show detailed information about dependency updates
  -h, --help     Print help
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
