//use std::io::stdin;

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
    
    fn ra(ef: isize) -> Result { Result {
        end_floor: Height(ef),
        basement_time: None
    }}
    fn rb(ef: isize, bt: usize) -> Result { Result {
        end_floor: Height(ef),
        basement_time: Some(Time(bt))
    }}

    macro_rules! case {
        ($s:expr => $ef:expr) => { assert_eq!(compute($s), ra($ef)) };
        ($s:expr => $ef:expr, $bt:expr) => { assert_eq!(compute($s), rb($ef, $t)) };
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
        assert_eq!(compute("(())"), ra(0));
        assert_eq!(compute("()()"), ra(0));
    }

    #[test]
    fn spec_line2() {
        assert_eq!(compute("((("), ra(3));
        assert_eq!(compute("(()(()("), ra(3));
    }

    #[test]
    fn spec_line3() {
        assert_eq!(compute("))((((("), rb(3, 0));
    }
}
