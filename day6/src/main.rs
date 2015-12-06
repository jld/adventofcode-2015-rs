use std::cmp::{min,max};

type Coord = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect { xmin: Coord, ymin: Coord, xmax: Coord, ymax: Coord }

impl Rect {
    fn area(self) -> u64 {
        let xspan = (self.xmax - self.xmin) as u64 + 1;
        let yspan = (self.ymax - self.ymin) as u64 + 1;
        xspan * yspan
    }
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

enum Cmd {
    TurnOff,
    TurnOn,
    Toggle,
}

fn compute(cmds: &[Cmd], rects: &[Rect]) -> u64 {
    // parallel arrays save memory (not that it matters)
    assert_eq!(cmds.len(), rects.len());
    assert_eq!(cmds.len() as u32 as usize, cmds.len());
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
    while let Some(State { bnd, mut idx, inv }) = stack.pop() {
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
            Cmd::TurnOff => (),
            Cmd::TurnOn => area += hit.area(),
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
                                           ymin: bnd.ymax + 1, ymax: hit.ymax },
                               idx: idx, inv: inv });
        }
    }
    area
}
