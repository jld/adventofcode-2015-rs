use std::collections::HashMap;

pub type Id = usize;

#[derive(Debug, Clone)]
pub struct SymTab {
    rname: HashMap<String, Id>,
    pname: Vec<String>,
}
impl SymTab {
    pub fn new() -> SymTab { SymTab {
        rname: HashMap::new(),
        pname: Vec::new(),
    }}
    pub fn read(&mut self, s: &str) -> Id {
        // This clones the string unnecessarily if it's already
        // present.  Using `contains_key` and `insert` unecessarily
        // hashes the string twice if it's not.  Such is life.
        let entry = self.rname.entry(s.to_owned());
        let pname = &mut self.pname;
        *entry.or_insert_with(|| {
            let n = pname.len();
            pname.push(s.to_owned());
            n
        })
    }
    pub fn print(&self, n: Id) -> String {
        self.pname[n].clone()
    }
    pub fn len(&self) -> usize {
        self.pname.len()
    }
}

#[cfg(test)]
mod tests {
    use super::SymTab;

    #[test]
    fn repeated_read() {
        let mut st = SymTab::new();
        let c0 = st.read("Word");
        let n0 = st.read("Law");
        let n1 = st.read("Law");
        let c1 = st.read("Word");
        assert_eq!(c0, c1);
        assert_eq!(n0, n1);
    }

    #[test]
    fn read_print() {
        let mut st = SymTab::new();
        let c0 = st.read("Word");
        let n0 = st.read("Law");
        assert_eq!(st.print(c0), "Word");
        assert_eq!(st.print(n0), "Law");
    }
}
