extern crate regex;
mod parser;
mod interp;

use std::convert::Into;
use std::io::{stdin,BufRead};
use parser::Parser;
use interp::Reg;

fn main() {
    let stdin = stdin();
    let p = Parser::new();
    let prog: Vec<_> =
        stdin.lock().lines().map(|rl| p.parse_line(&rl.expect("I/O error"))).collect();
    let output0 = interp::run(&prog, [].into());
    println!("[0] A = {}", output0[Reg::A]);
    println!("[0] B = {}", output0[Reg::B]);
    let output1 = interp::run(&prog, [(Reg::A, 1)].into());
    println!("[1] A = {}", output1[Reg::A]);
    println!("[1] B = {}", output1[Reg::B]);
}
