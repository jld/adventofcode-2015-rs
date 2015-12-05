use std::env;
use std::io;
use std::io::{Write, stdin, BufRead, BufReader};
use std::mem::drop;
use std::process;
use std::thread;

static RULES: &'static [&'static str] = &[
    "\\([a-z][a-z]\\).*\\1",
    "\\([a-z]\\).\\1",
];

fn grep_one_line(needle: &str, haystack: &str) -> bool {
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

fn compute_slow(s: &str) -> bool {
    RULES.iter().all(|re| grep_one_line(re, s))
}

struct Pipeline {
    procs: Vec<process::Child>,
    threads: Vec<thread::JoinHandle<()>>,
    stdin: Option<process::ChildStdin>,
    stdout: Option<BufReader<process::ChildStdout>>,
}
impl Pipeline {
    fn join(self) {
        drop(self.stdin);
        drop(self.stdout);
        for p in self.procs {
            let mut p = p;
            let _ = p.wait().unwrap();
        }
        for t in self.threads {
            t.join().unwrap();
        }
    }
}
fn grep(needle: &str) -> Pipeline {
    let mut cld = process::Command::new("grep")
        .arg(needle)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap();
    let cin = cld.stdin.take().unwrap();
    let cout = cld.stdout.take().unwrap();
    Pipeline {
        procs: vec![cld],
        threads: vec![],
        stdin: Some(cin),
        stdout: Some(BufReader::new(cout)),
    }
}
fn pipe(lhs: Pipeline, rhs: Pipeline) -> Pipeline {
    let mut rv = Pipeline {
        procs: lhs.procs,
        threads: lhs.threads,
        stdin: lhs.stdin,
        stdout: rhs.stdout,
    };
    let mut rprocs = rhs.procs;
    let mut rthreads = rhs.threads;
    let rstdin = rhs.stdin.expect("can't pipe when downstream input already consumed");
    let lstdout = lhs.stdout.expect("can't pipe when upstream output already consumed");
    rv.procs.append(&mut rprocs);
    let th = thread::spawn(move || cat(lstdout, rstdin));
    rv.threads.push(th);
    rv.threads.append(&mut rthreads);
    rv
}
fn cat<R: BufRead, W: Write>(rd: R, wr: W) {
    // Yes, it *would* be nice to use actual pipes here.  But there's
    // no public/stable interface for that yet, and this is an excuse
    // to learn more about reader/writer traits.
    let mut rd = rd;
    let mut wr = wr;
    loop {
        let n = {
            let buf = rd.fill_buf().expect("read error in pipeline");
            if buf.len() == 0 {
                break;
            }
            wr.write(buf).expect("write error in pipeline")
        };
        rd.consume(n);
    }
}

fn santa() -> Pipeline {
    // This way runs in 0.45x the wall time and 0.75x the CPU time as
    // piping them the other way around.  Not too surprising, given
    // the large state space for the backreference in `[0]`.
    pipe(grep(RULES[1]), grep(RULES[0]))
}

fn print_count<I: Iterator<Item = io::Result<String>>>(input: I) {
    let count = input.map(|line| line.expect("read error while counting")).count();
    println!("{} string{} nice.", count, if count == 1 { " is" } else { "s are" });
}
fn main_slow() {
    let stdin = stdin();
    print_count(stdin.lock().lines().filter(|rline| {
        rline.as_ref().map(|l| compute_slow(l)).unwrap_or(true)
    }));
}
fn main_fast() {
    let mut tubes = santa();
    let wr = tubes.stdin.take().unwrap();
    tubes.threads.push(thread::spawn(move || {
        let stdin = stdin();
        cat(stdin.lock(), wr);
    }));
    print_count(tubes.stdout.take().unwrap().lines());
    tubes.join();
}

pub fn main() {
    let argv1 = env::args().nth(1);
    match argv1.as_ref().map(|s| s as &str /* Sigh. */).unwrap_or("fast") {
        "slow" => main_slow(),
        "fast" => main_fast(),
        huh => panic!("unknown command {}", huh)
    }
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};
    use super::{RULES, grep_one_line, compute_slow, santa, Pipeline};

    macro_rules! case {
        ($s:expr => $b:expr) => { assert_eq!(compute_slow($s), $b); };
        ([$i:expr] $s:expr => $b:expr) => { assert_eq!(grep_one_line(RULES[$i], $s), $b); };
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

    fn is_echoed(p: Pipeline, s: &str) -> bool {
        let mut p = p;
        let mut s = s.to_owned();
        s.push('\n');
        {
            let mut pin = p.stdin.take().unwrap();
            // There's more than enough buffering that this doesn't
            // need to be async:
            pin.write_all(s.as_bytes()).unwrap();
        }
        let mut out = String::new();
        let _n = p.stdout.as_mut().unwrap().read_to_string(&mut out).unwrap();
        p.join();
        if out.len() == 0 {
            return false;
        }
        assert_eq!(s, out);
        return true;
    }

    #[test]
    fn fidelity() {
        static ALL_STRS: &'static [&'static str] = &[
            "xyxy", "aabcdefgaa", "aaa", "xyx", "abcdefeghi",
            "qjhvhtzxzqqjkmpb", "xxyxx", "uurcxstgmygtbstg", "ieodomkazucvgmuy",
        ];
        for s in ALL_STRS {
            assert_eq!(is_echoed(santa(), s), compute_slow(s));
        }
    }

    // TODO: test the pipeline stuff by running more than one string
    // through it at a time.  I've done this manually, but more
    // automation would be nice.
}
