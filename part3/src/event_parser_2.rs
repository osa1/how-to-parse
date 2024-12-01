use crate::event_parser::{Container, ParserState};
use crate::{ParseError, ParseEvent, ParseEventKind, Token};

type Item = Result<(usize, Token), usize>;

/// Parses input to [ParseEvent]s.
pub fn parse_events_iter_using_lexer_iter<I: Iterator<Item = Item>>(
    lexer: I,
    input_size: usize,
) -> EventParser<I> {
    EventParser::new(lexer, input_size)
}

pub struct EventParser<I: Iterator<Item = Item>> {
    lexer: I,
    container_stack: Vec<Container>,
    state: ParserState,
    input_size: usize,
}

impl<I: Iterator<Item = Item>> EventParser<I> {
    fn new(lexer: I, input_size: usize) -> EventParser<I> {
        EventParser {
            lexer,
            container_stack: vec![],
            state: ParserState::TopLevel,
            input_size,
        }
    }
}

impl<I: Iterator<Item = Item>> Iterator for EventParser<I> {
    type Item = Result<ParseEvent, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            ParserState::TopLevel => self.top_level(),
            ParserState::Done => self.done(),
            ParserState::ObjectExpectComma => self.object_expect_comma(),
            ParserState::ObjectExpectKeyValue => self.object_expect_key_value(),
            ParserState::ObjectExpectColon => self.object_expect_colon(),
            ParserState::ArrayExpectComma => self.array_expect_comma(),
        }
    }
}

