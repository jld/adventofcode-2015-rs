use std::cmp::{min,max};
use std::ops::Range;

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

fn compute_simple(cmds: &[Cmd], rects: &[Rect]) -> Area {
    if cmds.len() == 0 {
        return 0;
    }
    let bnd = rects.iter().skip(1).fold(rects[0], |ra, &rb| ra.merge(rb));
    let mut lights = vec![vec![false; bnd.xrange().len()]; bnd.yrange().len()];
    for i in 0..cmds.len() {
        let r = rects[i];
        for y in r.yrange() {
            for x in r.xrange() {
                let light = &mut lights[y - bnd.ymin as usize][x - bnd.xmin as usize];
                match cmds[i] {
                    Cmd::TurnOff => *light = false,
                    Cmd::TurnOn => *light = true,
                    Cmd::Toggle => *light = !*light,
                }
            }
        }
    }
    lights.iter()
          .map(|row| row.iter()
                        .map(|&light| if light { 1 } else { 0 })
                        .fold(0, |a, n| a + n))
          .fold(0, |a, n| a + n)
}

#[cfg(test)]
mod test {
    extern crate rand;
    use super::{compute, compute_simple, Coord, Area, Cmd, Rect};
    use self::rand::{Rng,SeedableRng};
    type Rand = self::rand::XorShiftRng;

    type FlatCase = [(Cmd, (Coord, Coord), (Coord, Coord), Option<Area>)];

    fn run_case(flat: &FlatCase) {
        let mut cmds = Vec::new();
        let mut rects = Vec::new();
        for &(cmd, xymin, xymax, maybe_exp) in flat {
            cmds.push(cmd);
            rects.push(Rect::new(xymin, xymax));
            let actual_simple = compute_simple(&cmds, &rects);
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
        run_case(&[(Cmd::TurnOn, (0, 0), (999, 999), Some(1000000))]);
        run_case(&[(Cmd::TurnOn, (0, 0), (9, 9), Some(100)),
                   (Cmd::TurnOn, (0, 0), (999, 999), Some(1000000))]);
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
}
