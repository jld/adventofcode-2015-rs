use std::collections::HashMap;
use std::iter::IntoIterator;
use std::str::FromStr;

use ::{Num,Stats};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    DupStat(String, Num, Num),
    MissingStat(&'static str),
    ColonFail,
    SpaceFail,
    NumberFail,
}

fn unpair<I: IntoIterator>(ii: I) -> Option<(I::Item, I::Item)> {
    let mut i = ii.into_iter();
    if let Some(v0) = i.next() {
        if let Some(v1) = i.next() {
            if let None = i.next() {
                return Some((v0, v1))
            }
        }
    }
    return None;
}

pub fn parse<'s>(s: &'s str) -> Result<(&'s str, Stats), Error> {
    let mut tab: HashMap<&'s str, Num> = HashMap::new();

    let (name, rest) = try!(unpair(s.split(':')).ok_or(Error::ColonFail));
    for sstat in rest.split(',').map(str::trim) {
        let (key, sval) = try!(unpair(sstat.split(' ')).ok_or(Error::SpaceFail));
        let val = try!(Num::from_str(sval).map_err(|_| Error::NumberFail));

        if let Some(oldval) = tab.insert(key, val) {
            return Err(Error::DupStat(key.to_owned(), oldval, val));
        }
    }

    let get = |key| {
        tab.get(key).ok_or(Error::MissingStat(key))
    };
    Ok((name, Stats {
        capacity: *try!(get("capacity")),
        durability: *try!(get("durability")),
        flavor: *try!(get("flavor")),
        texture: *try!(get("texture")),
        calories: *try!(get("calories")),
    }))
}

#[cfg(test)]
mod tests {
    use super::{unpair,parse};
    use ::Stats;
    
    #[test]
    fn check_unpair() {
        assert_eq!(unpair(vec![]), None::<(i32,i32)>);
        assert_eq!(unpair(vec![23]), None);
        assert_eq!(unpair(vec![23, 17]), Some((23, 17)));
        assert_eq!(unpair(vec![23, 17, 3]), None);
    }

    #[test]
    fn examples() {
        const EX: [&'static str; 2] = [
            "Butterscotch: capacity -1, durability -2, flavor 6, texture 3, calories 8",
            "Cinnamon: capacity 2, durability 3, flavor -2, texture -1, calories 3"];
        assert_eq!(parse(EX[0]), Ok(("Butterscotch", Stats {
            capacity: -1, durability: -2, flavor: 6, texture: 3, calories: 8
        })));
        assert_eq!(parse(EX[1]), Ok(("Cinnamon", Stats {
            capacity: 2, durability: 3, flavor: -2, texture: -1, calories: 3
        })));
    }
}
