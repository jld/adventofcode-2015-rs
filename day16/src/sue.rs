use std::collections::HashMap;

pub type Count = u8;
pub type Pred = Box<Fn(Count) -> bool>;

pub struct UrSue<'s> {
    pub facts: HashMap<&'s str, Count>,
}
pub struct Sue<'s> {
    pub ident: &'s str,
    pub facts: HashMap<&'s str, Pred>,
}

impl<'s> Sue<'s> {
    pub fn test(&self, ur: &UrSue) -> bool {
        self.facts.iter()
            .flat_map(|(k, vp)| ur.facts.get(k).map(|&v| (vp, v)))
            .all(|(vp, v)| vp(v))
    }
}

macro_rules! ur_sue {
    { $($k:ident: $v:expr),* } => { UrSue::from(&[$((stringify!($k), $v)),*]) }
}

impl<'s> UrSue<'s> {
    pub fn from(f: &[(&'s str, Count)]) -> UrSue<'s> {
        UrSue {
            facts: f.iter().cloned().collect()
        }
    }
    pub fn the() -> UrSue<'s> {
        ur_sue!{
            children: 3,
              cats: 7,
              samoyeds: 2,
              pomeranians: 3,
              akitas: 0,
              vizslas: 0,
              goldfish: 5,
              trees: 3,
              cars: 2,
              perfumes: 1
        }
    }
    pub fn reify<F>(&self, name: &'s str, interp: F) -> Sue<'s>
        where F: Fn(&'s str, Count) -> Pred {
        Sue {
            ident: name,
            facts: self.facts.iter().map(|(&k, &v)| (k, interp(k, v))).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Sue,UrSue};

    fn the_sue() -> Sue<'static> { UrSue::the().reify("Ω", |_k, v| Box::new(move |ov| v == ov)) }

    #[test]
    fn the_get() {
        assert_eq!(UrSue::the().facts.get("trees"), Some(&3));
    }

    #[test]
    fn test_yes() {
        assert!(the_sue().test(&ur_sue!{ cats: 7, samoyeds: 2 }));
    }

    #[test]
    fn test_no() {
        assert!(!the_sue().test(&ur_sue!{ cats: 7, cars: 9 }));
    }

    #[test]
    fn test_slide_yes() {
        assert!(the_sue().test(&ur_sue!{ cats: 7, corgis: 2 }));
    }

    #[test]
    fn test_slide_no() {
        assert!(!the_sue().test(&ur_sue!{ cats: 7, corgis: 2, akitas: 5 }));
    }

    #[test]
    fn test_vacuous() {
        assert!(the_sue().test(&ur_sue!{ }));
    }

    #[test]
    fn test_semivac() {
        assert!(the_sue().test(&ur_sue!{ corgis: 2 }));
    }
}
