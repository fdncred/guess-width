// Attribution: https://github.com/noborus/guesswidth/blob/main/guesswidth.go
// The MIT License (MIT) as of 2024-03-22
//
// GuessWidth handles the format as formatted by printf.
// Spaces exist as delimiters, but spaces are not always delimiters.
// The width seems to be a fixed length, but it doesn't always fit.
// GuessWidth finds the column separation position
// from the reference line(header) and multiple lines(body).

// Briefly, the algorithm uses a histogram of spaces to find widths.
// blanks, lines, and pos are variables used in the algorithm. The other
// items names below are just for reference.
// blanks =  0000003000113333111100003000
//  lines = "   PID TTY          TIME CMD"
//          "302965 pts/3    00:00:11 zsh"
//          "709737 pts/3    00:00:00 ps"
//
// measure= "012345678901234567890123456789"
// spaces = "      ^        ^        ^"
//    pos =  6 15 24 <- the carets show these positions
// the items in pos map to 3's in the blanks array

// Now that we have pos, we can let split() use this pos array to figure out
// how to split all lines by comparing each index to see if there's a space.
// So, it looks at position 6, 15, 24 and sees if it has a space in those
// positions. If it does, it splits the line there. If it doesn't, it wiggles
// around the position to find the next space and splits there.
use std::io::{self, BufRead};
use unicode_width::UnicodeWidthStr;

// GuessWidth reads records from printf-like output.
pub struct GuessWidth {
    pub reader: io::BufReader<Box<dyn io::Read>>,
    // pos is a list of separator positions.
    pub pos: Vec<usize>,
    // pre_lines stores the lines read for scan.
    pub pre_lines: Vec<String>,
    // pre_count is the number returned by read.
    pub pre_count: usize,
    // scan_num is the number to scan to analyze.
    pub scan_num: usize,
    // header is the base line number. It starts from 0.
    pub header: usize,
    // limit_split is the maximum number of columns to split.
    pub limit_split: usize,
    // min_lines is the minimum number of lines to recognize as a separator.
    // 1 if only the header, 2 or more if there is a blank in the body.
    pub min_lines: usize,
    // trim_space is whether to trim the space in the value.
    pub trim_space: bool,
}

impl GuessWidth {
    pub fn new_reader(r: Box<dyn io::Read>) -> GuessWidth {
        let reader = io::BufReader::new(r);
        GuessWidth {
            reader,
            pos: Vec::new(),
            pre_lines: Vec::new(),
            pre_count: 0,
            scan_num: 100,
            header: 0,
            limit_split: 0,
            min_lines: 2,
            trim_space: true,
        }
    }

    // read_all reads all rows
    // and returns a two-dimensional slice of rows and columns.
    pub fn read_all(&mut self) -> Vec<Vec<String>> {
        if self.pre_lines.is_empty() {
            self.scan(self.scan_num);
        }

        let mut rows = Vec::new();
        loop {
            match self.read() {
                Ok(columns) => rows.push(columns),
                Err(_) => break,
            }
        }

        rows
    }

    // scan preReads and parses the lines.
    fn scan(&mut self, num: usize) {
        for _ in 0..num {
            let mut buf = String::new();
            if self.reader.read_line(&mut buf).unwrap() == 0 {
                break;
            }

            let line = buf.trim_end().to_string();
            self.pre_lines.push(line);
        }

        self.pos = positions(&self.pre_lines, self.header, self.min_lines);
        if self.limit_split > 0 && self.pos.len() > self.limit_split {
            self.pos.truncate(self.limit_split);
        }
    }

    // read reads one row and returns a slice of columns.
    // scan is executed first if it is not preRead.
    fn read(&mut self) -> Result<Vec<String>, io::Error> {
        if self.pre_lines.is_empty() {
            self.scan(self.scan_num);
        }

        let line = if self.pre_count < self.pre_lines.len() {
            let line = self.pre_lines[self.pre_count].clone();
            self.pre_count += 1;
            line
        } else {
            let mut buf = String::new();
            if self.reader.read_line(&mut buf)? == 0 {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "End of file"));
            }

            buf.trim_end().to_string()
        };

        Ok(split(&line, &self.pos, self.trim_space))
    }
}

