
#[derive(Clone, Debug)]
enum QTree {
    Off,
    On,
    Mixed(Box<[QTree; 4]>),
    LazyRect(Box<Rect>),
}
#[derive(Clone, Debug)]
struct Rect { level: u8, xmin: u64, ymin: u64, xmax: u64, ymax: u64 }
/*
impl Rect {
    fn reduce(self) -> Rect {
        let xtrabits = (self.xmin | self.xmax | self.ymin | self.ymax | 1 << self.level)
            .trailing_zeros();
        Rect { level: self.level - xtrabits,
               xmin: self.xmin >> xtrabits,
               xmax: self.xmax >> xtrabits,
               ymin: self.ymin >> xtrabits,
               ymax: self.ymax >> xtrabits,
        }
    }
}
*/
impl QTree {
    fn zero() -> QTree {
        Off
    }
    fn one() -> QTree {
        On
    }
    fn rect(r: Rect) -> QTree {
        debug_assert!(xmin <= xmax);
        debug_assert!(ymin <= ymax);
        if r.xmin >= r.xmax || r.ymin >= r.ymax {
            return Off;
        }
        let cmax = 1 << r.level;
        debug_assert!(xmax <= cmax);
        debug_assert!(ymax <= cmax);
        if r.xmin == 0 && r.ymin == 0 && r.xmax == cmax && r.ymax == cmax {
            return On;
        }
        LazyRect(Box::new(r))
    }

    fn count(&self, level: u8) -> u64 {
        match *self {
            Off => 0,
            On => 1 << (2 * level),
            Mixed(ref bx) => {
                assert!(level > 0);
                bx.iter().map(|quad| quad.count(level - 1)).fold(0, |a, c| a + c)
            }
        }
    }
    fn probe(&self, level: u8, x: u64, y: u64) -> bool {
        debug_assert_eq!(x >> level, 0);
        debug_assert_eq!(y >> level, 0);
        match *self {
            Off => false,
            On => true,
            Mixed(ref quads) => {
                assert!(level > 0);
                let cmid = 1 << (level - 1);
                let idx = (if y >= cmid { 2 } else { 0 }) + (if x >= cmid { 1 } else { 0 });
                // LLVM can optimize these `%`s (but GCC doesn't?).
                quads[idx].probe(level - 1, x % cmid, y % cmid)
            },
            LazyRect(ref r) => {
                assert!(r.level >= level);
                let ax = x << (r.level - level);
                let ay = y << (r.level - level);
                // FIXME: ... do I even want to finish this?
            }
        }
    }
    fn unwrap(self) -> [QTree; 4] {
        match self {
            Off | On => panic!("don't use QTree::unwrap without a variant test"),
            Mixed(quads) => quads,
            LazyRect(r) {
                assert!(r.level > 0);
                let nlevel = r.level - 1;
                let cmid = 1 << nlevel;
                let xminl = min(r.xmin, cmid);
                let xminh = max(r.xmin, cmid) - cmid;
                let xmaxl = min(r.xmax, cmid);
                let xmaxh = max(r.xmax, cmid) - cmid;
                let yminl = min(r.ymin, cmid);
                let yminh = max(r.ymin, cmid) - cmid;
                let ymaxl = min(r.ymax, cmid);
                let ymaxh = max(r.ymax, cmid) - cmid;
                [QTree::rect(Rect { level: nlevel
                                    xmin: xminl, xmax: xmaxl,
                                    ymin: yminl, ymax: ymaxl }),
                 QTree::rect(Rect { level: nlevel,
                                    xmin: xminh, xmax: xmaxh,
                                    ymin: yminl, ymax: ymaxl }),
                 QTree::rect(Rect { level: nlevel,
                                    xmin: xminl, xmax: xmaxl,
                                    ymin: yminh, ymax: ymaxh }),
                 QTree::rect(Rect { level: nlevel,
                                    xmin: xminh, xmax: xmaxh,
                                    ymin: yminh, ymax: ymaxh })]
            }
        }
    }
}
        
