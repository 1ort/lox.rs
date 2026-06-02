#[derive(Clone, Debug)]
pub struct Span {
    pub pos: usize,
    pub len: usize,
}

impl Span {
    pub fn union(&self, other: &Span) -> Self {
        Self {
            pos: self.pos,
            len: (other.pos + other.len) - self.pos,
        }
    }
}
