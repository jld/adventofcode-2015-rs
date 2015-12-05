use std::io::Write;
use std::process;

static RULES: &'static [&'static str] = &[
    "\\([a-z][a-z]\\).*\\1",
    "\\([a-z]\\).\\1",
];

fn run_grep(needle: &str, haystack: &str) -> bool {
    let mut cld = process::Command::new("grep")
        .arg(needle)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::null())
        .spawn()
        .unwrap();
    {
        let cin = cld.stdin.as_mut().unwrap();
        cin.write_all(haystack.as_bytes()).unwrap();
        cin.write_all("\n".as_bytes()).unwrap();
    }
    cld.wait().unwrap().success()
}

fn compute_serial(s: &str) -> bool {
    RULES.iter().all(|re| run_grep(re, s))
}

#[cfg(test)]
mod test {
    use super::{RULES, run_grep, compute_serial};

    macro_rules! case {
        ($s:expr => $b:expr) => { assert_eq!(compute_serial($s), $b); };
        ([$i:expr] $s:expr => $b:expr) => { assert_eq!(run_grep(RULES[$i], $s), $b); };
    }
    
    #[test]
    fn spec_rule1() {
        case!([0] "xyxy" => true);
        case!([0] "aabcdefgaa" => true);
        case!([0] "aaa" => false);
    }

    #[test]
    fn spec_rule2() {
        case!([1] "xyx" => true);
        case!([1] "abcdefeghi" => true);
        case!([1] "aaa" => true);
    }

    #[test]
    fn spec_ex1() {
        case!("qjhvhtzxzqqjkmpb" => true);
    }
    #[test]
    fn spec_ex2() {
        case!("xxyxx" => true);
    }
    #[test]
    fn spec_ex3() {
        case!("uurcxstgmygtbstg" => false);
        case!([0] "uurcxstgmygtbstg" => true);
    }
    #[test]
    fn spec_ex4() {
        case!("ieodomkazucvgmuy" => false);
        case!([1] "ieodomkazucvgmuy" => true);
    }
}
