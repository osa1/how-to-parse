use crate::{Json, ParseError};

use std::iter::Peekable;
use std::str::CharIndices;

/// Parses input directly to [Json].
pub fn parse(input: &str) -> Result<Json, ParseError> {
    let mut iter = input.char_indices().peekable();
    let (_, json) = parse_single(&mut iter, input)?;
    skip_trivia(&mut iter)?;
    if let Some((byte_offset, _)) = iter.next() {
        // We should return the parsed object with this error, but it's OK for the purposes of this
        // post.
        return Err(ParseError {
            byte_offset,
            reason: "trailing characters after paring",
        });
    }
    Ok(json)
}

fn parse_single(
    iter: &mut Peekable<CharIndices>,
    input: &str,
) -> Result<(usize, Json), ParseError> {
    skip_trivia(iter)?;

    let (byte_offset, char) = match iter.next() {
        Some(next) => next,
        None => {
            return Err(ParseError {
                byte_offset: input.len(),
                reason: "unexpected end of input",
            })
        }
    };

    if char == '[' {
        let mut array: Vec<Json> = Vec::with_capacity(10);
        loop {
            skip_trivia(iter)?;
            match iter.peek().copied() {
                Some((_, ']')) => {
                    // Consume ']'
                    iter.next();
                    return Ok((byte_offset, Json::Array(array)));
                }

                Some((comma_byte_offset, ',')) => {
                    if array.is_empty() {
                        return Err(ParseError {
                            byte_offset: comma_byte_offset,
                            reason: "unexpected character while parsing array",
                        });
                    }

                    // Consume ','
                    iter.next();
                    array.push(parse_single(iter, input)?.1);
                }

                Some((byte_offset, _)) => {
                    if !array.is_empty() {
                        // Need to see a ',' before the next element.
                        return Err(ParseError {
                            byte_offset,
                            reason: "unexpected character while parsing array",
                        });
                    }

                    array.push(parse_single(iter, input)?.1);
                }

                None => {
                    return Err(ParseError {
                        byte_offset: input.len(),
                        reason: "end of input while parsing array",
                    })
                }
            }
        }
    }

    if char == '{' {
        let mut object: Vec<(String, Json)> = Vec::with_capacity(10);

        enum State {
            Done,
            ExpectKey,
            ExpectColon { key: String },
            ExpectValue { key: String },
        }

        let mut state = State::Done;

        loop {
            skip_trivia(iter)?;
            match std::mem::replace(&mut state, State::Done) {
                State::Done => {
                    match iter.peek().copied() {
                        Some((byte_offset, ',')) => {
                            if object.is_empty() {
                                return Err(ParseError {
                                    byte_offset,
                                    reason: "unexpected comma while parsing object",
                                });
                            }
                            iter.next(); // consume ','
                            state = State::ExpectKey;
                        }

                        Some((_, '}')) => {
                            iter.next(); // consume '}'
                            return Ok((byte_offset, Json::Object(object)));
                        }

                        Some((_, '"')) => {
                            let key = parse_single(iter, input)?.1.into_string();
                            state = State::ExpectColon { key };
                        }

                        Some((byte_offset, _)) => {
                            return Err(ParseError {
                                byte_offset,
                                reason: "unexpected char while parsing object",
                            })
                        }

                        None => {
                            return Err(ParseError {
                                byte_offset: input.len(),
                                reason: "unexpected end of input while parsing object",
                            })
                        }
                    }
                }

                State::ExpectKey => match parse_single(iter, input)? {
                    (_, Json::String(key)) => {
                        state = State::ExpectColon { key };
                    }

                    (byte_offset, _) => {
                        return Err(ParseError {
                            byte_offset,
                            reason: "unexpected value while parsing object key",
                        })
                    }
                },

                State::ExpectColon { key } => match iter.next() {
                    Some((_, ':')) => {
                        state = State::ExpectValue { key };
                    }

                    Some((byte_offset, _)) => {
                        return Err(ParseError {
                            byte_offset,
                            reason: "unexpected char while parsing object",
                        })
                    }

                    None => {
                        return Err(ParseError {
                            byte_offset: input.len(),
                            reason: "unexpected end of input while parsing object",
                        })
                    }
                },

                State::ExpectValue { key } => {
                    let value = parse_single(iter, input)?.1;
                    object.push((key, value));
                    state = State::Done;
                }
            }
        }
    }

    if char == 't' {
        if next_char(iter) == Some('r')
            && next_char(iter) == Some('u')
            && next_char(iter) == Some('e')
        {
            return Ok((byte_offset, Json::Bool(true)));
        }
        return Err(ParseError {
            byte_offset,
            reason: "unexpected keyword",
        });
    }

    if char == 'f' {
        if next_char(iter) == Some('a')
            && next_char(iter) == Some('l')
            && next_char(iter) == Some('s')
            && next_char(iter) == Some('e')
        {
            return Ok((byte_offset, Json::Bool(false)));
        }
        return Err(ParseError {
            byte_offset,
            reason: "unexpected keyword",
        });
    }

    if char == 'n' {
        if next_char(iter) == Some('u')
            && next_char(iter) == Some('l')
            && next_char(iter) == Some('l')
        {
            return Ok((byte_offset, Json::Null));
        }
        return Err(ParseError {
            byte_offset,
            reason: "unexpected keyword",
        });
    }

    if char.is_ascii_digit() {
        let mut i: u64 = u64::from((char as u8) - b'0');

        while let Some((_, next)) = iter.peek().copied() {
            if !next.is_ascii_digit() {
                break;
            }

            // Consume the digit.
            iter.next();

            // Ignore overflows for the purposes of this post.
            i *= 10;
            i += u64::from((next as u8) - b'0');
        }

        return Ok((byte_offset, Json::Int(i)));
    }

    if char == '"' {
        for (next_byte_offset, next) in iter.by_ref() {
            if next == '"' {
                let string = input[byte_offset + 1..next_byte_offset].to_string();
                return Ok((byte_offset, Json::String(string)));
            }
        }

        return Err(ParseError {
            byte_offset: input.len(),
            reason: "unexpected end of input while parsing string",
        });
    }

    Err(ParseError {
        byte_offset,
        reason: "unexpected character",
    })
}

fn next_char(iter: &mut Peekable<CharIndices>) -> Option<char> {
    iter.next().map(|(_, char)| char)
}

pub(crate) fn skip_trivia(iter: &mut Peekable<CharIndices>) -> Result<(), ParseError> {
    while let Some((byte_offset, char)) = iter.peek().copied() {
        if char.is_ascii_whitespace() {
            iter.next();
            continue;
        }

        if char == '/' {
            iter.next();
            match iter.next() {
                Some((_, '/')) => {
                    skip_until_eol(iter);
                    break;
                }

                Some(_) => {
                    return Err(ParseError {
                        byte_offset,
                        reason: "unexpected character",
                    })
                }

                None => {
                    return Err(ParseError {
                        byte_offset,
                        reason: "unexpected end of input",
                    })
                }
            }
        }

        break;
    }

    Ok(())
}

fn skip_until_eol(iter: &mut Peekable<CharIndices>) {
    for (_, char) in iter.by_ref() {
        if char == '\n' {
            break;
        }
    }
}

#[test]
fn ast_tests() {
    for (str, ast) in crate::test_common::ast_tests() {
        println!("Parsing {:?}", str);
        assert_eq!(parse(&str).unwrap(), ast);
    }
}
