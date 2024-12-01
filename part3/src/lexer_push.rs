use crate::token::Token;

pub trait LexerEventListener {
    fn handle_int(&mut self, byte_offset: usize, i: u64);

    fn handle_str(&mut self, byte_offset: usize, size_in_bytes: usize);

    fn handle_true(&mut self, byte_offset: usize);

    fn handle_false(&mut self, byte_offset: usize);

    fn handle_null(&mut self, byte_offset: usize);

    fn handle_lbracket(&mut self, byte_offset: usize);

    fn handle_rbracket(&mut self, byte_offset: usize);

    fn handle_lbrace(&mut self, byte_offset: usize);

    fn handle_rbrace(&mut self, byte_offset: usize);

    fn handle_colon(&mut self, byte_offset: usize);

    fn handle_comma(&mut self, byte_offset: usize);

    fn handle_comment(&mut self, byte_offset: usize, size_in_bytes: usize);

    fn handle_error(&mut self, byte_offset: usize);
}

// TODO: Skip trivia everywhere.
pub fn tokenize_push<L: LexerEventListener>(input: &str, listener: &mut L) {
    let mut input = input.char_indices().peekable();

    'outer: while let Some((byte_offset, c)) = input.next() {
        if c.is_ascii_whitespace() {
            continue;
        }

        match c {
            '/' => {
                match input.next() {
                    Some((mut end_offset, '/')) => {
                        // Skip until newline.
                        for (byte_offset_, c_) in input.by_ref() {
                            end_offset = byte_offset_;
                            if c_ == '\n' {
                                listener.handle_comment(byte_offset, end_offset - byte_offset + 1);
                                continue 'outer;
                            }
                        }

                        // End of input.
                        listener.handle_comment(byte_offset, end_offset - byte_offset + 1);
                        break;
                    }
                    _ => {
                        listener.handle_error(byte_offset + 1);
                        break;
                    }
                }
            }

            '"' => {
                let mut last_byte_offset = byte_offset;
                for (byte_offset_, c_) in input.by_ref() {
                    if c_ == '"' {
                        listener.handle_str(byte_offset + 1, byte_offset_ - byte_offset - 1);
                        continue 'outer;
                    }

                    last_byte_offset = byte_offset_;
                }

                // Unterminated string.
                listener.handle_error(last_byte_offset);
                break;
            }

            't' => {
                if matches!(input.next(), Some((_, 'r')))
                    && matches!(input.next(), Some((_, 'u')))
                    && matches!(input.next(), Some((_, 'e')))
                {
                    listener.handle_true(byte_offset);
                } else {
                    listener.handle_error(byte_offset);
                    break;
                }
            }

            'f' => {
                if matches!(input.next(), Some((_, 'a')))
                    && matches!(input.next(), Some((_, 'l')))
                    && matches!(input.next(), Some((_, 's')))
                    && matches!(input.next(), Some((_, 'e')))
                {
                    listener.handle_false(byte_offset);
                } else {
                    listener.handle_error(byte_offset);
                    break;
                }
            }

            'n' => {
                if matches!(input.next(), Some((_, 'u')))
                    && matches!(input.next(), Some((_, 'l')))
                    && matches!(input.next(), Some((_, 'l')))
                {
                    listener.handle_null(byte_offset);
                } else {
                    listener.handle_error(byte_offset);
                    break;
                }
            }

            c if c.is_ascii_digit() => {
                let mut i: u64 = u64::from((c as u8) - b'0');

                while let Some((_, next)) = input.peek().copied() {
                    if !next.is_ascii_digit() {
                        break;
                    }

                    // Consume the digit.
                    input.next();

                    // Ignore overflows for the purposes of this post.
                    i *= 10;
                    i += u64::from((next as u8) - b'0');
                }

                listener.handle_int(byte_offset, i);
            }

            ',' => listener.handle_comma(byte_offset),

            ':' => listener.handle_colon(byte_offset),

            '[' => listener.handle_lbracket(byte_offset),

            ']' => listener.handle_rbracket(byte_offset),

            '{' => listener.handle_lbrace(byte_offset),

            '}' => listener.handle_rbrace(byte_offset),

            _ => {
                listener.handle_error(byte_offset);
                break;
            }
        }
    }
}

