use std::borrow::Borrow;
use regex::Regex;
use interp::{Reg,Offset,Insn};

pub struct Parser {
    opcode: Regex,
    sep: Regex,
}
impl Parser {
    pub fn new() -> Parser { Parser {
        opcode: Regex::new(r"^\s*(\pL+)\s+(.*)").unwrap(),
        sep: Regex::new(r"\s*,\s*|\s+").unwrap(), // "DWIM"
    }}
    fn parse_reg(&self, tok: &str) -> Reg {
        match tok {
            "a" | "A" => Reg::A,
            "b" | "B" => Reg::B,
            _ => panic!("invalid register {}", tok)
        }
    }
    fn parse_off(&self, tok: &str) -> Offset {
        let tok = if tok.chars().next() == Some('+') { &tok[1..] } else { tok };
        tok.parse().unwrap_or_else(|err| panic!("invalid offset {}: {:?}", tok, err))
    }
    pub fn parse_line(&self, line: &str) -> Insn {
        let o_caps = self.opcode.captures(line).expect("syntax error finding opcode");
        let opc = &o_caps[1];        
        let args: Vec<_> = self.sep.split(o_caps[2].trim()).collect();
        let check = |n| if args.len() != n {
            panic!("too {} arguments for opcode {}: got {}, expected {}",
                   if args.len() > n { "many" } else { "few" }, opc, args.len(), n);
        };
        match opc.to_lowercase().borrow() {
            "hlf" => { check(1); Insn::Hlf(self.parse_reg(args[0])) },
            "tpl" => { check(1); Insn::Tpl(self.parse_reg(args[0])) },
            "inc" => { check(1); Insn::Inc(self.parse_reg(args[0])) },
            "jmp" => { check(1); Insn::Jmp(self.parse_off(args[0])) },
            "jie" => { check(2); Insn::Jie(self.parse_reg(args[0]), self.parse_off(args[1])) },
            "jio" => { check(2); Insn::Jio(self.parse_reg(args[0]), self.parse_off(args[1])) },
            _ => panic!("unrecognized opcode {}", opc)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use interp::{Insn,Reg};

    #[test]
    fn offsets() {
        let p = Parser::new();
        assert_eq!(p.parse_off("23"), 23);
        assert_eq!(p.parse_off("+23"), 23);
        assert_eq!(p.parse_off("-23"), -23);
    }

    #[test] #[should_panic(expected = "invalid offset")]
    fn badnum1() {
        let p = Parser::new();
        let _o = p.parse_off("");
    }

    #[test] #[should_panic(expected = "invalid offset")]
    fn badnum2() {
        let p = Parser::new();
        let _o = p.parse_off("b");
    }
    
    #[test]
    fn example() {
        let p = Parser::new();
        assert_eq!(p.parse_line("inc a"), Insn::Inc(Reg::A));
        assert_eq!(p.parse_line("jio a, +2"), Insn::Jio(Reg::A, 2));
        assert_eq!(p.parse_line("jmp +0"), Insn::Jmp(0));
    }
}
