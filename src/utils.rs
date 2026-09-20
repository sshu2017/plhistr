use atty::{is, Stream};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

/// Open an input source: a file path, or stdin when `path` is `None` (or "-").
///
/// Panics when stdin is a TTY and no file was given, because there is then no
/// data to read.
pub fn input_reader(path: Option<&str>) -> Box<dyn BufRead> {
    match path {
        Some(p) if p != "-" => Box::new(BufReader::new(File::open(p).expect("Cannot open file"))),
        _ => {
            if is(Stream::Stdin) {
                panic!("Expected a file or piped input, but stdin is a TTY");
            }
            Box::new(BufReader::new(io::stdin()))
        }
    }
}

/// Detect the delimiter of a delimited-text input by counting occurrences of
/// common delimiters on the first line (tab > pipe > comma > space priority on
/// ties). Defaults to comma when no delimiter is found.
pub fn detect_delimiter(reader: &mut dyn BufRead) -> Result<char, Box<dyn Error>> {
    // Peek at buffer without consuming it
    let buffer = reader.fill_buf()?;

    if buffer.is_empty() {
        return Err("File is empty or contains no data".into());
    }

    // Find first newline position
    let first_line_end = buffer
        .iter()
        .position(|&b| b == b'\n')
        .unwrap_or(buffer.len());

    // Convert bytes to string (still just viewing, not consuming)
    let first_line =
        std::str::from_utf8(&buffer[..first_line_end]).map_err(|_| "Invalid UTF-8 in file")?;

    if first_line.trim().is_empty() {
        return Err("File is empty or contains no data".into());
    }

    // Detect delimiter by counting occurrences of common delimiters
    let delimiters = [('\t', "tab"), ('|', "pipe"), (',', "comma"), (' ', "space")];
    let mut counts: Vec<(char, usize, &str)> = delimiters
        .iter()
        .map(|&(delim, name)| (delim, first_line.matches(delim).count(), name))
        .collect();

    // Sort by count (descending), with tie-breaking by delimiter priority
    counts.sort_by(|a, b| {
        match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => {
                // If counts are equal, prioritize: tab > pipe > comma > space
                let priority = |c: char| match c {
                    '\t' => 0,
                    '|' => 1,
                    ',' => 2,
                    ' ' => 3,
                    _ => 4,
                };
                priority(a.0).cmp(&priority(b.0))
            }
            other => other,
        }
    });

    // Return the delimiter with the highest count
    // If no delimiter found (all counts are 0), default to comma
    let delimiter = if counts[0].1 > 0 { counts[0].0 } else { ',' };

    Ok(delimiter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_detect_delimiter_comma() {
        let data = "name,age,city\nAlice,30,NYC\n";
        let mut reader = Cursor::new(data);
        let delimiter = detect_delimiter(&mut reader).unwrap();
        assert_eq!(delimiter, ',');
    }

    #[test]
    fn test_detect_delimiter_pipe() {
        let data = "name|age|city\nBob|25|LA\n";
        let mut reader = Cursor::new(data);
        let delimiter = detect_delimiter(&mut reader).unwrap();
        assert_eq!(delimiter, '|');
    }

    #[test]
    fn test_detect_delimiter_space() {
        let data = "name age city\nCharlie 35 Boston\n";
        let mut reader = Cursor::new(data);
        let delimiter = detect_delimiter(&mut reader).unwrap();
        assert_eq!(delimiter, ' ', "Should detect space delimiter");
    }

    #[test]
    fn test_detect_delimiter_tab() {
        let data = "name\tage\tcity\nAlice\t30\tNYC\n";
        let mut reader = Cursor::new(data);
        let delimiter = detect_delimiter(&mut reader).unwrap();
        assert_eq!(delimiter, '\t', "Should detect tab delimiter");
    }

    #[test]
    fn test_detect_delimiter_defaults_to_comma() {
        let data = "name\nAlice\n";
        let mut reader = Cursor::new(data);
        let delimiter = detect_delimiter(&mut reader).unwrap();
        assert_eq!(
            delimiter, ',',
            "Should default to comma when no delimiter found"
        );
    }

    #[test]
    fn test_detect_delimiter_empty_file() {
        let data = "";
        let mut reader = Cursor::new(data);
        let result = detect_delimiter(&mut reader);
        assert!(result.is_err(), "Should return error for empty file");
        assert_eq!(
            result.unwrap_err().to_string(),
            "File is empty or contains no data"
        );
    }

    #[test]
    fn test_detect_delimiter_only_whitespace() {
        let data = "   \n  \n";
        let mut reader = Cursor::new(data);
        let result = detect_delimiter(&mut reader);
        assert!(
            result.is_err(),
            "Should return error for whitespace-only file"
        );
    }
}
