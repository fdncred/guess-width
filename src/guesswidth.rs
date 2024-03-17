use std::io::{self, BufRead};

pub struct GuessWidth<R: BufRead> {
    reader: R,
    pos: Vec<usize>,
    pre_lines: Vec<String>,
    pre_count: usize,
    scan_num: usize,
    header: usize,
    limit_split: usize,
    min_lines: usize,
    trim_space: bool,
}

impl<R: BufRead> GuessWidth<R> {
    pub fn new(reader: R) -> GuessWidth<R> {
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

    pub fn scan(&mut self, num: usize) {
        for _ in 0..num {
            let mut buf = String::new();
            if self.reader.read_line(&mut buf).is_err() {
                break;
            }
            self.pre_lines.push(buf);
        }
        self.pos = positions(&self.pre_lines, self.header, self.min_lines);
        if self.limit_split > 0 && self.pos.len() > self.limit_split {
            self.pos.truncate(self.limit_split);
        }
    }

    pub fn read(&mut self) -> Result<Vec<String>, io::Error> {
        if self.pre_lines.is_empty() {
            self.scan(self.scan_num);
        }
        let line = if self.pre_count < self.pre_lines.len() {
            let line = self.pre_lines[self.pre_count].clone();
            self.pre_count += 1;
            line
        } else {
            let mut buf = String::new();
            if self.reader.read_line(&mut buf).is_err() {
                return Err(io::Error::new(io::ErrorKind::Other, "EOF"));
            }
            buf
        };
        Ok(split(&line, &self.pos, self.trim_space))
    }
}

pub fn to_table(lines: &[String], header: usize, trim_space: bool) -> Vec<Vec<String>> {
    let pos = positions(lines, header, 2);
    to_rows(lines, &pos, trim_space)
}

pub fn to_table_n(
    lines: &[String],
    header: usize,
    num_split: usize,
    trim_space: bool,
) -> Vec<Vec<String>> {
    let mut pos = positions(lines, header, 2);
    if pos.len() > num_split {
        pos.truncate(num_split);
    }
    to_rows(lines, &pos, trim_space)
}

fn positions(lines: &[String], header: usize, min_lines: usize) -> Vec<usize> {
    let mut blanks = Vec::new();
    let header = if header < 0 { 0 } else { header };
    for (n, line) in lines.iter().enumerate() {
        if n < header {
            continue;
        }
        if n == header {
            blanks = lookup_blanks(line.trim_end_matches(' '));
            continue;
        }
        blanks = count_blanks(&blanks, line.trim_end_matches(' '));
    }
    positions(&blanks, min_lines)
}

fn separator_position(lr: &[char], p: usize, pos: &[usize], n: usize) -> usize {
    if lr[p].is_whitespace() {
        return p;
    }
    let mut f = p;
    let mut fp = 0;
    while f < lr.len() && !lr[f].is_whitespace() {
        f += 1;
        fp += 1;
    }
    let mut b = p;
    let mut bp = 0;
    while b > 0 && !lr[b].is_whitespace() {
        b -= 1;
        bp += 1;
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
    }
    0
}

fn split(line: &str, pos: &[usize], trim_space: bool) -> Vec<String> {
    let mut result = Vec::new();
    let chars: Vec<char> = if trim_space {
        line.trim().chars().collect()
    } else {
        line.chars().collect()
    };
    let mut start = 0;
    for (i, &p) in pos.iter().enumerate() {
        let end = separator_position(&chars, p, pos, i);
        result.push(chars[start..end].iter().collect());
        start = end + 1;
    }
    result.push(chars[start..].iter().collect());
    result
}

fn to_rows(lines: &[String], pos: &[usize], trim_space: bool) -> Vec<Vec<String>> {
    lines
        .iter()
        .map(|line| split(line, pos, trim_space))
        .collect()
}

fn lookup_blanks(line: &str) -> Vec<usize> {
    let mut blanks = Vec::new();
    let mut iter = line.chars().enumerate().peekable();
    while let Some((i, c)) = iter.next() {
        if c.is_whitespace() {
            blanks.push(i);
            while let Some((_, c)) = iter.peek() {
                if !c.is_whitespace() {
                    break;
                }
                iter.next();
            }
        }
    }
    blanks
}

fn count_blanks(blanks: &[usize], line: &str) -> Vec<usize> {
    let mut new_blanks = Vec::new();
    let mut iter = line.chars().enumerate().peekable();
    for &blank in blanks {
        while let Some((i, c)) = iter.next() {
            if i == blank {
                new_blanks.push(blank);
                while let Some((_, c)) = iter.peek() {
                    if !c.is_whitespace() {
                        break;
                    }
                    iter.next();
                }
                break;
            }
        }
    }
    new_blanks
}

fn positions(blanks: &[usize], min_lines: usize) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut start = 0;
    let mut end = 0;
    for &blank in blanks {
        if blank - end >= min_lines {
            positions.push(start);
            start = blank + 1;
        }
        end = blank;
    }
    positions.push(start);
    positions
}
