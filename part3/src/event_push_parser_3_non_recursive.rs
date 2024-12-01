use crate::direct_non_recursive::ParserState;
use crate::event_parser::Container;
use crate::{EventListener, ParseError, Token};

type Item = Result<(usize, Token), usize>;

#[allow(unused)]
pub fn parse<L: EventListener, I: Iterator<Item = Item>>(
    lexer: &mut I,
    listener: &mut L,
    input_size: usize,
) {
    if !parse_single(lexer, input_size, listener) {
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
    lexer: &mut I,
    input_size: usize,
    listener: &mut L,
) -> bool {
    let mut container_stack: Vec<Container> = vec![];
    let mut state = ParserState::TopLevel;

    loop {
        let (byte_offset, token) = match lexer.next() {
            Some(Ok(next)) => next,

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
        };

        if let Token::Comment { size_in_bytes } = token {
            listener.handle_comment(byte_offset, size_in_bytes);
            continue;
        }

        match dbg!(state) {
            ParserState::TopLevel => match token {
                Token::LBrace => {
                    container_stack.push(Container::Object);
                    state = ParserState::ObjectExpectKeyValueTerminate;
                    listener.handle_start_object(byte_offset);
                }

                Token::LBracket => {
                    container_stack.push(Container::Array);
                    state = ParserState::TopLevel;
                    listener.handle_start_array(byte_offset);
                }

                Token::RBracket => {
                    if !matches!(container_stack.pop(), Some(Container::Array)) {
                        listener.handle_error(ParseError {
                            byte_offset,
                            reason: "unexpected ']'",
                        });
                        return false;
                    }

                    listener.handle_end_array(byte_offset);

                    if container_stack.is_empty() {
                        break;
                    }
                    state = ParserState::ExpectComma;
                }

                Token::True => {
                    listener.handle_bool(byte_offset, true);
                    if container_stack.is_empty() {
                        break;
                    }
                    state = ParserState::ExpectComma;
                }

                Token::False => {
                    listener.handle_bool(byte_offset, false);
                    if container_stack.is_empty() {
                        break;
                    }
                    state = ParserState::ExpectComma;
                }

                Token::Null => {
                    listener.handle_null(byte_offset);
                    if container_stack.is_empty() {
                        break;
                    }
                    state = ParserState::ExpectComma;
                }

                Token::Int(i) => {
                    listener.handle_int(byte_offset, i);
                    if container_stack.is_empty() {
                        break;
                    }
                    state = ParserState::ExpectComma;
                }

                Token::Str { size_in_bytes } => {
                    listener.handle_str(byte_offset, size_in_bytes);
                    if container_stack.is_empty() {
                        break;
                    }
                    state = ParserState::ExpectComma;
                }

                _ => {
                    listener.handle_error(ParseError {
                        byte_offset,
                        reason: "unexpected token",
                    });
                }
            },

            ParserState::ExpectComma => match token {
                Token::Comma => match container_stack.last() {
                    Some(Container::Array) => state = ParserState::TopLevel,
                    Some(Container::Object) => state = ParserState::ObjectExpectKeyValue,
                    None => unreachable!(),
                },

                Token::RBrace => {
                    if let Container::Array = container_stack.pop().unwrap() {
                        listener.handle_error(ParseError {
                            byte_offset,
                            reason: "unexpected '}'",
                        });
                        return false;
                    }

                    listener.handle_end_object(byte_offset);

                    if container_stack.is_empty() {
                        break;
                    }
                    state = ParserState::ExpectComma;
                }

                Token::RBracket => {
                    if let Container::Object = container_stack.pop().unwrap() {
                        listener.handle_error(ParseError {
                            byte_offset,
                            reason: "unexpected ']'",
                        });
                        return false;
                    }

                    listener.handle_end_array(byte_offset);

                    if container_stack.is_empty() {
                        break;
                    }
                    state = ParserState::ExpectComma;
                }

                _ => {
                    listener.handle_error(ParseError {
                        byte_offset,
                        reason: "unexpected token",
                    });
                    return false;
                }
            },

            ParserState::ObjectExpectKeyValue => match token {
                Token::Str { size_in_bytes } => {
                    listener.handle_str(byte_offset, size_in_bytes);
                    state = ParserState::ObjectExpectColon;
                }

                _ => {
                    listener.handle_error(ParseError {
                        byte_offset,
                        reason: "unexpected token",
                    });
                    return false;
                }
            },

            ParserState::ObjectExpectKeyValueTerminate => match token {
                Token::Str { size_in_bytes } => {
                    listener.handle_str(byte_offset, size_in_bytes);
                    state = ParserState::ObjectExpectColon;
                }

                Token::RBrace => {
                    container_stack.pop();
                    listener.handle_end_object(byte_offset);

                    if container_stack.is_empty() {
                        break;
                    }
                    state = ParserState::ExpectComma;
                }

                _ => {
                    listener.handle_error(ParseError {
                        byte_offset,
                        reason: "unexpected token",
                    });
                    return false;
                }
            },

            ParserState::ObjectExpectColon => match token {
                Token::Colon => {
                    state = ParserState::TopLevel;
                }

                _ => {
                    listener.handle_error(ParseError {
                        byte_offset,
                        reason: "unexpected token",
                    });
                    return false;
                }
            },
        }
    }

    true
}

#[cfg(test)]
fn collect_events(input: &str) -> Vec<crate::ParseEvent> {
    let mut listener = crate::PushToEvents::new();
    parse(&mut crate::tokenize_iter(input), &mut listener, input.len());
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
