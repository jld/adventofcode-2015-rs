use regex::Regex;
use sue::{Sue,UrSue,Count,Pred};

pub struct Parser {
    line: Regex,
    kv: Regex,
}
impl Parser {
    // This one is a little more permissive than the other.  Maybe
    // we're trying to gather all the evidence possible or something?
    // I don't know.
    pub fn new() -> Parser { Parser {
        line: Regex::new(r"^Sue\s+(\w+)\s*:(.*)").unwrap(),
        kv: Regex::new(r"\b(\w+)\s*:\s*([0-9]+)\b").unwrap(),
    }}
    pub fn parse_ur<'a, 'b>(&'a self, line: &'b str) -> Option<(&'b str, UrSue<'b>)> {
        self.line.captures(line).map(|lcaps| {
            let (ident, kvs) = (lcaps.at(1).unwrap(), lcaps.at(2).unwrap());
            (ident, UrSue {
                facts: self.kv.captures_iter(kvs).flat_map(|kvcaps| {
                    let (kstr, vstr) = (kvcaps.at(1).unwrap(), kvcaps.at(2).unwrap());
                    vstr.parse().ok().map(|v| {
                        (kstr, v)
                    })
                }).collect()
            })
        })
    }
    pub fn parse<'a, 'b>(&'a self, line: &'b str) -> Option<(Sue<'b>, Sue<'b>)> {
        self.parse_ur(line).map(|(name, ur)| {
            (ur.reify(name, |k, v| mkpred(false, k, v)),
             ur.reify(name, |k, v| mkpred(true, k, v)))
        })
    }
}

fn mkpred(p2: bool, k: &str, v: Count) -> Pred {
    match (p2, k) {
        (true, "cats") | (true, "trees") => Box::new(move |uv| uv > v),
        (true, "pomeranians") | (true, "goldfish") => Box::new(move |uv| uv < v),
        _ => Box::new(move |uv| uv == v)
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;

    #[test]
    fn example_as_ur() {
        let (name, ur) = {
            let p = Parser::new();
            p.parse_ur("Sue 124: cars: 10, children: 1, trees: 0").unwrap()
        };
        assert_eq!(name, "124");
        assert_eq!(ur.facts.len(), 3);
        assert_eq!(ur.facts.get("cars"), Some(&10));
        assert_eq!(ur.facts.get("children"), Some(&1));
        assert_eq!(ur.facts.get("trees"), Some(&0));
        assert_eq!(ur.facts.get("124"), None);
    }
}