impl<I: Iterator<Item = Item>> EventParser<I> {
    fn top_level(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        match self.lexer.next() {
            Some(Ok((byte_offset, t))) => match t {
                Token::LBracket => {
                    self.container_stack.push(Container::Array);
                    Some(Ok(ParseEvent::new(byte_offset, ParseEventKind::StartArray)))
                }

                Token::RBracket => match self.pop_array(byte_offset) {
                    Ok(()) => {
                        self.update_state();
                        Some(Ok(ParseEvent::new(byte_offset, ParseEventKind::EndArray)))
                    }
                    Err(err) => Some(Err(err)),
                },

                Token::LBrace => {
                    self.container_stack.push(Container::Object);
                    self.state = ParserState::ObjectExpectKeyValue;
                    Some(Ok(ParseEvent::new(
                        byte_offset,
                        ParseEventKind::StartObject,
                    )))
                }

                Token::True => {
                    self.update_state();
                    Some(Ok(ParseEvent::new(byte_offset, ParseEventKind::Bool(true))))
                }

                Token::False => {
                    self.update_state();
                    Some(Ok(ParseEvent::new(
                        byte_offset,
                        ParseEventKind::Bool(false),
                    )))
                }

                Token::Null => {
                    self.update_state();
                    Some(Ok(ParseEvent::new(byte_offset, ParseEventKind::Null)))
                }

                Token::Int(i) => {
                    self.update_state();
                    Some(Ok(ParseEvent::new(byte_offset, ParseEventKind::Int(i))))
                }

                Token::Str { size_in_bytes } => {
                    self.update_state();

                    Some(Ok(ParseEvent::new(
                        byte_offset,
                        ParseEventKind::Str { size_in_bytes },
                    )))
                }

                Token::Comment { size_in_bytes } => Some(Ok(ParseEvent::new(
                    byte_offset,
                    ParseEventKind::Comment { size_in_bytes },
                ))),

                Token::RBrace | Token::Colon | Token::Comma => Some(Err(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                })),
            },

            Some(Err(byte_offset)) => Some(Err(ParseError {
                byte_offset,
                reason: "invalid token",
            })),

            None => Some(Err(ParseError {
                byte_offset: self.input_size,
                reason: "unexpected end of input",
            })),
        }
    }

    fn done(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        match self.lexer.next() {
            Some(Ok((byte_offset, t))) => match t {
                Token::Comment { size_in_bytes } => Some(Ok(ParseEvent::new(
                    byte_offset,
                    ParseEventKind::Comment { size_in_bytes },
                ))),
                _ => Some(Err(ParseError {
                    byte_offset,
                    reason: "trailing tokens",
                })),
            },
            Some(Err(byte_offset)) => Some(Err(ParseError {
                byte_offset,
                reason: "invalid token",
            })),
            None => None,
        }
    }

    fn array_expect_comma(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        match self.lexer.next() {
            Some(Ok((byte_offset, t))) => match t {
                Token::Comma => {
                    self.state = ParserState::TopLevel;
                    self.next()
                }

                Token::RBracket => match self.pop_array(byte_offset) {
                    Ok(()) => {
                        self.update_state();
                        Some(Ok(ParseEvent::new(byte_offset, ParseEventKind::EndArray)))
                    }
                    Err(err) => Some(Err(err)),
                },

                Token::Comment { size_in_bytes } => Some(Ok(ParseEvent::new(
                    byte_offset,
                    ParseEventKind::Comment { size_in_bytes },
                ))),

                _ => Some(Err(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                })),
            },

            Some(Err(byte_offset)) => Some(Err(ParseError {
                byte_offset,
                reason: "invalid token",
            })),

            None => Some(Err(ParseError {
                byte_offset: self.input_size,
                reason: "unexpected end of input",
            })),
        }
    }

    fn object_expect_key_value(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        match self.lexer.next() {
            Some(Ok((byte_offset, t))) => match t {
                Token::RBrace => match self.pop_object(byte_offset) {
                    Ok(()) => {
                        self.update_state();
                        Some(Ok(ParseEvent::new(byte_offset, ParseEventKind::EndObject)))
                    }
                    Err(err) => Some(Err(err)),
                },

                Token::Str { size_in_bytes } => {
                    self.state = ParserState::ObjectExpectColon;
                    Some(Ok(ParseEvent::new(
                        byte_offset,
                        ParseEventKind::Str { size_in_bytes },
                    )))
                }

                Token::Comment { size_in_bytes } => Some(Ok(ParseEvent::new(
                    byte_offset,
                    ParseEventKind::Comment { size_in_bytes },
                ))),

                _ => Some(Err(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                })),
            },

            Some(Err(byte_offset)) => Some(Err(ParseError {
                byte_offset,
                reason: "invalid token",
            })),

            None => Some(Err(ParseError {
                byte_offset: self.input_size,
                reason: "unexpected end of input",
            })),
        }
    }

    fn object_expect_colon(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        match self.lexer.next() {
            Some(Ok((byte_offset, t))) => match t {
                Token::Colon => {
                    self.state = ParserState::TopLevel;
                    self.next()
                }

                Token::Comment { size_in_bytes } => Some(Ok(ParseEvent::new(
                    byte_offset,
                    ParseEventKind::Comment { size_in_bytes },
                ))),

                _ => Some(Err(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                })),
            },

            Some(Err(byte_offset)) => Some(Err(ParseError {
                byte_offset,
                reason: "invalid token",
            })),

            None => Some(Err(ParseError {
                byte_offset: self.input_size,
                reason: "unexpected end of input",
            })),
        }
    }

    fn object_expect_comma(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        match self.lexer.next() {
            Some(Ok((byte_offset, t))) => match t {
                Token::Comma => {
                    self.state = ParserState::ObjectExpectKeyValue;
                    self.next()
                }

                Token::RBrace => match self.pop_object(byte_offset) {
                    Ok(()) => {
                        self.update_state();
                        Some(Ok(ParseEvent::new(byte_offset, ParseEventKind::EndObject)))
                    }
                    Err(err) => Some(Err(err)),
                },

                Token::Comment { size_in_bytes } => Some(Ok(ParseEvent::new(
                    byte_offset,
                    ParseEventKind::Comment { size_in_bytes },
                ))),

                _ => Some(Err(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                })),
            },

            Some(Err(byte_offset)) => Some(Err(ParseError {
                byte_offset,
                reason: "invalid token",
            })),

            None => Some(Err(ParseError {
                byte_offset: self.input_size,
                reason: "unexpected end of input",
            })),
        }
    }

    fn update_state(&mut self) {
        self.state = match self.container_stack.last() {
            Some(Container::Array) => ParserState::ArrayExpectComma,
            Some(Container::Object) => ParserState::ObjectExpectComma,
            None => ParserState::Done,
        };
    }

    fn pop_array(&mut self, byte_offset: usize) -> Result<(), ParseError> {
        match self.container_stack.pop() {
            Some(Container::Array) => Ok(()),

            _ => Err(ParseError {
                byte_offset,
                reason: "unexpected ']'",
            }),
        }
    }

    fn pop_object(&mut self, byte_offset: usize) -> Result<(), ParseError> {
        match self.container_stack.pop() {
            Some(Container::Object) => Ok(()),

            _ => Err(ParseError {
                byte_offset,
                reason: "unexpected '}'",
            }),
        }
    }
}

#[cfg(test)]
fn collect_events(input: &str) -> Vec<crate::ParseEventKind> {
    parse_events_iter_using_lexer_iter(crate::tokenize_iter(input), input.len())
        .map(|ev| ev.unwrap().kind)
        .collect()
}

#[test]
fn event_tests() {
    for (str, events) in crate::test_common::event_tests() {
        println!("Parsing {:?}", str);
        let events_ = collect_events(&str);
        assert_eq!(events_, events);
    }
}

#[test]
fn event_to_tree_tests() {
    for (str, ast) in crate::test_common::ast_tests() {
        let mut parser = parse_events_iter_using_lexer_iter(crate::tokenize_iter(&str), str.len());
        let ast_ = crate::event_to_tree(&mut parser, &str).unwrap();
        assert_eq!(ast_, ast);
    }
}

#[test]
fn event_to_tree_random_tests() {
    for input_size in [10, 100, 1_000, 2_000, 5_000, 10_000] {
        let input = crate::gen_input(input_size);
        let event_ast = crate::event_to_tree(
            &mut parse_events_iter_using_lexer_iter(crate::Lexer::new(&input), input.len()),
            &input,
        )
        .unwrap();
        let ast = crate::recursive_descent::parse(&input).unwrap();
        assert_eq!(event_ast, ast);
    }
}
