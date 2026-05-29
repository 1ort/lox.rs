#[derive(Clone, Debug)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub pos: usize,
    pub len: usize,
}
