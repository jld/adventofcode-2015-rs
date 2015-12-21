extern crate aho_corasick;
extern crate util;
mod cyk;

use std::collections::HashSet;
use std::io::{stdin,BufRead};
use aho_corasick::{Automaton,AcAutomaton};
use util::SymTab;
use cyk::CYK;

struct Problem {
    rewrites: Vec<(String, String)>,
}
impl Problem {
    #![allow(dead_code)]
    fn new() -> Self {
        Problem { rewrites: vec![] }
    }
    fn add(&mut self, lhs: &str, rhs: &str) {
        self.rewrites.push((lhs.to_owned(), rhs.to_owned()));
    }
    fn add_lines<I: Iterator<Item=String>>(&mut self, lines: &mut I) {
        for line in lines {
            if line.is_empty() {
                break;
            }
            let arrow_b = line.find(" => ").expect("expected \" => \"");
            let arrow_e = arrow_b + " => ".len();
            self.add(&line[..arrow_b], &line[arrow_e..]);
        }
    }
    fn from_lines<I: Iterator<Item=String>>(lines: &mut I) -> Self {
        let mut that = Self::new();
        that.add_lines(lines);
        that
    }
    fn rewrite(&self, before: &str) -> HashSet<String> {
        let mut set = HashSet::new();
        self.rewrite_into(before, &mut set);
        set
    }
    fn rewrite_all(&self, befores: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for before in befores {
            self.rewrite_into(before, &mut set);
        }
        set
    }
    fn rewrite_into(&self, before: &str, set: &mut HashSet<String>) {
        for rw in self.rewrites.iter() {
            let mut cursor = 0;
            while let Some(offset) = before[cursor..].find(&rw.0) {
                let start = cursor + offset;
                let end = start + rw.0.len();
                cursor = start + 1;
                
                let mut after = String::new();
                after.push_str(&before[..start]);
                after.push_str(&rw.1);
                after.push_str(&before[end..]);
                set.insert(after);
            }
        }
    }
    fn search(&self, before: &str, after: &str) -> usize {
        let mut stuff = HashSet::new();
        stuff.insert(before.to_owned());
        for len in 0.. {
            if stuff.contains(after) {
                return len;
            }
            let oldstuff = stuff;
            stuff = self.rewrite_all(&oldstuff);
        }
        unreachable!()
    }
    fn invert(&self) -> Self {
        Problem { rewrites: self.rewrites.iter().cloned().map(|(l,r)| (r,l)).collect() }
    }
    fn search_fast(&self, before: &str, after: &str) -> Option<usize> {
        let mut stab = SymTab::new();
        for &(ref lhs, _) in &self.rewrites {
            if lhs != before {
                let _ = stab.read(lhs);
            }
        }
        let stab = stab; // freeze
        let syms = AcAutomaton::new(stab.pborrow());
        let parse = |s: &str| {
            let mut acc = Vec::new();
            let mut expected = 0;
            for found in syms.find(s) {
                assert!(found.start == expected,
                        "Unparseable thing: {:?}", &s[expected..found.start]);
                acc.push(found.pati);
                expected = found.end;
            }
            acc
        };
        let mut cyk = CYK::new(stab.len());
        let mut starts = Vec::new();
        for &(ref lhs, ref rhs) in &self.rewrites {
            let prhs = parse(rhs);
            if lhs == before {
                assert!(prhs.len() == 1, "start symbol must have only unit productions");
                starts.push(prhs[0]);
            } else {
                let plhs = stab.try_read(lhs).unwrap();
                cyk.add_rule(plhs, &prhs);
            }
        }
        let target = parse(after);
        cyk.solve(&starts, &target).map(|u| /* compensate for initial unit prod. */ u + 1)
    }
}

fn main() {
    let stdin = stdin();
    let mut inline = stdin.lock().lines().map(|l| l.expect("I/O error"));
    let prob = Problem::from_lines(&mut inline);
    let input = inline.next().expect("expected target string after blank line");
    println!("Calibration: {}", prob.rewrite(&input).len());
    println!("Path length (fast): {:?}", prob.search_fast("e", &input));
    println!("Path length (fwd): {}", prob.search("e", &input));
    println!("Path length (inv): {}", prob.search(&input, "e"));
}

#[cfg(test)]
mod tests {
    use super::Problem;

    fn get_example() -> Problem {
        let mut l = "H => HO\nH => OH\nO => HH".lines().map(|s| s.to_owned());
        Problem::from_lines(&mut l)
    }

    fn get_example2() -> Problem {
        let mut p = get_example();
        let mut l = "e => H\ne => O".lines().map(|s| s.to_owned());
        p.add_lines(&mut l);
        p
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

    #[test]
    fn example_path() {
        let p = get_example2();
        assert_eq!(p.search("e", "HOH"), 3);
        assert_eq!(p.search("e", "HOHOHO"), 6);
    }

    #[test]
    fn example_inv() {
        let p = get_example().invert();
        for s in &["HOOH", "HOHO", "OHOH", "HHHH"] {
            assert!(p.rewrite(s).contains("HOH"));
        }
    }

    #[test]
    fn path_inv() {
        let p = get_example2().invert();
        assert_eq!(p.search("HOH", "e"), 3);
        assert_eq!(p.search("HOHOHO", "e"), 6);
    }

    #[test]
    fn path_fast() {
        let p = get_example2();
        assert_eq!(p.search_fast("e", "HOH"), Some(3));
        assert_eq!(p.search_fast("e", "HOHOHO"), Some(6));
        assert_eq!(p.search_fast("e", "OOO"), None);
    }
}
