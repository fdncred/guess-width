mod guesswidth;

use guesswidth::GuessWidth;

fn main() {
    let input = r"   PID TTY          TIME CMD
302965 pts/3    00:00:11 zsh
709737 pts/3    00:00:00 ps";
    let reader = Box::new(std::io::BufReader::new(input.as_bytes()));
    let mut gw = GuessWidth::new_reader(reader);
    let rows = gw.read_all();
    for row in rows {
        println!("{:?}", row);
    }
}

#[cfg(test)]
mod tests {
    use crate::guesswidth::GuessWidth;

    #[test]
    fn test_guess_width_read_all() {
        let input = "   PID TTY          TIME CMD
302965 pts/3    00:00:11 zsh
709737 pts/3    00:00:00 ps";

        let r = Box::new(std::io::BufReader::new(input.as_bytes())) as Box<dyn std::io::Read>;
        let reader = std::io::BufReader::new(r);

        // let reader = Cursor::new(input);
        let mut guess_width = GuessWidth {
            reader,
            pos: Vec::new(),
            pre_lines: Vec::new(),
            pre_count: 0,
            scan_num: 100,
            header: 0,
            limit_split: 0,
            min_lines: 2,
            trim_space: true,
        };
        let want = vec![
            vec!["PID", "TTY", "TIME", "CMD"],
            vec!["302965", "pts/3", "00:00:11", "zsh"],
            vec!["709737", "pts/3", "00:00:00", "ps"],
        ];
        let got = guess_width.read_all();
        assert_eq!(got, want);
    }
}
