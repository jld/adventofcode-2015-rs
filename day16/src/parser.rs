use regex::Regex;
use sue::Sue;

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
    pub fn parse(&self, line: &str) -> Option<Sue> {
        self.line.captures(line).map(|lcaps| {
            let (ident, kvs) = (&lcaps[1], &lcaps[2]);
            Sue { ident: ident.to_owned(),
                  facts: self.kv.captures_iter(kvs).flat_map(|kvcaps| {
                      let (kstr, vstr) = (&kvcaps[1], &kvcaps[2]);
                      vstr.parse().ok().map(|v| {
                          (kstr.to_owned(), v)
                      })
                  }).collect()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;

    #[test]
    fn example() {
        let p = Parser::new();
        let sue = p.parse("Sue 124: cars: 10, children: 1, trees: 0").unwrap();
        assert_eq!(sue.ident, "124");
        assert_eq!(sue.facts.len(), 3);
        assert_eq!(sue.facts.get("cars"), Some(&10));
        assert_eq!(sue.facts.get("children"), Some(&1));
        assert_eq!(sue.facts.get("trees"), Some(&0));
        assert_eq!(sue.facts.get("124"), None);
    }
}
