extern crate rustc_serialize;
use rustc_serialize::json::Json;
use std::io::{stdin,Read};
use std::i64;

fn json_seq_int_sum<'j, I: Iterator<Item = &'j Json>>(i: I) -> i64 {
    i.map(json_int_sum).fold(0, |s, n| s.checked_add(n).expect("overflow"))
}

fn json_int_sum(j: &Json) -> i64 {
    match *j {
        Json::I64(i) => i,
        Json::U64(u) if u <= i64::MAX as u64 => u as i64,
        Json::String(_) => 0,
        Json::Array(ref a) => json_seq_int_sum(a.iter()),
        Json::Object(ref o) => json_seq_int_sum(o.values()),
        _ => panic!("Unhandled JSON thing {:?}", j)
    }
}

fn is_red(j: &Json) -> bool {
    match *j {
        Json::String(ref s) => s == "red",
        _ => false
    }
}

fn elf_correction(j: &Json) -> Json {
    match *j {
        Json::Object(ref o) => if o.values().any(is_red) {
            Json::String("elf fail redacted".to_owned())
        } else {
            Json::Object(o.iter().map(|(k, v)| (k.clone(), elf_correction(v))).collect())
        },
        Json::Array(ref a) =>
            Json::Array(a.iter().map(elf_correction).collect()),
        _ => j.clone()
    }
}

fn main() {
    let stdin = stdin();
    let mut input = String::new();
    stdin.lock().read_to_string(&mut input).unwrap();
    let j = Json::from_str(&input).unwrap();
    println!("Sum of numbers: {}", json_int_sum(&j));
    let ej = elf_correction(&j);
    println!("With elf correction: {}", json_int_sum(&ej));
}

#[cfg(test)]
mod test {
    use super::{json_int_sum, elf_correction};
    use rustc_serialize::json::Json;

    fn plain_sum(s: &str) -> i64 {
        json_int_sum(&Json::from_str(s).unwrap())
    }
    fn fixed_sum(s: &str) -> i64 {
        json_int_sum(&elf_correction(&Json::from_str(s).unwrap()))
    }

    #[test]
    fn examples() {
        assert_eq!(plain_sum("[1,2,3]"), 6);
        assert_eq!(plain_sum("{\"a\":2,\"b\":4}"), 6);
        assert_eq!(plain_sum("[[[3]]]"), 3);
        assert_eq!(plain_sum("{\"a\":{\"b\":4},\"c\":-1}"), 3);
        assert_eq!(plain_sum("{\"a\":[-1,1]}"), 0);
        assert_eq!(plain_sum("[-1,{\"a\":1}]"), 0);
        assert_eq!(plain_sum("[]"), 0);
        assert_eq!(plain_sum("{}"), 0);
    }

    #[test] #[should_panic(expected = "Unhandled JSON thing")]
    fn float1() {
        let _ = plain_sum("[1,2.3]");
    }
    #[test] #[should_panic(expected = "Unhandled JSON thing")]
    fn float2() {
        let _ = plain_sum("[1,2.0]");
    }

    #[test]
    fn bignum() {
        assert_eq!(plain_sum("[4294967296]"), 4294967296);
        assert_eq!(plain_sum("[-9223372036854775808]"), -9223372036854775808);
        assert_eq!(plain_sum("[9223372036854775807]"), 9223372036854775807);
        assert_eq!(plain_sum("[7, 9223372036854775800]"), 9223372036854775807);
    }

    #[test] #[should_panic(expected = "Unhandled JSON thing")]
    fn toobignum() {
        let _ = plain_sum("[9223372036854775808]");
    }
    #[test] #[should_panic(expected = "overflow")]
    fn toobigsum() {
        let _ = plain_sum("[8, 9223372036854775800]");
    }

    #[test]
    fn part2() {
        assert_eq!(fixed_sum("[1,2,3]"), 6);
        assert_eq!(fixed_sum("[1,{\"c\":\"red\",\"b\":2},3]"), 4);
        assert_eq!(fixed_sum("{\"d\":\"red\",\"e\":[1,2,3,4],\"f\":5}"), 0);
        assert_eq!(fixed_sum("[1,\"red\",5]"), 6);
    }
}
