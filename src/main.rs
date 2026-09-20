use clap::Parser;
use std::error::Error;

mod histogram;
mod utils;
use histogram::histogram;

/// Plot a histogram of a column in the terminal.
#[derive(Parser, Debug)]
#[command(name = "plhistr", author, version, about, long_about = None)]
struct Cli {
    /// Input file. Reads from stdin when omitted or set to "-".
    #[arg(value_name = "FILE")]
    file: Option<String>,

    /// Column to histogram. With a header row: the column NAME (default: first
    /// column). With --no-header: a 1-based column INDEX (default: 1).
    #[arg(short, long, value_name = "COLUMN")]
    column: Option<String>,

    /// Treat the first row as data (no header row).
    #[arg(long)]
    no_header: bool,

    /// Sort by value (numerically/alphabetically) instead of by frequency.
    #[arg(long)]
    sort_index: bool,

    /// Print the frequency table without the bar plot.
    #[arg(long)]
    no_plot: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    histogram(
        cli.file.as_deref(),
        cli.column.as_deref(),
        cli.no_header,
        !cli.no_plot,
        cli.sort_index,
    )
}
