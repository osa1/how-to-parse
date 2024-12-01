#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Int(u64),
    Str { size_in_bytes: usize },
    True,
    False,
    Null,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Comment { size_in_bytes: usize },
}
