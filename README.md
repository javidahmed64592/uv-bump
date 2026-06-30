[![PyPI](https://img.shields.io/pypi/v/uv-align?style=flat-square)](https://pypi.org/project/uv-align/)
[![Rust](https://img.shields.io/badge/Rust-1.95.0-blue?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![uv](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/uv/main/assets/badge/v0.json&style=flat-square)](https://docs.astral.sh/uv/)
[![CI](https://img.shields.io/github/actions/workflow/status/javidahmed64592/uv-align/ci.yml?style=flat-square&label=CI&logo=github)](https://github.com/javidahmed64592/uv-align/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/github/actions/workflow/status/javidahmed64592/uv-align/docs.yml?style=flat-square&label=Docs&logo=github)](https://github.com/javidahmed64592/uv-align/actions/workflows/docs.yml)
[![Publish](https://img.shields.io/github/actions/workflow/status/javidahmed64592/uv-align/publish.yml?style=flat-square&label=Publish&logo=github)](https://github.com/javidahmed64592/uv-align/actions/workflows/publish.yml)
[![Release](https://img.shields.io/github/actions/workflow/status/javidahmed64592/uv-align/release.yml?style=flat-square&label=Release&logo=github)](https://github.com/javidahmed64592/uv-align/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

<!-- omit from toc -->
# uv-align

`uv-align` is a Rust command-line tool that keeps your Python dependency constraints in sync between `pyproject.toml` and `uv.lock`.

When you run `uv lock --upgrade`, `uv` resolves and updates `uv.lock` with the latest compatible versions, but leaves the version constraints in `pyproject.toml` untouched.
This means your declared constraints can drift from what `uv` actually resolved - `uv-align` bridges that gap.

It reads the resolved versions from `uv.lock` and updates the version numbers in `pyproject.toml` accordingly, preserving your existing operators (`>=`, `==`, etc.), upper bounds, environment markers, and formatting.
It does not require a virtual environment as it does not install packages, and never modifies `uv.lock` directly (`uv` is responsible for that).

**Links:**

- [PyPI](https://pypi.org/project/uv-align/)
- [GitHub](https://github.com/javidahmed64592/uv-align)

<!-- omit from toc -->
## Table of Contents
- [Installation](#installation)
- [Quick Guide](#quick-guide)
- [Usage](#usage)
- [How It Works](#how-it-works)
- [License](#license)

## Installation

**As a tool (recommended)**

If you use `uv`, you can install `uv-align` as a global tool:

```sh
uv tool install uv-align
```

**As a dev dependency**

To pin `uv-align` to a specific project, add it to your development dependencies in `pyproject.toml`:

```sh
uv add uv-align --optional dev
```

**As a pre-built binary**

Pre-built binaries for Linux, macOS, and Windows are available on the [GitHub Releases page](https://github.com/javidahmed64592/uv-align/releases).

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

```
Align `pyproject.toml` dependency constraints with versions resolved by `uv`

Usage: uv-align [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to folder containing `pyproject.toml` and `uv.lock` files [default: .]

Options:
      --check    Show a diff of dependency updates without applying them
  -y, --yes      Automatically apply all changes without prompting
  -u, --upgrade  Upgrade dependencies in `uv.lock` with `uv lock --upgrade`
  -v, --verbose  Show detailed information about dependency updates
  -h, --help     Print help
```

## How It Works

`uv-align` is a file transformer - in its default mode it reads two files and writes one, with no network access or environment inspection.
The optional `--upgrade` flag adds a `uv` invocation that does require network access to fetch updated package metadata.

**Step 1 - Parse `pyproject.toml`**

All dependency entries are read from three locations:

- `[project.dependencies]` - main dependencies
- `[project.optional-dependencies]` - extras groups (e.g. `dev`, `docs`)
- `[dependency-groups]` - PEP 735 groups supported by uv

Each entry is parsed as a PEP 508 string.
The package name, version operator (`>=`, `==`, `~=`, etc.), lower-bound version, and any suffix constraints (e.g. `,<1.0`) are extracted separately.
Git/URL dependencies (e.g. `package @ git+https://...`) are included in the parsed output but have no version to compare, so they are naturally skipped during the diff step.
Extras (e.g. `package[extra]`) and environment markers (e.g. `; python_version >= '3.11'`) are stripped as they are not relevant to version bumping.
Package names are normalised per PEP 503 (lowercase, runs of `[-_.]` collapsed to `-`) to ensure consistent matching.

**Step 2 - Parse `uv.lock`**

Every `[[package]]` entry in the lockfile is read to build a list of resolved package names and versions.
Package names are normalised in the same way as step 1, so that `my_package` and `my-package` match correctly.
Packages without a `version` field (git/URL sources) are skipped naturally by the parser.

**Step 3 - Compute the diff**

Each dependency from `pyproject.toml` that has a resolved version is looked up in the lockfile by its normalised name.
If the resolved version differs from the declared version, a change is recorded.
Version strings are compared after normalising trailing `.0` components, so `0.25.0` and `0.25` are treated as equal and no spurious change is emitted.

**Step 4 - Report or apply**

In `--check` mode, the diff is printed and the tool exits with code 1 if any changes are needed (suitable for CI), or code 0 if everything is already in sync.
Otherwise, the user is prompted to confirm before changes are applied (or `-y` skips the prompt).
Changes are written back to `pyproject.toml` using `toml_edit` - a format-preserving TOML library that modifies only the version numbers, leaving all comments, whitespace, and key ordering intact.
The original operator and any suffix constraints are preserved verbatim.

**Step 5 - Upgrade mode (`-u`)**

With `--upgrade`, `uv lock --upgrade` runs first to fetch and resolve the latest compatible versions, updating `uv.lock` accordingly.
Steps 1–4 then run as normal to align `pyproject.toml` with the newly resolved versions.
The number of updated, added, and removed packages reported by `uv` is printed as a summary, with full package details available via `--verbose`.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
