mod exhaustive;
mod witnessed;
mod pascal;

use std::io::{stdin,BufRead};

pub type Vol = u8;
pub type Num = u64;

fn main() {
    let stdin = stdin();
    let nums: Vec<Vol> =
        stdin.lock().lines().map(|l| l.expect("I/O error").parse().expect("NaN")).collect();
    println!("Hello, world! {:?}", nums);
}
