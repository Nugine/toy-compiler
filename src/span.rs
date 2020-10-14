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

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub byte_pos: usize,
    pub lc_pos: LineColumn,
}

impl Pos {
    pub fn add1(self) -> Self {
        Self {
            byte_pos: self.byte_pos + 1,
            lc_pos: LineColumn {
                line: self.lc_pos.line,
                column: self.lc_pos.column + 1,
            },
        }
    }
}
