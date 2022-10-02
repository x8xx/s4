pub enum MatchKind<'a> {
    Lpm(&'a [u8], u8),
    Exact(&'a [u8]),
    Ternary(&'a [u8]),
}


impl<'a> MatchKind<'a> {
    pub fn is_match(&self, kind: &MatchKind) -> bool {
        true
    }
}


pub struct Table<'a> {
    entries: Vec<Vec<MatchKind<'a>>>,
}


impl<'a> Table<'a> {
    pub fn new() -> Self {
        Table {
            entries: Vec::new(),
        }
    }

    pub fn push_entry() {

    }

    pub fn search() {

    }
}
