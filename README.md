# plhistr

Plot a **histogram** of a CSV column directly in the terminal.

`plhistr` reads tabular data from **stdin** (or a file), counts the frequency of
each unique value in a column, and prints a frequency table with a horizontal
bar plot.

```
$ printf 'city\nNew York\nLA\nNew York\nChicago\nNew York\nLA\n' | plhistr -c city
Value       |                                                  |  Count        Pct         CumPct
New York    |██████████████████████████████████████████████████|      3       50.00%        50.00%
LA          |█████████████████████████████████████████         |      2       33.33%        83.33%
Chicago     |█████████████████                                 |      1       16.67%       100.00%
```

## Usage

```
plhistr [OPTIONS] [FILE]

Arguments:
  [FILE]  Input file. Reads from stdin when omitted or set to "-".

Options:
  -c, --column <COLUMN>  Column to histogram. With a header row: the column
                         NAME (default: first column). With --no-header: a
                         1-based column INDEX (default: 1).
      --no-header        Treat the first row as data (no header row).
      --sort-index       Sort by value (numerically/alphabetically) instead of
                         by frequency (the default).
      --no-plot          Print the frequency table without the bar plot.
  -h, --help             Print help
  -V, --version          Print version
```

The delimiter is auto-detected: comma (`,`), pipe (`|`), tab, or space.

### Examples

```sh
# Histogram the first column of a file
plhistr data.csv

# Histogram a specific column from stdin
cat data.csv | plhistr -c occupation

# Sort bars by value instead of by frequency
plhistr -c age --sort-index data.csv

# Histogram headerless input (e.g. from a pipeline)
seq 1 100 | plhistr --no-header

# Plain table, no bars
plhistr --no-plot -c city data.csv
```

## Installation

Prebuilt binaries for **Linux (x86_64/aarch64, musl-static)**, **macOS
(x86_64/Apple Silicon)**, and **Windows (x86_64)** are attached to every
[GitHub Release](https://github.com/sshu2017/plhistr/releases).

The Linux binary is statically linked against musl, so it runs on **RHEL 9**
(and RHEL 7/8, Ubuntu, Alpine, etc.) without any glibc version requirements.

Or build from source:

```sh
cargo install --path .
```

## Development

```sh
cargo build          # build
cargo test           # run unit + integration tests
cargo fmt --check    # formatting
cargo clippy         # lints
```

## License

MIT
