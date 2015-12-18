const BUFSIZE: usize = 10;

use std::mem::drop;
use std::sync::mpsc;
use std::thread;

type Line = [bool];
type LineIn = mpsc::Receiver<Box<Line>>;
type LineOut = mpsc::SyncSender<Box<Line>>;

fn new_line(like: &Line) -> Box<Line> { vec![false; like.len()].into_boxed_slice() }

fn prepend_life(line_out: LineOut, n: usize) -> LineOut {
    if n == 0 {
        return line_out;
    }
    let (new_out, line_in) = mpsc::sync_channel(BUFSIZE);
    thread::spawn(move || life_stage(line_in, line_out, n - 1));
    new_out
}

fn life_line(top: &Line, mid: &Line, bot: &Line) -> Box<Line> {
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
    buf.into_iter().map(|b| b >= 5 && b <= 7).collect::<Vec<_>>().into_boxed_slice()
}

fn life_stage(line_in: LineIn, line_out: LineOut, n: usize) {
    let mut bot = if let Ok(bot) = line_in.recv() { bot } else { return };
    let mut mid = new_line(&bot);
    let mut top;
    
    let line_out = prepend_life(line_out, n);

    while let Ok(inc) = line_in.recv() {
        assert_eq!(inc.len(), bot.len());
        top = mid;
        mid = bot;
        bot = inc;
        line_out.send(life_line(&top, &mid, &bot)).expect("broken pipe in life_stage");
    }
    top = mid;
    mid = bot;
    bot = new_line(&mid);
    line_out.send(life_line(&top, &mid, &bot)).expect("broken pipe in life_stage");
}


fn run_life<I>(input: I, n: usize) -> mpsc::IntoIter<Box<Line>>
    where I: IntoIterator<Item=Box<Line>> + Send + 'static {
    let (final_out, final_in) = mpsc::sync_channel(BUFSIZE);
    let init_out = prepend_life(final_out, n);
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
    // stuff
}

#[cfg(test)]
mod tests {
    use super::{run_life,parse_line,print_line};

    fn run(strs: &[&str], n: usize) -> Vec<String> {
        run_life(own(strs).into_iter().map(|s| parse_line(&s)), n).map(|l| print_line(&l)).collect()
    }

    fn own(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|&s| s.to_owned()).collect()
    }

    #[test]
    fn blink() {
        assert_eq!(run(&["...",
                         "###",
                         "..."], 1),
                   own(&[".#.",
                         ".#.",
                         ".#."]));
        assert_eq!(run(&[".#.",
                         ".#.",
                         ".#."], 1),
                   own(&["...",
                         "###",
                         "..."]));
    }
    
    #[test]
    fn blink_more() {
        assert_eq!(run(&["...",
                         "###",
                         "..."], 99),
                   own(&[".#.",
                         ".#.",
                         ".#."]));
        assert_eq!(run(&[".#.",
                         ".#.",
                         ".#."], 99),
                   own(&["...",
                         "###",
                         "..."]));
    }
}
