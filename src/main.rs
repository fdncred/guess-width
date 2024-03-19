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
    fn test_guess_width_ps() {
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
            trim_space: false,
        };

        #[rustfmt::skip]
        let want = vec![
            vec!["   PID", " TTY     ", "     TIME", "CMD"],
            vec!["302965", " pts/3   ", " 00:00:11", "zsh"],
            vec!["709737", " pts/3   ", " 00:00:00", "ps"],
        ];
        let got = guess_width.read_all();
        assert_eq!(got, want);
    }

    #[test]
    fn test_guess_width_ps_trim() {
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

        #[rustfmt::skip]
        let want = vec![
            vec!["PID", "TTY", "TIME", "CMD"],
            vec!["302965", "pts/3", "00:00:11", "zsh"],
            vec!["709737", "pts/3", "00:00:00", "ps"],
        ];
        let got = guess_width.read_all();
        assert_eq!(got, want);
    }

    #[test]
    fn test_guess_width_ps_overflow() {
        let input = "USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
root           1  0.0  0.0 168576 13788 ?        Ss   Mar11   0:49 /sbin/init splash
noborus   703052  2.1  0.7 1184814400 230920 ?   Sl   10:03   0:45 /opt/google/chrome/chrome
noborus   721971  0.0  0.0  13716  3524 pts/3    R+   10:39   0:00 ps aux";

        let r = Box::new(std::io::BufReader::new(input.as_bytes())) as Box<dyn std::io::Read>;
        let reader = std::io::BufReader::new(r);

        let mut guess_width = GuessWidth {
            reader,
            pos: Vec::new(),
            pre_lines: Vec::new(),
            pre_count: 0,
            scan_num: 100,
            header: 0,
            limit_split: 0,
            min_lines: 2,
            trim_space: false,
        };

        #[rustfmt::skip]
        let want = vec![
            vec!["USER     ", "    PID", " %CPU", " %MEM", "    VSZ", "   RSS", " TTY     ", " STAT", " START  ", " TIME", "COMMAND"],
            vec!["root     ", "      1", "  0.0", "  0.0", " 168576", " 13788", " ?       ", " Ss  ", " Mar11  ", " 0:49", "/sbin/init splash"],
            vec!["noborus  ", " 703052", "  2.1", "  0.7", " 1184814400", " 230920", " ?  ", " Sl  ", " 10:03  ", " 0:45", "/opt/google/chrome/chrome"],
            vec!["noborus  ", " 721971", "  0.0", "  0.0", "  13716", "  3524", " pts/3   ", " R+  ", " 10:39  ", " 0:00", "ps aux"],
        ];
        let got = guess_width.read_all();
        assert_eq!(got, want);
    }

    #[test]
    fn test_guess_width_ps_overflow_trim() {
        let input = "USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
root           1  0.0  0.0 168576 13788 ?        Ss   Mar11   0:49 /sbin/init splash
noborus   703052  2.1  0.7 1184814400 230920 ?   Sl   10:03   0:45 /opt/google/chrome/chrome
noborus   721971  0.0  0.0  13716  3524 pts/3    R+   10:39   0:00 ps aux";

        let r = Box::new(std::io::BufReader::new(input.as_bytes())) as Box<dyn std::io::Read>;
        let reader = std::io::BufReader::new(r);

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

        #[rustfmt::skip]
        let want = vec![
            vec!["USER", "PID", "%CPU", "%MEM", "VSZ", "RSS", "TTY", "STAT", "START", "TIME", "COMMAND"],
            vec!["root", "1", "0.0", "0.0", "168576", "13788", "?", "Ss", "Mar11", "0:49", "/sbin/init splash"],
            vec!["noborus", "703052", "2.1", "0.7", "1184814400", "230920", "?", "Sl", "10:03", "0:45", "/opt/google/chrome/chrome"],
            vec!["noborus", "721971", "0.0", "0.0", "13716", "3524", "pts/3", "R+", "10:39", "0:00", "ps aux"],
        ];
        let got = guess_width.read_all();
        assert_eq!(got, want);
    }

    #[test]
    fn test_guess_width_ps_limit() {
        let input = "   PID TTY          TIME CMD
302965 pts/3    00:00:11 zsh
709737 pts/3    00:00:00 ps";

        let r = Box::new(std::io::BufReader::new(input.as_bytes())) as Box<dyn std::io::Read>;
        let reader = std::io::BufReader::new(r);

        let mut guess_width = GuessWidth {
            reader,
            pos: Vec::new(),
            pre_lines: Vec::new(),
            pre_count: 0,
            scan_num: 100,
            header: 0,
            limit_split: 2,
            min_lines: 2,
            trim_space: false,
        };

        #[rustfmt::skip]
        let want = vec![
            vec!["   PID", " TTY     ", "    TIME CMD"],
            vec!["302965", " pts/3   ", "00:00:11 zsh"],
            vec!["709737", " pts/3   ", "00:00:00 ps"],
        ];
        let got = guess_width.read_all();
        assert_eq!(got, want);
    }

    #[test]
    fn test_guess_width_ps_limit_trim() {
        let input = "   PID TTY          TIME CMD
302965 pts/3    00:00:11 zsh
709737 pts/3    00:00:00 ps";

        let r = Box::new(std::io::BufReader::new(input.as_bytes())) as Box<dyn std::io::Read>;
        let reader = std::io::BufReader::new(r);

        let mut guess_width = GuessWidth {
            reader,
            pos: Vec::new(),
            pre_lines: Vec::new(),
            pre_count: 0,
            scan_num: 100,
            header: 0,
            limit_split: 2,
            min_lines: 2,
            trim_space: true,
        };

        #[rustfmt::skip]
        let want = vec![
            vec!["PID", "TTY", "TIME CMD"],
            vec!["302965", "pts/3", "00:00:11 zsh"],
            vec!["709737", "pts/3", "00:00:00 ps"],
        ];
        let got = guess_width.read_all();
        assert_eq!(got, want);
    }
}
