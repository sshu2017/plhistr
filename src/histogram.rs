use crate::utils::{detect_delimiter, input_reader};
use colored::Colorize;
use csv::ReaderBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::error::Error;

/// Build and print a histogram for a single column of a delimited input.
///
/// * `column` — the column to histogram. With a header row this is a column
///   *name* (defaulting to the first column); with `no_header` it is a
///   1-based column *index* (defaulting to `1`).
/// * `plot` — draw a horizontal bar plot; otherwise print a plain table.
/// * `sort_index` — sort by value (numerically/alphabetically) instead of by
///   frequency (the default).
pub fn histogram(
    path: Option<&str>,
    column: Option<&str>,
    no_header: bool,
    plot: bool,
    sort_index: bool,
) -> Result<(), Box<dyn Error>> {
    let mut reader = input_reader(path);
    let delimiter = detect_delimiter(&mut *reader)?;

    let mut csv = ReaderBuilder::new()
        .has_headers(!no_header)
        .delimiter(delimiter as u8)
        .from_reader(reader);

    // Resolve the column index (0-based) to count.
    let col_idx = if no_header {
        match column {
            Some(s) => s
                .trim()
                .parse::<usize>()
                .ok()
                .filter(|&i| i >= 1)
                .map(|i| i - 1)
                .ok_or_else(|| {
                    format!("Invalid column index '{}' (expected a 1-based index)", s)
                })?,
            None => 0,
        }
    } else {
        let headers = csv.headers()?.clone();
        match column {
            Some(name) => headers.iter().position(|h| h == name).ok_or_else(|| {
                let available: Vec<&str> = headers.iter().collect();
                format!(
                    "Column '{}' not found in input.\nAvailable columns: {}",
                    name,
                    available.join(", ")
                )
            })?,
            None => 0,
        }
    };

    // Count frequencies with a progress indicator (only shows on a TTY stderr).
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut row_count = 0;

    let spinner = if atty::is(atty::Stream::Stderr) {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        pb.set_message("Reading data ...");
        Some(pb)
    } else {
        None
    };

    for result in csv.records() {
        let record = result?;
        if let Some(val) = record.get(col_idx) {
            *counts.entry(val.to_string()).or_insert(0) += 1;
        }
        row_count += 1;

        if let Some(ref pb) = spinner {
            if row_count % 1000 == 0 {
                pb.set_message(format!("Counting ... {} rows", row_count));
                pb.tick();
            }
        }
    }

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    // Convert to a vector for sorting.
    let mut counts_vec: Vec<(String, usize)> = counts.into_iter().collect();

    if sort_index {
        // Try to sort numerically if all values are numeric, otherwise
        // alphabetically.
        let all_numeric = counts_vec
            .iter()
            .all(|(val, _)| val.trim().parse::<f64>().is_ok());

        if all_numeric {
            counts_vec.sort_by(|a, b| {
                let a_num = a.0.trim().parse::<f64>().unwrap();
                let b_num = b.0.trim().parse::<f64>().unwrap();
                a_num.partial_cmp(&b_num).unwrap()
            });
        } else {
            counts_vec.sort_by(|a, b| a.0.cmp(&b.0));
        }
    } else {
        // Default: sort by frequency (high to low).
        counts_vec.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
    }

    if plot {
        print_with_plot(&counts_vec);
    } else {
        print_without_plot(&counts_vec);
    }

    Ok(())
}

fn print_without_plot(counts: &[(String, usize)]) {
    if counts.is_empty() {
        return;
    }

    // Find max value length for alignment
    let max_val_len = counts.iter().map(|(val, _)| val.len()).max().unwrap_or(0);

    // Find max count length for alignment
    let max_count_len = counts
        .iter()
        .map(|(_, count)| count.to_string().len())
        .max()
        .unwrap_or(0);

    // Calculate total for percentages
    let total: usize = counts.iter().map(|(_, count)| *count).sum();

    // Print header
    println!(
        "{:<val_width$}  {:>count_width$}  {:>7}  {:>11}",
        "Value".green().bold(),
        "Count".green().bold(),
        "Pct".green().bold(),
        "CumPct".green().bold(),
        val_width = max_val_len.max(5),
        count_width = max_count_len.max(5)
    );

    // Track cumulative percentage
    let mut cumulative = 0;

    for (val, count) in counts {
        // Calculate percentages
        let percentage = (*count as f64 / total as f64) * 100.0;
        cumulative += count;
        let cumulative_pct = (cumulative as f64 / total as f64) * 100.0;

        println!(
            "{:<val_width$}  {:>count_width$}  {:>7.2}%  {:>10.2}%",
            val,
            count,
            percentage,
            cumulative_pct,
            val_width = max_val_len.max(5),
            count_width = max_count_len.max(5)
        );
    }
}

fn print_with_plot(counts: &[(String, usize)]) {
    if counts.is_empty() {
        return;
    }

    // Find max count for scaling
    let max_count = counts.iter().map(|(_, count)| *count).max().unwrap_or(1);

    // Find max value length for alignment
    let max_val_len = counts.iter().map(|(val, _)| val.len()).max().unwrap_or(0);

    // Find max count length for alignment
    let max_count_len = counts
        .iter()
        .map(|(_, count)| count.to_string().len())
        .max()
        .unwrap_or(0);

    // Calculate total for percentages
    let total: usize = counts.iter().map(|(_, count)| *count).sum();

    // Bar width: scale to max 50 characters (not including the | borders)
    let max_bar_width = 50;

    // Print header
    let bar_header = format!("|{:^50}|", "");
    println!(
        "{:<val_width$}  {}  {:>count_width$}  {:>9}  {:>15}",
        "Value".green().bold(),
        bar_header,
        "Count".green().bold(),
        "Pct".green().bold(),
        "CumPct".green().bold(),
        val_width = max_val_len,
        count_width = max_count_len
    );

    // Track cumulative percentage
    let mut cumulative = 0;

    for (val, count) in counts {
        let bar_width = if max_count > 0 {
            ((*count as f64 / max_count as f64) * max_bar_width as f64).round() as usize
        } else {
            0
        };

        // Create the bar with padding to ensure consistent width
        let bar_chars = "▪".repeat(bar_width);
        let padding = " ".repeat(max_bar_width - bar_width);
        let bar = format!("|{}{}|", bar_chars, padding);

        // Calculate percentages
        let percentage = (*count as f64 / total as f64) * 100.0;
        cumulative += count;
        let cumulative_pct = (cumulative as f64 / total as f64) * 100.0;

        println!(
            "{:<val_width$}  {}  {:>count_width$}  {:>15.2}%  {:>12.2}%",
            val,
            bar,
            count,
            percentage,
            cumulative_pct,
            val_width = max_val_len,
            count_width = max_count_len
        );
    }
}
