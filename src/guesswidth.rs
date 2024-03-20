use std::io::{self, BufRead};
use unicode_width::UnicodeWidthChar;

pub struct GuessWidth {
    pub reader: io::BufReader<Box<dyn io::Read>>,
    pub pos: Vec<usize>,
    pub pre_lines: Vec<String>,
    pub pre_count: usize,
    pub scan_num: usize,
    pub header: usize,
    pub limit_split: usize,
    pub min_lines: usize,
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

        w += match UnicodeWidthChar::width(lr[p]) {
            Some(w) => w,
            None => 0,
        };
    }

    if n <= columns.len() {
        let col = &line[start..];
        let col = if trim_space { col.trim() } else { col };
        columns.push(col.to_string());
    }

    columns
}

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
        match UnicodeWidthChar::width(c) {
            Some(w) => {
                if w == 2 {
                    blanks.push(0)
                }
            }
            None => {}
        };
    }

    blanks
}

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
        match UnicodeWidthChar::width(c) {
            Some(w) => {
                if w == 2 {
                    n += 1;
                }
            }
            None => {}
        };
    }

    blanks.to_vec()
}

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
