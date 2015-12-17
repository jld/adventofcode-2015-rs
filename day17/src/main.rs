use std::io::{stdin,BufRead};

type Num = u64;

fn main() {
    let stdin = stdin();
    let nums: Vec<Num> =
        stdin.lock().lines().map(|l| l.expect("I/O error").parse().expect("NaN")).collect();
    println!("Hello, world! {:?}", nums);
}
