use crate::span::Span;

#[derive(Debug)]
pub struct SynError {
    pub span: Span,
    pub msg: String,
}
