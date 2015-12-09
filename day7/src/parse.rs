use ast::{Signal,Gate,Shift};
use generic::ExprMap;
use shells::Insn;
use std::io::BufRead;
use std::str::FromStr;

fn wrangle_id(s: &str) -> Result<String, Signal> {
    match Signal::from_str(&s) {
        Ok(imm) => Err(imm),
        Err(_) => Ok(s.to_owned())
    }
}

fn mkshift(lnum: usize, s: &str) -> Shift {
    Shift::new(u8::from_str(s).unwrap())
        .unwrap_or_else(|| panic!("line {}: shift count out of range", lnum))
}

pub fn parse<B: BufRead>(input: B) -> Vec<Insn> {
    let mut insns = Vec::new();
    for (lnum, line) in input.lines().enumerate() {
        let line = line.expect("I/O error");
        let mut words: Vec<_> = line
            .split(char::is_whitespace)
            .filter(|s| s.len() > 0)
            .collect();
        assert!(words.len() >= 3, "line {}: too few tokens", lnum);
        assert!(words.len() <= 5, "line {}: too many tokens", lnum);
        let dst = words.pop().unwrap().to_owned();
        let arrow = words.pop().unwrap();
        assert!(arrow == "->", "line {}: token {:?} is not an arrow", lnum, arrow);
        let expr: Gate<Result<String, Signal>> = match words.len() {
            1 => match wrangle_id(words[0]) {
                Err(imm) => Gate::Imm(imm),
                Ok(id) => Gate::Or(Ok(id.clone()), Ok(id)),
            },
            2 => match words[0] {
                "NOT" => Gate::Not(wrangle_id(words[1])),
                huh => panic!("line {}: bad unary operator {:?}", lnum, huh)
            },
            3 => match words[1] {
                "AND" => Gate::And(wrangle_id(words[0]), wrangle_id(words[2])),
                "OR" => Gate::Or(wrangle_id(words[0]), wrangle_id(words[2])),
                "LSHIFT" => Gate::LShift(wrangle_id(words[0]), mkshift(lnum, words[2])),
                "RSHIFT" => Gate::RShift(wrangle_id(words[0]), mkshift(lnum, words[2])),
                huh => panic!("line {}: bad binary operator {:?}", lnum, huh)
            },
            _ => unreachable!()
        };
        let mut fixups: usize = 0;
        // Let's pretend this justifies making ExprMap so excitingly generic:
        let gate: Gate<String> = expr.idmap(|r| match *r {
            Ok(ref id) => id.clone(),
            Err(imm) => {
                let id = format!(" __lit{}", fixups);
                fixups += 1;
                insns.push((Gate::Imm(imm), id.clone()));
                id
            }
        });
        insns.push((gate, dst));
    }
    insns
}

#[cfg(test)]
mod test {
    use super::parse;
    use ast::Gate;

    fn s(s: &str) -> String { s.to_owned() }

    #[test]
    fn examples() {
        assert_eq!(parse("123 -> x\n".as_bytes()),
                   vec![(Gate::Imm(123), s("x"))]);
        assert_eq!(parse("x AND y -> z\n".as_bytes()),
                   vec![(Gate::And(s("x"), s("y")), s("z"))]);
        assert_eq!(parse("x OR y -> z\n".as_bytes()),
                   vec![(Gate::Or(s("x"), s("y")), s("z"))]);
        assert_eq!(parse("p LSHIFT 2 -> q\n".as_bytes()),
                   vec![(Gate::lshift(s("p"), 2), s("q"))]);
        assert_eq!(parse("p RSHIFT 2 -> q\n".as_bytes()),
                   vec![(Gate::rshift(s("p"), 2), s("q"))]);
        assert_eq!(parse("NOT e -> f\n".as_bytes()),
                   vec![(Gate::Not(s("e")), s("f"))]);
    }

    #[test]
    fn lolhax() {
        assert_eq!(parse("x -> y".as_bytes()),
                   vec![(Gate::Or(s("x"), s("x")), s("y"))]);
        assert_eq!(parse("1 AND p -> q".as_bytes()),
                   vec![(Gate::Imm(1), s(" __lit0")),
                        (Gate::And(s(" __lit0"), s("p")), s("q"))]);
    }

    // I could test the error cases... but do I even care at this point?
}
