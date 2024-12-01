use crate::{EventListener, ParseError, Token};

use std::iter::Peekable;

type Item = Result<(usize, Token), usize>;

pub fn parse<L: EventListener, I: Iterator<Item = Item>>(
    lexer: I,
    listener: &mut L,
    input_size: usize,
) {
    let mut lexer = lexer.peekable();

    if !parse_single(&mut lexer, input_size, listener) {
        return;
    }

    // Check trailing tokens.
    for token in lexer {
        match token {
            Ok((byte_offset, t)) => match t {
                Token::Comment { size_in_bytes } => {
                    listener.handle_comment(byte_offset, size_in_bytes);
                }
                _ => {
                    listener.handle_error(ParseError {
                        byte_offset,
                        reason: "trailing token",
                    });
                    break;
                }
            },
            Err(byte_offset) => {
                listener.handle_error(ParseError {
                    byte_offset,
                    reason: "invalid token",
                });
                break;
            }
        }
    }
}

fn parse_single<L: EventListener, I: Iterator<Item = Item>>(
    lexer: &mut Peekable<I>,
    input_size: usize,
    listener: &mut L,
) -> bool {
    while let Some(token) = lexer.next() {
        let (byte_offset, token) = match token {
            Ok(next) => next,
            Err(byte_offset) => {
                listener.handle_error(ParseError {
                    byte_offset,
                    reason: "invalid token",
                });
                return false;
            }
        };

        match token {
            Token::Comment { size_in_bytes } => {
                listener.handle_comment(byte_offset, size_in_bytes);
            }

            Token::LBracket => {
                listener.handle_start_array(byte_offset);
                let mut array_is_empty = true;

                loop {
                    match lexer.peek().copied() {
                        Some(Ok((byte_offset, t))) => match t {
                            Token::Comment { size_in_bytes } => {
                                listener.handle_comment(byte_offset, size_in_bytes);
                                lexer.next(); // consume comment
                                continue;
                            }

                            Token::Comma => {
                                if array_is_empty {
                                    listener.handle_error(ParseError {
                                        byte_offset,
                                        reason: "unexpected character while parsing array",
                                    });
                                    return false;
                                }

                                lexer.next(); // consume comma
                                if !parse_single(lexer, input_size, listener) {
                                    return false;
                                }
                            }

                            Token::RBracket => {
                                listener.handle_end_array(byte_offset);
                                lexer.next(); // consume bracket
                                return true;
                            }

                            _ => {
                                if !array_is_empty {
                                    // Need to see a ',' before the next element.
                                    listener.handle_error(ParseError {
                                        byte_offset,
                                        reason: "unexpected character while parsing array",
                                    });
                                    return false;
                                }

                                if !parse_single(lexer, byte_offset, listener) {
                                    return false;
                                }

                                array_is_empty = false;
                            }
                        },

                        Some(Err(byte_offset)) => {
                            listener.handle_error(ParseError {
                                byte_offset,
                                reason: "invalid token",
                            });
                            return false;
                        }

                        None => {
                            listener.handle_error(ParseError {
                                byte_offset: input_size,
                                reason: "unexpected end of input",
                            });
                            return false;
                        }
                    }
                }
            }

            Token::LBrace => {
                listener.handle_start_object(byte_offset);
                let mut object_is_empty = true;

                enum State {
                    Done,
                    ExpectKey,
                    ExpectColon,
                    ExpectValue,
                }

                let mut state = State::Done;

                loop {
                    match state {
                        State::Done => {
                            match lexer.peek().copied() {
                                Some(Ok((byte_offset, Token::Comment { size_in_bytes }))) => {
                                    listener.handle_comment(byte_offset, size_in_bytes);
                                    lexer.next(); // consume comment
                                    continue;
                                }

                                Some(Ok((byte_offset, Token::Comma))) => {
                                    if object_is_empty {
                                        listener.handle_error(ParseError {
                                            byte_offset,
                                            reason: "unexpected comma while parsing object",
                                        });
                                        return false;
                                    }
                                    lexer.next(); // consume ','
                                    state = State::ExpectKey;
                                }

                                Some(Ok((_, Token::RBrace))) => {
                                    lexer.next(); // consume '}'
                                    listener.handle_end_object(byte_offset);
                                    return true;
                                }

                                Some(Ok((byte_offset, Token::Str { size_in_bytes }))) => {
                                    lexer.next(); // consume string
                                    listener.handle_str(byte_offset, size_in_bytes);
                                    state = State::ExpectColon;
                                }

                                Some(Ok((byte_offset, _))) => {
                                    listener.handle_error(ParseError {
                                        byte_offset,
                                        reason: "unexpected token while parsing object",
                                    });
                                    return false;
                                }

                                Some(Err(byte_offset)) => {
                                    listener.handle_error(ParseError {
                                        byte_offset,
                                        reason: "invalid token",
                                    });
                                    return false;
                                }

                                None => {
                                    listener.handle_error(ParseError {
                                        byte_offset: input_size,
                                        reason: "unexpected end of input while parsing object",
                                    });
                                    return false;
                                }
                            }
                        }

                        State::ExpectKey => {
                            match lexer.peek().copied() {
                                Some(Ok((byte_offset, Token::Comment { size_in_bytes }))) => {
                                    listener.handle_comment(byte_offset, size_in_bytes);
                                    lexer.next(); // consume comment
                                    continue;
                                }

                                Some(Ok((byte_offset, Token::Str { size_in_bytes }))) => {
                                    listener.handle_str(byte_offset, size_in_bytes);
                                    lexer.next(); // consume string
                                    state = State::ExpectColon;
                                }

                                Some(Ok((byte_offset, _))) => {
                                    listener.handle_error(ParseError {
                                        byte_offset,
                                        reason: "unexpected token",
                                    });
                                    return false;
                                }

                                Some(Err(byte_offset)) => {
                                    listener.handle_error(ParseError {
                                        byte_offset,
                                        reason: "invalid token",
                                    });
                                    return false;
                                }

                                None => {
                                    listener.handle_error(ParseError {
                                        byte_offset: input_size,
                                        reason: "unexpected end of input while parsing object",
                                    });
                                    return false;
                                }
                            }
                        }

                        State::ExpectColon => {
                            match lexer.peek().copied() {
                                Some(Ok((byte_offset, Token::Comment { size_in_bytes }))) => {
                                    listener.handle_comment(byte_offset, size_in_bytes);
                                    lexer.next(); // consume comment
                                    continue;
                                }

                                Some(Ok((_, Token::Colon))) => {
                                    lexer.next(); // consume colon
                                    state = State::ExpectValue;
                                }

                                Some(Ok((byte_offset, _))) => {
                                    listener.handle_error(ParseError {
                                        byte_offset,
                                        reason: "unexpected token",
                                    });
                                    return false;
                                }

                                Some(Err(byte_offset)) => {
                                    listener.handle_error(ParseError {
                                        byte_offset,
                                        reason: "invalid token",
                                    });
                                    return false;
                                }

                                None => {
                                    listener.handle_error(ParseError {
                                        byte_offset: input_size,
                                        reason: "unexpected end of input while parsing object",
                                    });
                                    return false;
                                }
                            }
                        }

                        State::ExpectValue => {
                            if !parse_single(lexer, input_size, listener) {
                                return false;
                            }
                            object_is_empty = false;
                            state = State::Done;
                        }
                    }
                }
            }

            Token::True => {
                listener.handle_bool(byte_offset, true);
                return true;
            }

            Token::False => {
                listener.handle_bool(byte_offset, false);
                return true;
            }

            Token::Null => {
                listener.handle_null(byte_offset);
                return true;
            }

            Token::Int(i) => {
                listener.handle_int(byte_offset, i);
                return true;
            }

            Token::Str { size_in_bytes } => {
                listener.handle_str(byte_offset, size_in_bytes);
                return true;
            }

            Token::RBracket | Token::RBrace | Token::Colon | Token::Comma => {
                listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
fn collect_events(input: &str) -> Vec<crate::ParseEvent> {
    let mut listener = crate::PushToEvents::new();
    parse(crate::tokenize_iter(input), &mut listener, input.len());
    let (events, error) = listener.into_events();
    assert_eq!(error, None);
    events
}

#[cfg(test)]
fn collect_event_kinds(input: &str) -> Vec<crate::ParseEventKind> {
    collect_events(input)
        .into_iter()
        .map(|ev| ev.kind)
        .collect()
}

#[test]
fn event_tests() {
    for (str, events) in crate::test_common::event_tests() {
        println!("Parsing {:?}", str);
        let events_ = collect_event_kinds(&str);
        assert_eq!(events_, events);
    }
}

#[test]
fn event_to_tree_tests() {
    for (str, ast) in crate::test_common::ast_tests() {
        let events = collect_events(&str);
        let ast_ = crate::event_to_tree(&mut events.into_iter().map(Result::Ok), &str).unwrap();
        assert_eq!(ast_, ast);
    }
}

#[test]
fn event_to_tree_random_tests() {
    for input_size in [10, 100, 1_000, 2_000, 5_000, 10_000, 10_000_000] {
        let input = crate::gen_input(input_size);
        let events = collect_events(&input);
        let event_ast =
            crate::event_to_tree(&mut events.into_iter().map(Result::Ok), &input).unwrap();
        let ast = crate::recursive_descent::parse(&input).unwrap();
        assert_eq!(event_ast, ast);
    }
}