pub struct PushToTokens {
    tokens: Vec<(usize, Token)>,
    error: Option<usize>,
}

impl LexerEventListener for PushToTokens {
    fn handle_int(&mut self, byte_offset: usize, i: u64) {
        self.tokens.push((byte_offset, Token::Int(i)));
    }

    fn handle_str(&mut self, byte_offset: usize, size_in_bytes: usize) {
        self.tokens
            .push((byte_offset, Token::Str { size_in_bytes }));
    }

    fn handle_true(&mut self, byte_offset: usize) {
        self.tokens.push((byte_offset, Token::True));
    }

    fn handle_false(&mut self, byte_offset: usize) {
        self.tokens.push((byte_offset, Token::False));
    }

    fn handle_null(&mut self, byte_offset: usize) {
        self.tokens.push((byte_offset, Token::Null));
    }

    fn handle_lbracket(&mut self, byte_offset: usize) {
        self.tokens.push((byte_offset, Token::LBracket));
    }

    fn handle_rbracket(&mut self, byte_offset: usize) {
        self.tokens.push((byte_offset, Token::RBracket));
    }

    fn handle_lbrace(&mut self, byte_offset: usize) {
        self.tokens.push((byte_offset, Token::LBrace));
    }

    fn handle_rbrace(&mut self, byte_offset: usize) {
        self.tokens.push((byte_offset, Token::RBrace));
    }

    fn handle_colon(&mut self, byte_offset: usize) {
        self.tokens.push((byte_offset, Token::Colon));
    }

    fn handle_comma(&mut self, byte_offset: usize) {
        self.tokens.push((byte_offset, Token::Comma));
    }

    fn handle_comment(&mut self, byte_offset: usize, size_in_bytes: usize) {
        self.tokens
            .push((byte_offset, Token::Comment { size_in_bytes }));
    }

    fn handle_error(&mut self, byte_offset: usize) {
        self.error = Some(byte_offset);
    }
}

impl PushToTokens {
    pub fn new() -> PushToTokens {
        PushToTokens {
            tokens: vec![],
            error: None,
        }
    }

    pub fn into_tokens(self) -> (Vec<(usize, Token)>, Option<usize>) {
        (self.tokens, self.error)
    }
}

#[cfg(test)]
fn tokenize_(input: &str) -> Vec<(usize, Token)> {
    let mut listener = PushToTokens::new();
    tokenize_push(input, &mut listener);
    let (tokens, error) = listener.into_tokens();
    assert_eq!(error, None);
    tokens
}

#[test]
fn test_keywords() {
    assert_eq!(
        tokenize_("true false null"),
        vec![(0, Token::True), (5, Token::False), (11, Token::Null)]
    );
}

#[test]
fn test_delimiters() {
    assert_eq!(
        tokenize_("{ } [ ]"),
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
        tokenize_("//"),
        vec![(0, Token::Comment { size_in_bytes: 2 })]
    );

    assert_eq!(
        tokenize_("//\n"),
        vec![(0, Token::Comment { size_in_bytes: 3 })]
    );

    assert_eq!(
        tokenize_("// asdf"),
        vec![(0, Token::Comment { size_in_bytes: 7 })]
    );

    assert_eq!(
        tokenize_("// asdf\n"),
        vec![(0, Token::Comment { size_in_bytes: 8 })]
    );
}

#[test]
fn test_strings() {
    assert_eq!(
        tokenize_(r#""""#),
        vec![(1, Token::Str { size_in_bytes: 0 })]
    );

    assert_eq!(
        tokenize_(r#""a""#),
        vec![(1, Token::Str { size_in_bytes: 1 })]
    );
}

#[test]
fn test_object() {
    assert_eq!(
        tokenize_(r#"{"a":1, "b":2}"#),
        vec![
            (0, Token::LBrace),
            (2, Token::Str { size_in_bytes: 1 }),
            (4, Token::Colon),
            (5, Token::Int(1)),
            (6, Token::Comma),
            (9, Token::Str { size_in_bytes: 1 }),
            (11, Token::Colon),
            (12, Token::Int(2)),
            (13, Token::RBrace),
        ]
    );
}
