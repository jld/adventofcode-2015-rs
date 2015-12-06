use std::cmp::{min,max};
use std::error::Error;
use std::env;
use std::fmt;
use std::io::{stdin, BufRead};
use std::num;
use std::ops::Range;
use std::str::FromStr;

type Coord = u16;
type Area = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect { xmin: Coord, ymin: Coord, xmax: Coord, ymax: Coord }

impl Rect {
    fn new(xymin: (Coord, Coord), xymax: (Coord, Coord)) -> Rect {
        let (xmin, ymin) = xymin;
        let (xmax, ymax) = xymax;
        assert!(xmin <= xmax);
        assert!(ymin <= ymax);
        Rect { xmin: xmin, xmax: xmax, ymin: ymin, ymax: ymax }
    }
    fn xrange(self) -> Range<usize> { (self.xmin as usize)..(self.xmax as usize + 1) }
    fn yrange(self) -> Range<usize> { (self.ymin as usize)..(self.ymax as usize + 1) }
    fn area(self) -> Area { self.xrange().len() as Area * self.yrange().len() as Area }
    fn intersect(self, other: Rect) -> Option<Rect> {
        #![allow(unused_parens)]
        if (self.xmax < other.xmin || other.xmax < self.xmin ||
            self.ymax < other.ymin || other.ymax < self.ymin) {
            None
        } else {
            Some(Rect {
                xmin: max(self.xmin, other.xmin),
                xmax: min(self.xmax, other.xmax),
                ymin: max(self.ymin, other.ymin),
                ymax: min(self.ymax, other.ymax),
            })
        }
    }
    fn merge(self, other: Rect) -> Rect {
        Rect {
            xmin: min(self.xmin, other.xmin),
            xmax: max(self.xmax, other.xmax),
            ymin: min(self.ymin, other.ymin),
            ymax: max(self.ymax, other.ymax),
        }
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cmd {
    TurnOff,
    TurnOn,
    Toggle,
}

fn compute(cmds: &[Cmd], rects: &[Rect]) -> Area {
    // parallel arrays save memory (not that it matters)
    assert_eq!(cmds.len(), rects.len());
    assert_eq!(cmds.len() as u32 as usize, cmds.len());
    #[derive(Debug)]
    struct State {
        bnd: Rect,
        idx: u32, // u32 saves memory (not that it matters)
        inv: bool,
    }
    let mut stack = Vec::new();
    if cmds.len() == 0 {
        return 0;
    }
    let bnd0 = rects.iter().skip(1).fold(rects[0], |ra, &rb| ra.merge(rb));
    stack.push(State { bnd: bnd0, idx: cmds.len() as u32, inv: false });
    let mut area = 0;
    // println!("Starting...");
    while let Some(State { bnd, mut idx, inv }) = stack.pop() {
        // println!("Handling bnd={:?} idx={:?} inv={:?}", bnd, idx, inv);
        debug_assert!(bnd.xmin <= bnd.xmax && bnd.ymin <= bnd.ymax);
        let mut maybe_hit = None;
        while maybe_hit.is_none() && idx > 0 {
            idx -= 1;
            maybe_hit = bnd.intersect(rects[idx as usize]);
        }
        let hit = match maybe_hit {
            None => {
                if inv {
                    area += bnd.area();
                }
                continue;
            }
            Some(hit) => hit,
        };
        match cmds[idx as usize] {
            Cmd::TurnOff => if inv { area += hit.area() },
            Cmd::TurnOn => if !inv { area += hit.area() },
            Cmd::Toggle => stack.push(State { bnd: hit, idx: idx, inv: !inv }),
        };
        // FIXME: the side rects could be arranged otherwise; does it matter?
        if bnd.xmin < hit.xmin {
            stack.push(State { bnd: Rect { xmin: bnd.xmin, xmax: hit.xmin - 1,
                                           ymin: bnd.ymin, ymax: bnd.ymax },
                               idx: idx, inv: inv });
        }
        if bnd.xmax > hit.xmax {
            stack.push(State { bnd: Rect { xmin: hit.xmax + 1, xmax: bnd.xmax,
                                           ymin: bnd.ymin, ymax: bnd.ymax },
                               idx: idx, inv: inv });
        }
        if bnd.ymin < hit.ymin {
            stack.push(State { bnd: Rect { xmin: hit.xmin, xmax: hit.xmax,
                                           ymin: bnd.ymin, ymax: hit.ymin - 1 },
                               idx: idx, inv: inv });
        }
        if bnd.ymax > hit.ymax {
            stack.push(State { bnd: Rect { xmin: hit.xmin, xmax: hit.xmax,
                                           ymin: hit.ymax + 1, ymax: bnd.ymax },
                               idx: idx, inv: inv });
        }
    }
    area
}

trait Light: Clone {
    fn zero() -> Self;
    fn turn_off(&mut self);
    fn turn_on(&mut self);
    fn toggle(&mut self);
    fn value(&self) -> Area;
}
impl Light for bool {
    fn zero() -> bool { false }
    fn turn_off(&mut self) { *self = false; }
    fn turn_on(&mut self) { *self = true; }
    fn toggle(&mut self) { *self = !*self; }
    fn value(&self) -> Area { if *self { 1 } else { 0 } }
}
impl Light for u16 {
    fn zero() -> u16 { 0 }
    fn turn_off(&mut self) { *self = self.saturating_sub(1); }
    fn turn_on(&mut self) { *self = self.checked_add(1).expect("overflow!"); }
    fn toggle(&mut self) { *self = self.checked_add(2).expect("overflow!"); }
    fn value(&self) -> Area { *self as Area }
}

fn compute_gen<L: Light>(cmds: &[Cmd], rects: &[Rect]) -> Area {
    if cmds.len() == 0 {
        return 0;
    }
    let bnd = rects.iter().skip(1).fold(rects[0], |ra, &rb| ra.merge(rb));
    let mut lights = vec![vec![L::zero(); bnd.xrange().len()]; bnd.yrange().len()];
    for i in 0..cmds.len() {
        let r = rects[i];
        for y in r.yrange() {
            for x in r.xrange() {
                let light = &mut lights[y - bnd.ymin as usize][x - bnd.xmin as usize];
                match cmds[i] {
                    Cmd::TurnOff => light.turn_off(),
                    Cmd::TurnOn => light.turn_on(),
                    Cmd::Toggle => light.toggle(),
                }
            }
        }
    }
    lights.iter()
          .map(|row| row.iter()
                        .map(L::value)
                        .fold(0 as Area, |a, n| a.checked_add(n).expect("overflow!")))
          .fold(0 as Area, |a, n| a.checked_add(n).expect("overflow!"))
}

#[derive(Debug)]
enum ParseError {
    EOL,
    ExtraJunk(String),
    BadVerb(String),
    BadState(String),
    BadPrep(String),
    CommaFail(String),
    IntFail(String, num::ParseIntError),
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::EOL =>
                write!(f, "unexpected end of line"),
            ParseError::ExtraJunk(ref junk) =>
                write!(f, "unexpected text {:?} after command", junk),
            ParseError::BadVerb(ref verb) =>
                write!(f, "unrecognized verb {:?}; expected \"toggle\" or \"turn\"", verb),
            ParseError::BadState(ref state) =>
                write!(f, "unrecognized state {:?}; expected \"on\" or \"off\"", state),
            ParseError::BadPrep(ref prep) =>
                write!(f, "unrecognized preposition {:?}; expected \"through\"", prep),
            ParseError::CommaFail(ref token) =>
                write!(f, "expected comma-separated pair; got {:?}", token),
            ParseError::IntFail(ref token, ref pie) =>
                write!(f, "invalid number {:?}: {}", token, pie),
        }
    }
}
impl Error for ParseError {
    fn cause(&self) -> Option<&Error> {
        match *self {
            ParseError::IntFail(_, ref ierr) => Some(ierr),
            _ => None
        }
    }
    fn description(&self) -> &str {
        match *self {
            ParseError::EOL => "unexpected end of line",
            ParseError::ExtraJunk(_) => "unexpected trailing words",
            ParseError::BadVerb(_) => "unrecognized verb",
            ParseError::BadState(_) => "unrecognized light state",
            ParseError::BadPrep(_) => "unrecognized preposition",
            ParseError::CommaFail(_) => "not a comma-separated pair",
            ParseError::IntFail(_, ref pie) => pie.description(),
        }
    }
}

