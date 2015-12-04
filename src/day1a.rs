//use std::io::stdin;

// At one point I mixed up a time and a height, which was caught only
// because one of them was signed.  So let's have some newtypes:

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Time(usize);
impl Time {
    pub fn get(&self) -> usize { self.0 }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Height(isize);
impl Height {
    pub fn get(&self) -> isize { self.0 }
    pub fn step(&self, m: Move) -> Height {
        match m {
            Move::Up => Height(self.get() + 1),
            Move::Down => Height(self.get() - 1),
        }
    }
    pub fn step_mut(&mut self, m: Move) {
        let next = self.step(m);
        *self = next;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Move {
    Up,
    Down,
}

// And isn't this so much nicer than `(isize, Option<usize>)`?
#[derive(PartialEq, Eq, Clone, Debug)]
struct Result {
    end_floor: Height,
    basement_time: Option<Time>
}

fn move_of_char(c: char) -> Move {
    match c {
        '(' => Move::Up,
        ')' => Move::Down,
        _ => panic!("unexpected character {}", c)
    }
}

fn compute(s: &str) -> Result {
    let start = Result { end_floor: Height(0), basement_time: None };
    let moves = s.chars().map(move_of_char);
    let when_where = moves.scan(start.end_floor, |state, m| {
        state.step_mut(m);
        Some(*state)
    }).enumerate();
    when_where.fold(start, |res, (t, h)| {
        let maybe_basement = if h < Height(0) { Some(Time(t)) } else { None };
        Result {
            end_floor: h,
            basement_time: res.basement_time.or(maybe_basement)
        }
    })
}

#[cfg(test)]
mod test {
    use super::{Result, Height, Time, compute};

    macro_rules! case {
        ($s:expr => $ef:expr) => {
            assert_eq!(compute($s), Result {
                end_floor: Height($ef),
                basement_time: None
            })
        };
        ($s:expr => $ef:expr, $bt:expr) => {
            assert_eq!(compute($s), Result {
                end_floor: Height($ef),
                basement_time: Some(Time($bt))
            })
        };
    }

    #[test]
    fn injectivity() {
        // This is mostly here to make dead code warnings go away.
        assert_eq!(Time(17).get(), 17);
        assert_eq!(Height(17).get(), 17);
        assert_eq!(Height(-17).get(), -17);
    }

    #[test]
    fn spec_line1() {
        case!("(())" => 0);
        case!("()()" => 0);
    }

    #[test]
    fn spec_line2() {
        case!("(((" => 3);
        case!("(()(()(" => 3);
    }

    #[test]
    fn spec_line3() {
        case!("))(((((" => 3, 0);
    }
}