// positions returns separator positions
// from multiple lines and header line number.
// Lines before the header line are ignored.
fn positions(lines: &[String], header: usize, min_lines: usize) -> Vec<usize> {
    let mut blanks = Vec::new();
    for (n, line) in lines.iter().enumerate() {
        if n < header {
            continue;
        }

        if n == header {
            blanks = lookup_blanks(line.trim_end_matches(' '));
            continue;
        }

        blanks = count_blanks(&mut blanks, line.trim_end_matches(' '));
    }

    positions_helper(&blanks, min_lines)
}

fn separator_position(lr: &[char], p: usize, pos: &[usize], n: usize) -> usize {
    if lr[p].is_whitespace() {
        return p;
    }

    let mut f = p;
    while f < lr.len() && !lr[f].is_whitespace() {
        f += 1;
    }

    let mut b = p;
    while b > 0 && !lr[b].is_whitespace() {
        b -= 1;
    }

    if b == pos[n] {
        return f;
    }

    if n < pos.len() - 1 {
        if f == pos[n + 1] {
            return b;
        }
        if b == pos[n] {
            return f;
        }
        if b > pos[n] && b < pos[n + 1] {
            return b;
        }
    }

    f
}

fn split(line: &str, pos: &[usize], trim_space: bool) -> Vec<String> {
    let mut n = 0;
    let mut start = 0;
    let mut columns = Vec::with_capacity(pos.len() + 1);
    let lr: Vec<char> = line.chars().collect();
    let mut w = 0;

    for p in 0..lr.len() {
        if n > pos.len() - 1 {
            start = p;
            break;
        }

        if pos[n] <= w {
            let end = separator_position(&lr, p, pos, n);
            if start > end {
                break;
            }
            let col = &line[start..end];
            let col = if trim_space { col.trim() } else { col };
            columns.push(col.to_string());
            n += 1;
            start = end;
        }

        w += UnicodeWidthStr::width(lr[p].to_string().as_str());
    }

    if n <= columns.len() {
        let col = &line[start..];
        let col = if trim_space { col.trim() } else { col };
        columns.push(col.to_string());
    }

    columns
}

// Creates a blank(1) and non-blank(0) slice.
// Execute for the base line (header line).
fn lookup_blanks(line: &str) -> Vec<usize> {
    let mut blanks = Vec::new();
    let mut first = true;

    for c in line.chars() {
        if c == ' ' {
            if first {
                blanks.push(0);
                continue;
            }
            blanks.push(1);
            continue;
        }

        first = false;
        blanks.push(0);
        if UnicodeWidthStr::width(c.to_string().as_str()) == 2 {
            blanks.push(0);
        }
    }

    blanks
}

// count up if the line is blank where the reference line was blank.
fn count_blanks(blanks: &mut [usize], line: &str) -> Vec<usize> {
    let mut n = 0;

    for c in line.chars() {
        if n >= blanks.len() {
            break;
        }

        if c == ' ' && blanks[n] > 0 {
            blanks[n] += 1;
        }

        n += 1;
        if UnicodeWidthStr::width(c.to_string().as_str()) == 2 {
            n += 1;
        }
    }

    blanks.to_vec()
}

// Generates a list of separator positions from a blank slice.
fn positions_helper(blanks: &[usize], min_lines: usize) -> Vec<usize> {
    let mut max = min_lines;
    let mut p = 0;
    let mut pos = Vec::new();

    for (n, v) in blanks.iter().enumerate() {
        if *v >= max {
            max = *v;
            p = n;
        }
        if *v == 0 {
            max = min_lines;
            if p > 0 {
                pos.push(p);
                p = 0;
            }
        }
    }
    pos
}

// to_rows returns rows separated by columns.
#[allow(dead_code)]
fn to_rows(lines: Vec<String>, pos: Vec<usize>, trim_space: bool) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(lines.len());
    for line in lines {
        let columns = split(&line, &pos, trim_space);
        rows.push(columns);
    }
    rows
}

// to_table parses a slice of lines and returns a table.
#[allow(dead_code)]
pub fn to_table(lines: Vec<String>, header: usize, trim_space: bool) -> Vec<Vec<String>> {
    let pos = positions(&lines, header, 2);
    to_rows(lines, pos, trim_space)
}

// to_table_n parses a slice of lines and returns a table, but limits the number of splits.
#[allow(dead_code)]
pub fn to_table_n(
    lines: Vec<String>,
    header: usize,
    num_split: usize,
    trim_space: bool,
) -> Vec<Vec<String>> {
    let mut pos = positions(&lines, header, 2);
    if pos.len() > num_split {
        pos.truncate(num_split);
    }
    to_rows(lines, pos, trim_space)
}
