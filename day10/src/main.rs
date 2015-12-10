use std::env;
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

fn elf_game(s: &str) -> String {
    let bits: Vec<_> = RLE::new(s.chars()).map(|(n, c)| format!("{}{}", n, c)).collect();
    bits.concat()
}

fn main() {
    let mut argv = env::args().skip(1);
    let mut thing = argv.next().expect("Usage: day10 <input> [<count>]");
    let count = argv.next().map(|s| usize::from_str(&s).unwrap()).unwrap_or(40);
    for _ in 0..count {
        thing = elf_game(&thing);
    }
    println!("Length: {}", thing.len());
}

#[cfg(test)]
mod test {
    use super::{RLE, elf_game};

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
}
