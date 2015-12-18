const BUFSIZE: usize = 10;

use std::env;
use std::io::{stdin,BufRead};
use std::mem::drop;
use std::sync::mpsc;
use std::thread;

type Line = [bool];
type LineIn = mpsc::Receiver<Box<Line>>;
type LineOut = mpsc::SyncSender<Box<Line>>;

fn new_line(like: &Line) -> Box<Line> { vec![false; like.len()].into_boxed_slice() }

fn prepend_life(line_out: LineOut, n: u64, stuck: bool) -> LineOut {
    if n == 0 {
        return line_out;
    }
    let (new_out, line_in) = mpsc::sync_channel(BUFSIZE);
    thread::spawn(move || life_stage(line_in, line_out, n - 1, stuck));
    new_out
}

fn life_line(top: &Line, mid: &Line, bot: &Line, stuck: bool) -> Box<Line> {
    debug_assert_eq!(top.len(), mid.len());
    debug_assert_eq!(mid.len(), bot.len());
    let w = mid.len();
    let mut buf = vec![0u8; w];
    for i in 1..w {
        if top[i] { buf[i-1] += 2; }
        if mid[i] { buf[i-1] += 2; }
        if bot[i] { buf[i-1] += 2; }
    }
    for i in 0..w {
        if top[i] { buf[i] += 2; }
        if mid[i] { buf[i] += 1; }
        if bot[i] { buf[i] += 2; }
    }
    for i in 0..w-1 {
        if top[i] { buf[i+1] += 2; }
        if mid[i] { buf[i+1] += 2; }
        if bot[i] { buf[i+1] += 2; }
    }
    let mut out: Vec<_> = buf.into_iter().map(|b| b >= 5 && b <= 7).collect();
    if stuck {
        out.first_mut().map_or((), |b| *b = true);
        out.last_mut().map_or((), |b| *b = true);
    }
    out.into_boxed_slice()
}

fn life_stage(line_in: LineIn, line_out: LineOut, n: u64, stuck: bool) {
    let mut first = true;
    let mut bot = if let Ok(bot) = line_in.recv() { bot } else { return };
    let mut mid = new_line(&bot);
    let mut top;
    
    let line_out = prepend_life(line_out, n, stuck);

    while let Ok(inc) = line_in.recv() {
        assert_eq!(inc.len(), bot.len());
        top = mid;
        mid = bot;
        bot = inc;
        line_out.send(life_line(&top, &mid, &bot, first && stuck))
            .expect("broken pipe in life_stage");
        first = false;
    }
    top = mid;
    mid = bot;
    bot = new_line(&mid);
    line_out.send(life_line(&top, &mid, &bot, stuck)).expect("broken pipe in life_stage");
}

fn run_life<I>(input: I, n: u64, stuck: bool) -> mpsc::IntoIter<Box<Line>>
    where I: IntoIterator<Item=Box<Line>> + Send + 'static {
    let (final_out, final_in) = mpsc::sync_channel(BUFSIZE);
    let init_out = prepend_life(final_out, n, stuck);
    let cat = thread::spawn(move || {
        for line in input {
            init_out.send(line).expect("broken pipe in cat");
        }
    });
    drop(cat);
    final_in.into_iter()
}

fn parse_line(s: &str) -> Box<Line> {
    s.chars().map(|c| match c {
        '#' => true,
        '.' => false,
        _ => panic!("unexpected character {:?}", c)
    }).collect::<Vec<_>>().into_boxed_slice()
}

fn print_line(l: &Line) -> String {
    l.iter().map(|&b| if b { '#' } else { '.' }).collect()
}

fn main() {
    let n: u64 = env::args()
        .nth(1).expect("give number of iterations as argument")
        .parse().unwrap();
    let is_print = "print".starts_with(&env::args().nth(2).unwrap_or("count".to_owned()));

    let stdin = stdin();
    let input: Vec<_> = stdin.lock().lines().map(|rl| {
        parse_line(&rl.expect("I/O error"))
    }).collect();
    for &stuck in [false, true].iter() {
        let label = if stuck { "Stuck" } else { "Unstuck" };
        let output = run_life(input.clone(), n, stuck);
        if is_print {
            println!("{}:", label);
            for line in output {
                println!("{}", print_line(&line));
            }
        } else {
            let popcnt: usize = output.flat_map(|line| line.into_vec().into_iter())
                .fold(0, |a, b| if b { a + 1 } else { a });
            println!("{}: {}", label, popcnt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{run_life,parse_line,print_line};

    fn run(strs: &[&str], n: u64, stuck: bool) -> Vec<String> {
        run_life(own(strs).into_iter().map(|s| parse_line(&s)), n, stuck)
            .map(|l| print_line(&l)).collect()
    }

    fn own(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|&s| s.to_owned()).collect()
    }

    #[test]
    fn blink() {
        assert_eq!(run(&["...",
                         "###",
                         "..."], 1, false),
                   own(&[".#.",
                         ".#.",
                         ".#."]));
        assert_eq!(run(&[".#.",
                         ".#.",
                         ".#."], 1, false),
                   own(&["...",
                         "###",
                         "..."]));
    }
    
    #[test]
    fn blink_more() {
        assert_eq!(run(&["...",
                         "###",
                         "..."], 99, false),
                   own(&[".#.",
                         ".#.",
                         ".#."]));
        assert_eq!(run(&[".#.",
                         ".#.",
                         ".#."], 99, false),
                   own(&["...",
                         "###",
                         "..."]));
    }

    #[test]
    fn example() {
        const STUFF: [&'static[&'static str]; 5] = [
            &[".#.#.#",
              "...##.",
              "#....#",
              "..#...",
              "#.#..#",
              "####.."],

            &["..##..",
              "..##.#",
              "...##.",
              "......",
              "#.....",
              "#.##.."],

            &["..###.",
              "......",
              "..###.",
              "......",
              ".#....",
              ".#...."],

            &["...#..",
              "......",
              "...#..",
              "..##..",
              "......",
              "......"],

            &["......",
              "......",
              "..##..",
              "..##..",
              "......",
              "......"]];

        for i in 0..STUFF.len() {
            for j in i..STUFF.len() {
                assert_eq!(run(STUFF[i], (j - i) as u64, false), own(STUFF[j]))
            }
        }
    }

    #[test]
    fn example_stuck() {
        const STUFF: [&'static[&'static str]; 6] = [
            &["##.#.#",
              "...##.",
              "#....#",
              "..#...",
              "#.#..#",
              "####.#"],

            &["#.##.#",
              "####.#",
              "...##.",
              "......",
              "#...#.",
              "#.####"],

            &["#..#.#",
              "#....#",
              ".#.##.",
              "...##.",
              ".#..##",
              "##.###"],

            &["#...##",
              "####.#",
              "..##.#",
              "......",
              "##....",
              "####.#"],

            &["#.####",
              "#....#",
              "...#..",
              ".##...",
              "#.....",
              "#.#..#"],

            &["##.###",
              ".##..#",
              ".##...",
              ".##...",
              "#.#...",
              "##...#"]];

        for i in 0..STUFF.len() {
            for j in i..STUFF.len() {
                assert_eq!(run(STUFF[i], (j - i) as u64, true), own(STUFF[j]))
            }
        }
    }
    
    #[test]
    fn glide() {
        assert_eq!(run(&[".#..",
                         "..#.",
                         "###.",
                         "...."], 4, false),

                   own(&["....",
                         "..#.",
                         "...#",
                         ".###"]));
    }
}
