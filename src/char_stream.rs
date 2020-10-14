use crate::{source_file::SourceFile, span::Pos};

use std::rc::Rc;

pub struct CharStream {
    content: Rc<[char]>,
    file_path: Rc<str>,

    idx: usize,
    byte_pos: usize,
    lineno: usize,
    column: usize,
}

impl CharStream {
    pub fn new(src: SourceFile) -> Self {
        Self {
            content: src.content,
            file_path: src.file_path,

            idx: 0,
            byte_pos: 0,
            lineno: 1,
            column: 0,
        }
    }

    pub fn cur(&self) -> char {
        if self.idx == 0 {
            panic!("no current char")
        }
        self.content[self.idx - 1]
    }

    pub fn peek(&self) -> Option<char> {
        self.content.get(self.idx).copied()
    }

    pub fn next_char(&mut self) -> Option<char> {
        if self.idx > 0 {
            let cur = self.content.get(self.idx - 1).copied()?;
            let char_len = cur.len_utf8();
            self.byte_pos += char_len;
            if cur == '\n' {
                self.lineno += 1;
                self.column = 0;
            }
        }
        self.column += 1;
        let ch = self.content.get(self.idx).copied()?;
        self.idx += 1;
        Some(ch)
    }

    pub fn consume1(&mut self) -> char {
        self.next().unwrap()
    }

    pub fn pos(&self) -> Pos {
        Pos {
            byte_pos: self.byte_pos,
            lineno: self.lineno,
            column: self.column,
        }
    }

    pub fn file_path(&self) -> &Rc<str> {
        &self.file_path
    }
}

impl Iterator for CharStream {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        self.next_char()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_char_stream(content: &str) -> CharStream {
        CharStream::new(SourceFile::new(content, "<dummy file>"))
    }

    #[test]
    fn char_stream() {
        {
            let mut chars = dummy_char_stream("");
            assert_eq!(chars.next(), None)
        }
        {
            let mut chars = dummy_char_stream("12");
            assert_eq!(chars.peek(), Some('1'));
            assert_eq!(chars.next(), Some('1'));
            assert_eq!(chars.cur(), '1');
            assert_eq!(chars.pos(), Pos::new(0, 1, 1));

            assert_eq!(chars.peek(), Some('2'));
            assert_eq!(chars.next(), Some('2'));
            assert_eq!(chars.cur(), '2');
            assert_eq!(chars.pos(), Pos::new(1, 1, 2));

            assert_eq!(chars.next(), None);
            assert_eq!(chars.pos(), Pos::new(2, 1, 3));
        }
        {
            let mut chars = dummy_char_stream("1\n2");
            assert_eq!(chars.peek(), Some('1'));
            assert_eq!(chars.next(), Some('1'));
            assert_eq!(chars.cur(), '1');
            assert_eq!(chars.pos(), Pos::new(0, 1, 1));

            assert_eq!(chars.peek(), Some('\n'));
            assert_eq!(chars.next(), Some('\n'));
            assert_eq!(chars.cur(), '\n');
            assert_eq!(chars.pos(), Pos::new(1, 1, 2));

            assert_eq!(chars.peek(), Some('2'));
            assert_eq!(chars.next(), Some('2'));
            assert_eq!(chars.cur(), '2');
            assert_eq!(chars.pos(), Pos::new(2, 2, 1));

            assert_eq!(chars.next(), None);
            assert_eq!(chars.pos(), Pos::new(3, 2, 2));
        }
        {
            let mut chars = dummy_char_stream("好，很有精神");
            assert_eq!(chars.peek(), Some('好'));
            assert_eq!(chars.next(), Some('好'));
            assert_eq!(chars.cur(), '好');
            assert_eq!(chars.pos(), Pos::new(0, 1, 1));

            assert_eq!(chars.peek(), Some('，'));
            assert_eq!(chars.next(), Some('，'));
            assert_eq!(chars.cur(), '，');
            assert_eq!(chars.pos(), Pos::new(3, 1, 2));

            assert_eq!(chars.next(), Some('很'));
            assert_eq!(chars.pos(), Pos::new(6, 1, 3));
        }
    }
}
