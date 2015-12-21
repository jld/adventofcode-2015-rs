extern crate regex;
use regex::Regex;
use std::collections::HashSet;

struct Problem {
    rewrites: Vec<(Regex, String)>,
}
impl Problem {
    fn new() -> Self {
        Problem { rewrites: vec![] }
    }
    fn add(&mut self, lhs: &str, rhs: &str) {
        self.rewrites.push((Regex::new(lhs).expect("bad pattern"), rhs.to_owned()));
    }
    fn add_lines<I: Iterator<Item=String>>(&mut self, lines: &mut I) {
        let arrow = Regex::new(r"^(\pL+) => (\pL+)$").unwrap();
        for line in lines {
            if line.is_empty() {
                break;
            }
            let caps = arrow.captures(&line).expect("syntax error");
            self.add(&caps[1], &caps[2]);
        }
    }
    fn from_lines<I: Iterator<Item=String>>(lines: &mut I) -> Self {
        let mut that = Self::new();
        that.add_lines(lines);
        that
    }
    fn rewrite(&self, before: &str) -> HashSet<String> {
        let mut set = HashSet::new();
        for rw in self.rewrites.iter() {
            for (start, end) in rw.0.find_iter(before) {
                let mut after = String::new();
                after.push_str(&before[0..start]);
                after.push_str(&rw.1);
                after.push_str(&before[end..]);
                set.insert(after);
            }
        }
        set
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::Problem;

    fn get_example() -> Problem {
        let mut l = "H => HO\nH => OH\nO => HH".lines().map(|s| s.to_owned());
        Problem::from_lines(&mut l)
    }

    #[test]
    fn example() {
        let stuff = get_example().rewrite("HOH");
        assert_eq!(stuff.len(), 4);
        assert!(stuff.contains("HOOH"));
        assert!(stuff.contains("HOHO"));
        assert!(stuff.contains("OHOH"));
        assert!(stuff.contains("HHHH"));
        assert_eq!(get_example().rewrite("HOHOHO").len(), 7);
    }

    #[test]
    fn example_weirdchar() {
        let mut p = Problem::new();
        p.add("H", "OO");
        let stuff = p.rewrite("H2O");
        assert_eq!(stuff.len(), 1);
        assert!(stuff.contains("OO2O"));
    }
}
