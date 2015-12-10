use std::env;
use std::fmt::Display;
use std::str::FromStr;

struct RLE<I> where I: Iterator, I::Item: Eq {
    inner: I,
    next_thing: Option<I::Item>
}
impl<I> RLE<I> where I: Iterator, I::Item: Eq {
    fn new(i: I) -> RLE<I> { RLE { inner: i, next_thing: None } }
}
impl<I> Iterator for RLE<I> where I: Iterator, I::Item: Eq {
    type Item = (usize, I::Item);
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next_thing.take();
        let next = next.or_else(|| self.inner.next());
        next.map(|thing| {
            let mut count = 1;
            loop {
                match self.inner.next() {
                    None => break,
                    Some(ref next) if *next == thing =>
                        count += 1,
                    Some(other) => {
                        self.next_thing = Some(other);
                        break
                    }
                }
            };
            (count, thing)
        })
    }
}

struct CharStream {
    buf: Vec<char>
}
impl CharStream {
    fn new() -> CharStream { CharStream { buf: Vec::new() } }
    fn from(s: String) -> CharStream {
        let mut cs = CharStream::new();
        cs.replenish(move || Some(s));
        cs
    }
    fn replenish<F>(&mut self, f: F) where F: FnOnce() -> Option<String> {
        if self.buf.is_empty() {
            if let Some(s) = f() {
                self.buf.extend(s.chars());
                self.buf.reverse();
            }
        }
    }
}
impl Iterator for CharStream {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        self.buf.pop()
    }
}

struct ElfGame<I> where I: Iterator, I::Item: Display + Eq {
    inner: RLE<I>,
    buf: CharStream,
}
impl<I> ElfGame<I> where I: Iterator, I::Item: Display + Eq {
    fn new(i: I) -> ElfGame<I> {
        ElfGame { inner: RLE::new(i), buf: CharStream::new() }
    }
}
impl<I> Iterator for ElfGame<I> where I: Iterator, I::Item: Display + Eq {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        let ib = &mut self.inner;
        self.buf.replenish(|| ib.next().map(|(n, c)| format!("{}{}", n, c)));
        self.buf.next()
    }
}

fn elf_game_n(s: &str, n: usize) -> Box<Iterator<Item=char>> {
    let mut bx: Box<Iterator<Item=char>> = Box::new(CharStream::from(s.to_owned()));
    for _ in 0..n {
        bx = Box::new(ElfGame::new(bx));
    }
    bx
}

#[allow(dead_code)]
fn elf_game(s: &str) -> String {
    elf_game_n(s, 1).collect()
}

fn main() {
    let mut argv = env::args().skip(1);
    let thing = argv.next().expect("Usage: day10 <input> [<count>]");
    let elves = argv.next().map(|s| usize::from_str(&s).unwrap()).unwrap_or(40);
    println!("Length: {}", elf_game_n(&thing, elves).count());
}

#[cfg(test)]
mod test {
    use super::{RLE, elf_game, elf_game_n};

    fn rle<I: Eq>(v: Vec<I>) -> Vec<(usize, I)> {
        RLE::new(v.into_iter()).collect()
    }
    
    #[test]
    fn rle_simple() {
        assert_eq!(rle::<usize>(vec![]), vec![]);
        assert_eq!(rle(vec![17]), vec![(1, 17)]);
        assert_eq!(rle(vec![17, 17]), vec![(2, 17)]);
        assert_eq!(rle(vec![17, 23]), vec![(1, 17), (1, 23)]);
        assert_eq!(rle(vec![17, 17, 23]), vec![(2, 17), (1, 23)]);
        assert_eq!(rle(vec![17, 23, 23]), vec![(1, 17), (2, 23)]);
        assert_eq!(rle(vec![17, 17, 23, 23]), vec![(2, 17), (2, 23)]);
    }

    #[test]
    fn examples() {
        assert_eq!(elf_game("211"), "1221");
        assert_eq!(elf_game("1"), "11");
        assert_eq!(elf_game("11"), "21");
        assert_eq!(elf_game("21"), "1211");
        assert_eq!(elf_game("1211"), "111221");
        assert_eq!(elf_game("111221"), "312211");
    }

    #[test]
    fn nested() {
        let s: String = elf_game_n("1", 5).collect();
        assert_eq!(s, "312211");
    }
}