fn parse_cmd<'l, 'w, I>(words: &'l mut I) -> Result<Cmd, ParseError>
    where I: Iterator<Item=&'w str> {
    match words.next() {
        Some("toggle") => Ok(Cmd::Toggle),
        Some("turn") => match words.next() {
            Some("on") => Ok(Cmd::TurnOn),
            Some("off") => Ok(Cmd::TurnOff),
            Some(huh) => Err(ParseError::BadState(huh.to_owned())),
            None => Err(ParseError::EOL),
        },
        Some(huh) => Err(ParseError::BadVerb(huh.to_owned())),
        None => Err(ParseError::EOL),
    }
}
fn parse_prep<'l, 'w, I>(words: &'l mut I) -> Result<(), ParseError>
    where I: Iterator<Item=&'w str> {
    match words.next() {
        Some("through") => Ok(()),
        Some(huh) => Err(ParseError::BadPrep(huh.to_owned())),
        None => Err(ParseError::EOL),
    }
}
fn parse_coord<'l, 'w, I>(words: &'l mut I) -> Result<Coord, ParseError>
    where I: Iterator<Item=&'w str> {
    if let Some(token) = words.next() {
        Coord::from_str(token).map_err(|ie| ParseError::IntFail(token.to_owned(), ie))
    } else {
        Err(ParseError::EOL)
    }
}
fn parse_point<'l, 'w, I>(words: &'l mut I) -> Result<(Coord, Coord), ParseError>
    where I: Iterator<Item=&'w str> {
    if let Some(token) = words.next() {
        let mut subtoks = token.split(',');
        let emap = |err| { match err {
            ParseError::EOL => ParseError::CommaFail(token.to_owned()),
            _ => err
        }};
        let x = try!(parse_coord(&mut subtoks).map_err(&emap));
        let y = try!(parse_coord(&mut subtoks).map_err(&emap));
        if subtoks.next().is_none() {
            Ok((x, y))
        } else {
            Err(ParseError::CommaFail(token.to_owned()))
        }
    } else {
        Err(ParseError::EOL)
    }
}
fn parse_eol<'l, 'w, I>(words: &mut I) -> Result<(), ParseError>
    where I: Iterator<Item=&'w str> {
    let stuff: Vec<_> = words.collect();
    if stuff.len() > 0 {
        Err(ParseError::ExtraJunk(stuff.join(" ")))
    } else {
        Ok(())
    }
}
fn parse(line: &str) -> Result<(Cmd, Rect), ParseError> {
    let mut words = line.split(char::is_whitespace).filter(|s| s.len() > 0);
    let cmd = try!(parse_cmd(&mut words));
    let xymin = try!(parse_point(&mut words));
    try!(parse_prep(&mut words));
    let xymax = try!(parse_point(&mut words));
    try!(parse_eol(&mut words));
    Ok((cmd, Rect::new(xymin, xymax)))
}

