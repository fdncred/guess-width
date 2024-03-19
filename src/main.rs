mod guesswidth;

use guesswidth::GuessWidth;

fn main() {
    let input = r"   PID TTY          TIME CMD
302965 pts/3    00:00:11 zsh
709737 pts/3    00:00:00 ps";
    let reader = std::io::BufReader::new(input.as_bytes());
    let mut gw = GuessWidth::new(reader);
    let rows = gw.read_all();
    for row in rows {
        println!("{:?}", row);
    }
}
