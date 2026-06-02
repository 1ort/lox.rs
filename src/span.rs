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

impl From<Span> for std::ops::Range<usize> {
    fn from(value: Span) -> Self {
        Self {
            start: value.pos,
            end: value.pos + value.len,
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            len: Default::default(),
        }
    }
}