pub fn main() {
    let argv1 = env::args().nth(1);
    let compute = &compute as &Fn(&[Cmd], &[Rect]) -> Area;
    let compute_simple = &compute_gen::<bool> as &Fn(&[Cmd], &[Rect]) -> Area;
    let compute_nordic = &compute_gen::<u16> as &Fn(&[Cmd], &[Rect]) -> Area;
    let compute_fn;
    let mut nordicp = false;
    match argv1.as_ref().map(|s| s as &str /* Sigh. */).unwrap_or("fast") {
        "fast" => compute_fn = compute,
        "slow" => compute_fn = compute_simple,
        "nordic" => { compute_fn = compute_nordic; nordicp = true },
        huh => panic!("unknown command {:?}", huh)
    };
    let stdin = stdin();
    let mut cmds = Vec::new();
    let mut rects = Vec::new();
    for (num, line) in stdin.lock().lines().enumerate() {
        let line = line.expect("I/O error reading stdin");
        let (cmd, rect) = parse(&line).unwrap_or_else(|err| {
            panic!("{} on input line {}", err, num)
        });
        cmds.push(cmd);
        rects.push(rect);
    }
    let lights = compute_fn(&cmds, &rects);
    if nordicp {
        println!("Total brightness is {}.", lights);
    } else {
        println!("{} light{} lit.", lights, if lights == 1 { " is" } else { "s are" });
    }
}

#[cfg(test)]
mod test {
    extern crate rand;
    use super::{compute, compute_gen, Coord, Area, Cmd, Rect, parse};
    use self::rand::{Rng,SeedableRng};
    type Rand = self::rand::XorShiftRng;

