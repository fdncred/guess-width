mod guess_width;

use guess_width::GuessWidth;
use std::io::{BufRead, BufReader, Cursor};

fn main() {
    let input = read_lines_into_string();

    let cursor = Cursor::new(input);
    let reader = BufReader::new(cursor);
    let mut gw = GuessWidth::new_reader(Box::new(reader));
    let rows = gw.read_all();

    let csv_data: Vec<String> = rows
        .iter()
        .map(|inner_vec| {
            inner_vec
                .iter()
                .map(|s| format!("\"{}\"", s)) // Wrap each element in quotes
                .collect::<Vec<String>>()
                .join(",") // Join elements with comma
        })
        .collect();

    for line in csv_data {
        println!("{}", line);
    }
}

fn read_lines_into_string() -> String {
    let mut lines = String::new();
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.expect("Failed to read line");
        lines.push_str(&line);
        lines.push('\n'); // Add newline character to separate lines
    }
    lines
}
