use std::ops::Range;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub byte_range: Range<usize>,
    pub lc_range: Range<LineColumn>,
    pub file_path: Rc<str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub byte_pos: usize,
    pub lineno: usize,
    pub column: usize,
}

impl Pos {
    pub fn new(byte_pos: usize, lineno: usize, column: usize) -> Self {
        Pos {
            byte_pos,
            lineno,
            column,
        }
    }

    pub fn add1(self) -> Self {
        Self {
            byte_pos: self.byte_pos + 1,
            lineno: self.lineno,
            column: self.column + 1,
        }
    }
}
