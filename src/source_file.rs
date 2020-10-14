use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub content: Rc<[char]>,
    pub file_path: Rc<str>,
}

impl SourceFile {
    pub fn new(content: &str, file_path: &str) -> Self {
        let content = content.chars().collect::<Vec<char>>().into();
        let file_path = file_path.into();
        Self { content, file_path }
    }

    pub fn generate_lines(&self) -> Vec<Vec<char>> {
        let mut chars = self.content.iter().copied().peekable();
        let mut lines: Vec<Vec<char>> = Vec::new();

        let mut line: Option<Vec<char>> = None;

        loop {
            match chars.next() {
                None => break,
                Some(ch) => match ch {
                    '\n' => {
                        lines.push(vec![]);
                        line = Some(Vec::new());
                    }
                    _ => {
                        let mut buf = vec![ch];
                        line = None;
                        for ch in &mut chars {
                            match ch {
                                '\n' => {
                                    line = Some(Vec::new());
                                    break;
                                }
                                _ => buf.push(ch),
                            }
                        }
                        lines.push(buf);
                    }
                },
            };
        }

        if let Some(line) = line {
            lines.push(line)
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_source_file(content: &str) -> SourceFile {
        SourceFile::new(content, "<dummy file>")
    }

    #[test]
    fn lines() {
        {
            let lhs = dummy_source_file("").generate_lines();
            let rhs: Vec<Vec<char>> = vec![];
            assert_eq!(lhs, rhs);
        }
        {
            let lhs = dummy_source_file("a").generate_lines();
            let rhs: Vec<Vec<char>> = vec![vec!['a']];
            assert_eq!(lhs, rhs);
        }
        {
            let lhs = dummy_source_file("a\nb\n").generate_lines();
            let rhs: Vec<Vec<char>> = vec![vec!['a'], vec!['b'], vec![]];
            assert_eq!(lhs, rhs);
        }
        {
            let lhs = dummy_source_file("a\nb\nc").generate_lines();
            let rhs: Vec<Vec<char>> = vec![vec!['a'], vec!['b'], vec!['c']];
            assert_eq!(lhs, rhs);
        }
        {
            let lhs = dummy_source_file("aa\nbb\ncc").generate_lines();
            let rhs: Vec<Vec<char>> = vec![vec!['a', 'a'], vec!['b', 'b'], vec!['c', 'c']];
            assert_eq!(lhs, rhs);
        }
    }
}