    type FlatCase = [(Cmd, (Coord, Coord), (Coord, Coord), Option<Area>)];

    fn run_case(flat: &FlatCase) {
        let mut cmds = Vec::new();
        let mut rects = Vec::new();
        for &(cmd, xymin, xymax, maybe_exp) in flat {
            cmds.push(cmd);
            rects.push(Rect::new(xymin, xymax));
            let actual_simple = compute_gen::<bool>(&cmds, &rects);
            if let Some(expected) = maybe_exp {
                assert!(actual_simple == expected,
                        "compute_simple failure: got {}; expected {}; cmds={:?} rects={:?}",
                        actual_simple, expected, cmds, rects);
            }
            let actual = compute(&cmds, &rects);
            assert!(actual == actual_simple,
                    "divergence: got {}; expected {}; cmds={:?} rects={:?}",
                    actual, actual_simple, cmds, rects);
        }
    }

    #[test]
    fn very_simple() {
        run_case(&[(Cmd::TurnOn, (1, 1), (2, 3), Some(6))]);
        run_case(&[(Cmd::Toggle, (1, 1), (2, 3), Some(6))]);
        run_case(&[(Cmd::TurnOff, (1, 1), (2, 3), Some(0))]);
        run_case(&[(Cmd::TurnOn, (11, 21), (12, 23), Some(6))]);
        run_case(&[(Cmd::TurnOn, (1, 1), (2, 3), None),
                   (Cmd::TurnOn, (1, 1), (2, 3), Some(6))]);
        run_case(&[(Cmd::Toggle, (1, 1), (2, 3), None),
                   (Cmd::Toggle, (1, 1), (2, 3), Some(0))]);
        run_case(&[(Cmd::TurnOn, (1, 1), (2, 3), None),
                   (Cmd::TurnOff, (1, 1), (2, 3), Some(0))]);
        run_case(&[(Cmd::Toggle, (1, 1), (2, 3), None),
                   (Cmd::Toggle, (1, 1), (3, 2), Some(4))]);
        run_case(&[(Cmd::TurnOn, (1, 1), (2, 3), None),
                   (Cmd::TurnOff, (1, 1), (3, 2), Some(2))]);
    }

    #[test]
    fn example1() {
        run_case(&[(Cmd::TurnOn, (0, 0), (999, 999), Some(1000_000))]);
        run_case(&[(Cmd::TurnOn, (0, 0), (9, 9), Some(100)),
                   (Cmd::TurnOn, (0, 0), (999, 999), Some(1000_000))]);
    }

    #[test]
    fn example2() {
        run_case(&[(Cmd::Toggle, (0, 0), (999, 0), Some(1000))]);
        run_case(&[(Cmd::TurnOn, (0, 0), (9, 9), Some(100)),
                   (Cmd::Toggle, (0, 0), (999, 0), Some(1000 - 10 + 90))]);
    }

    #[test]
    fn example3() {
        run_case(&[(Cmd::TurnOff, (499, 499), (500, 500), Some(0))]);
        run_case(&[(Cmd::TurnOn, (498, 498), (501, 499), Some(8)),
                   (Cmd::TurnOff, (499, 499), (500, 500), Some(6))]);
        run_case(&[(Cmd::TurnOn, (498, 498), (499, 501), Some(8)),
                   (Cmd::TurnOff, (499, 499), (500, 500), Some(6))]);
    }

    fn random_range(rng: &mut Rand, bmin: Coord, bmax: Coord) -> (Coord, Coord) {
        loop {
            let cmin = rng.gen_range(bmin as usize, bmax as usize + 1) as Coord;
            let cmax = rng.gen_range(bmin as usize, bmax as usize + 1) as Coord;
            if cmin <= cmax {
                return (cmin, cmax);
            }
        }
    }

    #[test] #[ignore]
    fn randomly() {
        const LEN_MAX: usize = 100;
        const TESTS: usize = 100;

        let mut rng = Rand::from_seed([17, 17, 17, 17]);
        let mut len = 1;
        while len <= LEN_MAX {
            for _ in 0..TESTS {
                let mut case = Vec::new();
                for _ in 0..len {
                    let cmd = *rng.choose(&[Cmd::TurnOff, Cmd::TurnOn, Cmd::Toggle]).unwrap();
                    let (xmin, xmax) = random_range(&mut rng, 0, 99);
                    let (ymin, ymax) = random_range(&mut rng, 0, 99);
                    case.push((cmd, (xmin, ymin), (xmax, ymax), None));
                }
                run_case(&case);
            }
            len += len/2 + 1;
        }
    }

