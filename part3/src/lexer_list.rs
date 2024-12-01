use crate::Token;

pub fn tokenize_list(input: &str) -> Result<Vec<(usize, Token)>, usize> {
    let mut tokens: Vec<(usize, Token)> = vec![];

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
                                tokens.push((
                                    byte_offset,
                                    Token::Comment {
                                        size_in_bytes: end_offset - byte_offset + 1,
                                    },
                                ));
                                continue 'outer;
                            }
                        }

                        // End of input.
                        tokens.push((
                            byte_offset,
                            Token::Comment {
                                size_in_bytes: end_offset - byte_offset + 1,
                            },
                        ));

                        break;
                    }
                    _ => {
                        return Err(byte_offset + 1);
                    }
                }
            }

            '"' => {
                let mut last_byte_offset = byte_offset;
                for (byte_offset_, c_) in input.by_ref() {
                    if c_ == '"' {
                        tokens.push((
                            byte_offset,
                            Token::Str {
                                size_in_bytes: byte_offset_ - byte_offset - 1,
                            },
                        ));
                        continue 'outer;
                    }

                    last_byte_offset = byte_offset_;
                }

                // Unterminated string.
                return Err(last_byte_offset);
            }

            't' => {
                if matches!(input.next(), Some((_, 'r')))
                    && matches!(input.next(), Some((_, 'u')))
                    && matches!(input.next(), Some((_, 'e')))
                {
                    tokens.push((byte_offset, Token::True));
                } else {
                    return Err(byte_offset);
                }
            }

            'f' => {
                if matches!(input.next(), Some((_, 'a')))
                    && matches!(input.next(), Some((_, 'l')))
                    && matches!(input.next(), Some((_, 's')))
                    && matches!(input.next(), Some((_, 'e')))
                {
                    tokens.push((byte_offset, Token::False));
                } else {
                    return Err(byte_offset);
                }
            }

            'n' => {
                if matches!(input.next(), Some((_, 'u')))
                    && matches!(input.next(), Some((_, 'l')))
                    && matches!(input.next(), Some((_, 'l')))
                {
                    tokens.push((byte_offset, Token::Null));
                } else {
                    return Err(byte_offset);
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

                tokens.push((byte_offset, Token::Int(i)));
            }

            ',' => tokens.push((byte_offset, Token::Comma)),

            ':' => tokens.push((byte_offset, Token::Colon)),

            '[' => tokens.push((byte_offset, Token::LBracket)),

            ']' => tokens.push((byte_offset, Token::RBracket)),

            '{' => tokens.push((byte_offset, Token::LBrace)),

            '}' => tokens.push((byte_offset, Token::RBrace)),

            _ => return Err(byte_offset),
        }
    }

    Ok(tokens)
}
