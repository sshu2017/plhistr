use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_histogram_default_first_column_file() {
    // No column given: histogram the first column (name) of the file.
    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("tests/fixtures/sample_comma.csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"));
}

#[test]
fn test_histogram_column_selection() {
    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("--column")
        .arg("city")
        .arg("tests/fixtures/sample_comma.csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("New York"))
        .stdout(predicate::str::contains("Los Angeles"))
        .stdout(predicate::str::contains("Chicago"));
}

#[test]
fn test_histogram_with_duplicates_via_stdin() {
    let temp_csv = "city\nNew York\nLos Angeles\nNew York\nChicago\nNew York\nLos Angeles\n";

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("city")
        .write_stdin(temp_csv)
        .assert()
        .success();

    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // New York 3, Los Angeles 2, Chicago 1. Default sort is by frequency.
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines[1].contains("New York") && lines[1].contains("3"),
        "First data line should be New York with 3"
    );
    assert!(
        lines[2].contains("Los Angeles") && lines[2].contains("2"),
        "Second data line should be Los Angeles with 2"
    );
    assert!(
        lines[3].contains("Chicago") && lines[3].contains("1"),
        "Third data line should be Chicago with 1"
    );
}

#[test]
fn test_histogram_plot_is_default() {
    let temp_csv = "city\nNew York\nLos Angeles\nNew York\nChicago\nNew York\n";

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("city")
        .write_stdin(temp_csv)
        .assert()
        .success()
        .stdout(predicate::str::contains("▪")); // bars are drawn by default
}

#[test]
fn test_histogram_no_plot() {
    let temp_csv = "city\nNew York\nLos Angeles\nNew York\n";

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("city")
        .arg("--no-plot")
        .write_stdin(temp_csv)
        .assert()
        .success()
        .stdout(predicate::str::contains("Value"))
        .stdout(predicate::str::contains("Count"))
        .stdout(predicate::str::contains("New York"));

    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains('▪'),
        "Table output should not contain bar characters"
    );
}

#[test]
fn test_histogram_sort_index_alphabetic() {
    let temp_csv = "name\nCharlie\nAlice\nBob\nAlice\nCharlie\nCharlie\n";

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("name")
        .arg("--sort-index")
        .write_stdin(temp_csv)
        .assert()
        .success();

    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines[1].contains("Alice"),
        "First data line should be Alice"
    );
    assert!(lines[2].contains("Bob"), "Second data line should be Bob");
    assert!(
        lines[3].contains("Charlie"),
        "Third data line should be Charlie"
    );
}

#[test]
fn test_histogram_sort_index_numeric() {
    let temp_csv = "value\n30\n10\n25\n30\n10\n10\n";

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("value")
        .arg("--sort-index")
        .write_stdin(temp_csv)
        .assert()
        .success();

    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Sorted numerically: 10, 25, 30 (not lexicographically).
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines[1].starts_with("10"), "First data line should be 10");
    assert!(lines[2].starts_with("25"), "Second data line should be 25");
    assert!(lines[3].starts_with("30"), "Third data line should be 30");
}

#[test]
fn test_histogram_pipe_delimited() {
    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("category")
        .arg("tests/fixtures/sample_pipe.csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tools"))
        .stdout(predicate::str::contains("Electronics"))
        .stdout(predicate::str::contains("Home"));
}

#[test]
fn test_histogram_invalid_column() {
    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("invalid_column")
        .arg("tests/fixtures/sample_comma.csv")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Column 'invalid_column' not found",
        ))
        .stderr(predicate::str::contains(
            "Available columns: name, age, city, occupation",
        ));
}

#[test]
fn test_histogram_no_header_default_column() {
    let temp_csv = "5\n3\n5\n1\n5\n";

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("--no-header")
        .write_stdin(temp_csv)
        .assert()
        .success()
        .stdout(predicate::str::contains("5"))
        .stdout(predicate::str::contains("3"));

    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // 5 appears 3 times and is the most frequent -> first data line.
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines[1].contains("5") && lines[1].contains("3"),
        "First data line should be 5 with count 3"
    );
}

#[test]
fn test_histogram_no_header_column_index() {
    let temp_csv = "30,9\n10,9\n30,8\n";

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("--no-header")
        .arg("-c")
        .arg("2")
        .write_stdin(temp_csv)
        .assert()
        .success();

    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Second column: 9, 9, 8 -> 9 twice, 8 once.
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines[1].contains("9") && lines[1].contains("2"),
        "First data line should be 9 with count 2"
    );
    assert!(
        lines[2].contains("8") && lines[2].contains("1"),
        "Second data line should be 8 with count 1"
    );
}

#[test]
fn test_histogram_no_header_invalid_index() {
    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("--no-header")
        .arg("-c")
        .arg("0")
        .write_stdin("1\n2\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid column index"));
}

#[test]
fn test_histogram_single_value() {
    let temp_csv = "value\n42\n";

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("value")
        .write_stdin(temp_csv)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"))
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_histogram_all_same_value() {
    let temp_csv = "name\nAlice\nAlice\nAlice\n";

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("name")
        .write_stdin(temp_csv)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_histogram_empty_values_counted() {
    let temp_csv = "name,value\nAlice,A\nBob,\nCharlie,B\nDave,A\nEve,\nFrank,A\n";

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("value")
        .write_stdin(temp_csv)
        .assert()
        .success();

    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // A appears 3 times, empty appears 2 times, B appears 1 time.
    assert!(
        stdout.contains("A") && stdout.contains("3"),
        "A should appear 3 times"
    );
    assert!(
        stdout.contains("B") && stdout.contains("1"),
        "B should appear 1 time"
    );

    // Empty values are counted too (1 header + 3 unique values).
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        4,
        "Should have 3 unique values including empty plus header"
    );
}

#[test]
fn test_histogram_empty_file() {
    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("tests/fixtures/empty.csv")
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn test_histogram_piped_input() {
    let csv_content = fs::read("tests/fixtures/sample_comma.csv").unwrap();

    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("-c")
        .arg("occupation")
        .write_stdin(csv_content)
        .assert()
        .success()
        .stdout(predicate::str::contains("Engineer"))
        .stdout(predicate::str::contains("Designer"))
        .stdout(predicate::str::contains("Manager"));
}

#[test]
fn test_histogram_version() {
    let mut cmd = cargo_bin_cmd!("plhistr");
    cmd.arg("--version").assert().success();
}
