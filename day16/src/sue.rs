use std::collections::HashMap;

pub type Count = u8;

pub struct Sue {
    pub ident: String,
    pub facts: HashMap<String, Count>,
}
impl Sue {
    pub fn consistent(&self, other: &Sue) -> bool {
        let (smaller, larger) = if self.facts.len() < other.facts.len() {
            (self, other)
        } else {
            (other, self)
        };
        for (k, v) in smaller.facts.iter() {
            if let Some(ov) = larger.facts.get(k) {
                if ov != v {
                    return false;
                }
            }
        }
        return true;
    }
    fn from(i: &str, f: &[(&str, Count)]) -> Sue {
        Sue {
            ident: i.to_owned(),
            facts: f.iter().map(|&(k,v)| (k.to_owned(), v)).collect()
        }
    }
}

macro_rules! sue {
    { $id:expr, $($k:ident: $v:expr),* } => { Sue::from($id, &[$((stringify!($k), $v)),*]) }
}

impl Sue {
    pub fn the() -> Sue {
        sue!{ "Ω",
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
}

#[cfg(test)]
mod tests {
    use super::Sue;

    #[test]
    fn the_get() {
        assert_eq!(Sue::the().facts.get("trees"), Some(&3));
    }

    #[test]
    fn consistent_yes() {
        assert!(Sue::the().consistent(&sue!{ "test", cats: 7, samoyeds: 2 }));
    }

    #[test]
    fn consistent_no() {
        assert!(!Sue::the().consistent(&sue!{ "test", cats: 7, cars: 9 }));
    }

    #[test]
    fn consistent_slide_yes() {
        assert!(Sue::the().consistent(&sue!{ "test", cats: 7, corgis: 2 }));
    }

    #[test]
    fn consistent_slide_no() {
        assert!(!Sue::the().consistent(&sue!{ "test", cats: 7, corgis: 2, akitas: 5 }));
    }

    #[test]
    fn consistent_vacuous() {
        assert!(Sue::the().consistent(&sue!{ "test" /* sigh: */ , }));
    }

    #[test]
    fn consistent_semivac() {
        assert!(Sue::the().consistent(&sue!{ "test", corgis: 2 }));
    }
}
