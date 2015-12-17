extern crate regex;
mod parser;
mod sue;

use std::io::{stdin,BufRead};
use sue::UrSue;
use parser::Parser;

fn main() {
    let p = Parser::new();
    let stdin = stdin();
    let ur_sue = UrSue::the();
    for line in stdin.lock().lines() {
        let line = line.expect("I/O error");
        let (sue1, sue2) = p.parse(&line).expect("Syntax error");
        if sue1.test(&ur_sue) {
            println!("Sue for Part 1: {}", sue1.ident);
        }
        if sue2.test(&ur_sue) {
            println!("Sue for Part 2: {}", sue2.ident);
        }
    }
}
