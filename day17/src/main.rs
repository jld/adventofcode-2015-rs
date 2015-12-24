extern crate util;
mod exhaustive;
mod witnessed;
mod pascal;

use std::io::{stdin,BufRead};
use std::env;

pub type Vol = u16;
pub type Num = u64;

fn main() {
    let mut args = env::args();
    let _ = args.next();
    let target: Vol = args.next().expect("first argument is total eggnog volume").parse().unwrap();
    let stdin = stdin();
    let nums: Vec<Vol> =
        stdin.lock().lines().map(|l| l.expect("I/O error").parse().expect("NaN")).collect();
    let cmd = args.next().unwrap_or("".to_owned());
    if "convolution".starts_with(&cmd) {
        println!("{}", pascal::subset_sum_at(&nums, target));
    } else if "exhaustive".starts_with(&cmd) {
        println!("{}", exhaustive::exhaustive(&nums, target));
    } else if "list".starts_with(&cmd) {
        for way in witnessed::eggnog_iter(&nums, target) {
            println!("{:?}", way);
        }
    } else {
        panic!("Unrecognized command {:?}", cmd);
    }
}
