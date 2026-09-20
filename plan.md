# Plan — `plhistr`: terminal histogram tool

## Goal

Build a small Rust CLI named **`plhistr`** that reads tabular data from **stdin**
(or an optional file), and prints a **histogram** (frequency table with a
horizontal bar plot) to the terminal.

It is derived from the existing repo `/mnt/toshiba/00_CODES/RUST_Codes/04_clw/clw`
by *stripping out* its histogram functionality (the `freq` command + `--plot`
output) and turning it into a standalone tool. No code is written from scratch;
the histogram logic is lifted from `clw` and adapted.

## Requirements coverage (from `project_instructions.md`)

| # | Requirement | How it is satisfied |
|---|---|---|
| 2 | Don't write from scratch | Copy/adapt `freq.rs` → `histogram.rs`, `utils.rs` (reader + delimiter detection) from `clw` |
| 3 | Read & understand `clw` | Studied `src/freq.rs`, `src/utils.rs`, `src/main.rs`, `tests/freq_integration_test.rs`, `.github/workflows/*` |
| 4 | Strip histogram func into this repo | `freq` (frequency counting + bar plot) becomes `plhistr` |
| 5 | Cross-platform build + release via GitHub Actions | `release.yml` matrix: Linux (musl, static — runs on RHEL9), macOS (x86_64 + aarch64), Windows (x86_64) |
| 6 | Push to `https://github.com/sshu2017/plhistr`, Actions releases binaries for 3 OSes | `git init`, remote `git@github.com:sshu2017/plhistr.git`, tag `v0.1.0` triggers `release.yml` |
| 7 | Include test cases, all pass | Unit tests (`detect_delimiter`) + integration tests (assert_cmd), fixtures |
| 8 | Plan first, review, then code | This file |
| 9 | Code only after plan review | Proceed after self-review below |

## What "histogram" means here

Faithful to `clw`: a **categorical frequency histogram** — count occurrences of
each unique value in a chosen column, sort them, and draw a horizontal bar per
value scaled to the max count. This is exactly `clw freq --plot`.

## CLI design

```
plhistr [OPTIONS] [FILE]

ARGS:
    [FILE]    Input CSV file. Reads from stdin when omitted or "-".

OPTIONS:
    -c, --column <COLUMN>  Column to histogram.
                           With a header (default): column NAME (default: first column).
                           With --no-header: 1-based column INDEX (default: 1).
        --no-header        Treat the first row as data (no header row).
        --sort-index       Sort by value (numeric if all numeric, else alphabetical)
                           instead of by frequency (default).
        --no-plot          Print the frequency table without bars.
    -h, --help
    -V, --version
```

Delimiter auto-detection (comma `,`, pipe `|`, tab `\t`, space) is reused from
`clw::utils::detect_delimiter`.

## File layout

```
plhistr/
├── plan.md
├── Cargo.toml
├── Cargo.lock            (generated)
├── LICENSE               (MIT)
├── README.md
├── .gitignore
├── src/
│   ├── main.rs           clap CLI, dispatch
│   ├── histogram.rs      freq counting + bar/table printing (from freq.rs)
│   └── utils.rs          input_reader + detect_delimiter (from utils.rs)
├── tests/
│   ├── histogram_integration_test.rs
│   └── fixtures/
│       ├── sample_comma.csv
│       ├── sample_pipe.csv
│       ├── single_column.csv
│       └── empty.csv
└── .github/workflows/
    ├── ci.yml
    └── release.yml
```

## Implementation notes

1. **`Cargo.toml`**: package `plhistr` v0.1.0, edition 2021.
   Deps: `atty`, `clap` (derive), `colored`, `csv`, `indicatif`.
   Dev-deps: `assert_cmd`, `predicates`. (`rand`, `tempfile`, python stuff dropped.)
2. **`histogram.rs`**: adapt `freq` — accept a column selector (name or index),
   headerless mode, plus the original `print_with_plot` / `print_without_plot`.
3. **`utils.rs`**: copy `input_reader` + `detect_delimiter` (with their unit tests),
   drop `csv_writer` (unused).
4. **Tests**: port `freq_integration_test.rs` to the new CLI, add `--no-header`
   cases, keep delimiter detection unit tests. All run with `cargo test`.
5. **`ci.yml`**: strip the Python/PyPI steps; keep fmt/clippy/build/test on a
   3-OS matrix.
6. **`release.yml`**: strip PyPI jobs; keep the build matrix + `create-release`
   (softprops/action-gh-release). Linux target `x86_64-unknown-linux-musl`
   produces a static binary that runs on **RHEL9** (glibc-independent) and any
   other Linux. Also build `aarch64-unknown-linux-musl` for parity with `clw`.
7. **Release trigger**: pushing tag `v0.1.0` runs the workflow; binaries land as
   GitHub Release assets (`plhistr-linux-x86_64`, `plhistr-macos-x86_64`,
   `plhistr-macos-aarch64`, `plhistr-windows-x86_64.exe`, plus aarch64 Linux).

## Steps to execute

1. Write `plan.md` (this file), review it against requirements.
2. Scaffold repo files (Cargo.toml, LICENSE, README, .gitignore, workflows).
3. Implement `src/utils.rs`, `src/histogram.rs`, `src/main.rs`.
4. Port tests + fixtures.
5. `cargo fmt`, `cargo clippy`, `cargo test` locally until green.
6. `git init`, commit, add remote `git@github.com:sshu2017/plhistr.git`.
7. Push `main`, then tag `v0.1.0` and push the tag to trigger the release.

## Risks / notes

- `clap`/`colored`/`indicatif` are pure Rust → musl cross-build works (no OpenSSL).
- GitHub-hosted runners have no RHEL9 image; the musl static binary is the
  standard way to target RHEL9-family distros and is documented in the README.
- If the aarch64 musl `cross` job proves flaky, x86_64 musl still covers
  "Linux (RHEL9)"; macOS x86_64 + aarch64 and Windows x86_64 cover the rest.
