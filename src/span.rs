#[derive(Clone, Debug)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub pos: usize,
    pub len: usize,
}

impl Span {
    pub fn union(&self, other: &Span) -> Self {
        Self {
            line: self.line,
            col: self.col,
            pos: self.pos,
            len: (other.pos + other.len) - self.pos,
        }
    }
}
