use crate::Token;

use std::iter::Peekable;
use std::str::CharIndices;

pub fn tokenize_iter<'a>(input: &'a str) -> Lexer<'a> {
    Lexer::new(input)
}

pub struct Lexer<'a> {
    input: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.char_indices().peekable(),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(usize, Token), usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let (byte_offset, c) = loop {
            let next @ (_, c) = match self.input.next() {
                None => return None,
                Some(next) => next,
            };

            if c.is_ascii_whitespace() {
                continue;
            }

            break next;
        };

        match c {
            '/' => {
                match self.input.next() {
                    Some((mut end_offset, '/')) => {
                        // Skip until newline.
                        for (byte_offset_, c_) in self.input.by_ref() {
                            end_offset = byte_offset_;
                            if c_ == '\n' {
                                return Some(Ok((
                                    byte_offset,
                                    Token::Comment {
                                        size_in_bytes: end_offset - byte_offset + 1,
                                    },
                                )));
                            }
                        }

                        // End of input.
                        Some(Ok((
                            byte_offset,
                            Token::Comment {
                                size_in_bytes: end_offset - byte_offset + 1,
                            },
                        )))
                    }
                    _ => Some(Err(byte_offset + 1)),
                }
            }

            '"' => {
                let mut last_byte_offset = byte_offset;
                for (byte_offset_, c_) in self.input.by_ref() {
                    if c_ == '"' {
                        return Some(Ok((
                            byte_offset + 1,
                            Token::Str {
                                size_in_bytes: byte_offset_ - byte_offset - 1,
                            },
                        )));
                    }

                    last_byte_offset = byte_offset_;
                }

                // Unterminated string.
                Some(Err(last_byte_offset))
            }

            't' => {
                if self.next_char() == Some('r')
                    && self.next_char() == Some('u')
                    && self.next_char() == Some('e')
                {
                    Some(Ok((byte_offset, Token::True)))
                } else {
                    Some(Err(byte_offset))
                }
            }

            'f' => {
                if self.next_char() == Some('a')
                    && self.next_char() == Some('l')
                    && self.next_char() == Some('s')
                    && self.next_char() == Some('e')
                {
                    Some(Ok((byte_offset, Token::False)))
                } else {
                    Some(Err(byte_offset))
                }
            }

            'n' => {
                if self.next_char() == Some('u')
                    && self.next_char() == Some('l')
                    && self.next_char() == Some('l')
                {
                    Some(Ok((byte_offset, Token::Null)))
                } else {
                    Some(Err(byte_offset))
                }
            }

            c if c.is_ascii_digit() => {
                let mut i: u64 = u64::from((c as u8) - b'0');

                while let Some((_, next)) = self.input.peek().copied() {
                    if !next.is_ascii_digit() {
                        break;
                    }

                    // Consume the digit.
                    self.input.next();

                    // Ignore overflows for the purposes of this post.
                    i *= 10;
                    i += u64::from((next as u8) - b'0');
                }

                Some(Ok((byte_offset, Token::Int(i))))
            }

            ',' => Some(Ok((byte_offset, Token::Comma))),

            ':' => Some(Ok((byte_offset, Token::Colon))),

            '[' => Some(Ok((byte_offset, Token::LBracket))),

            ']' => Some(Ok((byte_offset, Token::RBracket))),

            '{' => Some(Ok((byte_offset, Token::LBrace))),

            '}' => Some(Ok((byte_offset, Token::RBrace))),

            _ => Some(Err(byte_offset)),
        }
    }
}

impl<'a> Lexer<'a> {
    fn next_char(&mut self) -> Option<char> {
        self.input.next().map(|(_, c)| c)
    }
}

#[cfg(test)]
fn tokenize(input: &str) -> Vec<(usize, Token)> {
    Lexer::new(input).map(|t| t.unwrap()).collect()
}

#[test]
fn test_keywords() {
    assert_eq!(
        tokenize("true false null"),
        vec![(0, Token::True), (5, Token::False), (11, Token::Null)]
    );
}

#[test]
fn test_delimiters() {
    assert_eq!(
        tokenize("{ } [ ]"),
        vec![
            (0, Token::LBrace),
            (2, Token::RBrace),
            (4, Token::LBracket),
            (6, Token::RBracket)
        ]
    );
}

#[test]
fn test_comments() {
    assert_eq!(
        tokenize("//"),
        vec![(0, Token::Comment { size_in_bytes: 2 })]
    );

    assert_eq!(
        tokenize("//\n"),
        vec![(0, Token::Comment { size_in_bytes: 3 })]
    );

    assert_eq!(
        tokenize("// asdf"),
        vec![(0, Token::Comment { size_in_bytes: 7 })]
    );

    assert_eq!(
        tokenize("// asdf\n"),
        vec![(0, Token::Comment { size_in_bytes: 8 })]
    );
}

#[test]
fn test_strings() {
    assert_eq!(
        tokenize(r#""""#),
        vec![(1, Token::Str { size_in_bytes: 0 })]
    );

    assert_eq!(
        tokenize(r#""a""#),
        vec![(1, Token::Str { size_in_bytes: 1 })]
    );
}
