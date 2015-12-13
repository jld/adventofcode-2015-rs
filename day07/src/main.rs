mod generic;
mod eager;
mod ast;
mod lazy;
mod shells;
mod parse;

use std::io::stdin;
use parse::parse;

pub fn main() {
    let stdin = stdin();
    let stuff = parse(stdin.lock());
    // println!("stuff = {:?}", stuff);
    let thing = shells::eval_lazy(stuff, &["a"]).unwrap()[0];
    println!("{} -> a", thing);
}