    #[test]
    fn example_area() {
        assert_eq!(Rect::new((0, 0), (999, 999)).area(), 1000_000);
        assert_eq!(Rect::new((0, 0), (999, 0)).area(), 1000);
        assert_eq!(Rect::new((499, 499), (500, 500)).area(), 4);
    }

    #[test]
    fn parse_examples() {
        assert_eq!(parse("turn on 0,0 through 999,999").unwrap(),
                   (Cmd::TurnOn, Rect::new((0, 0), (999, 999))));
        assert_eq!(parse("toggle 0,0 through 999,0").unwrap(),
                   (Cmd::Toggle, Rect::new((0, 0), (999, 0))));
        assert_eq!(parse("turn off 499,499 through 500,500").unwrap(),
                   (Cmd::TurnOff, Rect::new((499, 499), (500, 500))));
    }

    #[test]
    fn parse_spacey() {
        assert_eq!(parse("turn on   0,0 through 999,999").unwrap(),
                   (Cmd::TurnOn, Rect::new((0, 0), (999, 999))));
        assert_eq!(parse("     turn on   0,0 through 999,999").unwrap(),
                   (Cmd::TurnOn, Rect::new((0, 0), (999, 999))));
        assert_eq!(parse("     turn  on   0,0   through   999,999     ").unwrap(),
                   (Cmd::TurnOn, Rect::new((0, 0), (999, 999))));
        assert_eq!(parse("\tturn on 0,0\tthrough 999,999\t\r").unwrap(),
                   (Cmd::TurnOn, Rect::new((0, 0), (999, 999))));
    }

    #[test] #[should_panic(expected="unrecognized verb \"switch\"")]
    fn parse_fail_verb() {
        panic!("{}", parse("switch on 0,0 through 999,999").unwrap_err());
    }
    #[test] #[should_panic(expected="unrecognized state \"up\"")]
    fn parse_fail_state() {
        panic!("{}", parse("turn up 0,0 through 999,999").unwrap_err());
    }
    #[test] #[should_panic(expected="unrecognized preposition \"to\"")]
    fn parse_fail_prep() {
        panic!("{}", parse("turn on 0,0 to 999,999").unwrap_err());
    }
    #[test] #[should_panic(expected="expected comma-separated pair; got \"0\"")]
    fn parse_fail_comma1() {
        panic!("{}", parse("turn on 0 through 999,999").unwrap_err());
    }
    #[test] #[should_panic(expected="expected comma-separated pair; got \"0,0,\"")]
    fn parse_fail_comma2() {
        panic!("{}", parse("turn on 0,0, through 999,999").unwrap_err());
    }
    #[test] #[should_panic(expected="unexpected end of line")]
    fn parse_fail_end1() {
        panic!("{}", parse("turn on 0,0").unwrap_err());
        // There are a few other variants but I don't feel like writing them all.
    }
    #[test] #[should_panic(expected="unexpected text \"or else\" after")]
    fn parse_fail_end2() {
        panic!("{}", parse("turn on 0,0 through 999,999 or else").unwrap_err());
    }
    #[test] #[should_panic(expected="number too large")]
    fn parse_fail_overflow() {
        panic!("{}", parse("turn on 0,0 through 99999,9").unwrap_err());
    }
    #[test] #[should_panic(expected="invalid digit")]
    fn parse_fail_underflow() {
        panic!("{}", parse("turn on -999,0 through 0,999").unwrap_err());
    }
    #[test] #[should_panic(expected="invalid digit")]
    fn parse_fail_nan() {
        panic!("{}", parse("turn on 0,0 through 0x3e7,0x3e7").unwrap_err());
    }
    #[test] #[should_panic(expected="parse integer from empty string")]
    fn parse_fail_emptynum() {
        panic!("{}", parse("turn on 0, through 999,999").unwrap_err());
    }

    #[test]
    fn nordic_examples() {
        assert_eq!(compute_gen::<u16>(&[Cmd::TurnOn], &[Rect::new((0, 0), (0, 0))]), 1);
        assert_eq!(compute_gen::<u16>(&[Cmd::Toggle], &[Rect::new((0, 0), (999, 999))]), 2000_000);
    }
}
