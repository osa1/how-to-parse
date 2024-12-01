use crate::event_parser::{Container, ParserState};
use crate::{tokenize_push, EventListener, LexerEventListener, ParseError};

pub fn parse<L: EventListener>(input: &str, listener: &mut L) {
    let mut lexer_event_listener = LexerEventListenerImpl {
        listener,
        container_stack: vec![],
        state: ParserState::TopLevel,
    };
    tokenize_push(input, &mut lexer_event_listener);
}

struct LexerEventListenerImpl<'a, L: EventListener> {
    listener: &'a mut L,
    container_stack: Vec<Container>,
    state: ParserState,
}

impl<'a, L: EventListener> LexerEventListenerImpl<'a, L> {
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

impl<'a, L: EventListener> LexerEventListener for LexerEventListenerImpl<'a, L> {
    fn handle_int(&mut self, byte_offset: usize, i: u64) {
        match self.state {
            ParserState::TopLevel => {
                self.listener.handle_int(byte_offset, i);
                self.update_state();
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_str(&mut self, byte_offset: usize, size_in_bytes: usize) {
        match self.state {
            ParserState::TopLevel => {
                self.listener.handle_str(byte_offset, size_in_bytes);
                self.update_state();
            }
            ParserState::ObjectExpectKeyValue => {
                self.listener.handle_str(byte_offset, size_in_bytes);
                self.state = ParserState::ObjectExpectColon;
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_true(&mut self, byte_offset: usize) {
        match self.state {
            ParserState::TopLevel => {
                self.listener.handle_bool(byte_offset, true);
                self.update_state();
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_false(&mut self, byte_offset: usize) {
        match self.state {
            ParserState::TopLevel => {
                self.listener.handle_bool(byte_offset, false);
                self.update_state();
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_null(&mut self, byte_offset: usize) {
        match self.state {
            ParserState::TopLevel => {
                self.listener.handle_null(byte_offset);
                self.update_state();
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_lbracket(&mut self, byte_offset: usize) {
        match self.state {
            ParserState::TopLevel => {
                self.container_stack.push(Container::Array);
                self.listener.handle_start_array(byte_offset);
                // state stays at TopLevel
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_rbracket(&mut self, byte_offset: usize) {
        match self.state {
            ParserState::TopLevel | ParserState::ArrayExpectComma => {
                match self.pop_array(byte_offset) {
                    Ok(()) => {
                        self.listener.handle_end_array(byte_offset);
                        self.update_state();
                    }
                    Err(err) => {
                        self.listener.handle_error(err);
                    }
                }
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_lbrace(&mut self, byte_offset: usize) {
        match self.state {
            ParserState::TopLevel => {
                self.container_stack.push(Container::Object);
                self.listener.handle_start_object(byte_offset);
                self.state = ParserState::ObjectExpectKeyValue;
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_rbrace(&mut self, byte_offset: usize) {
        match self.state {
            ParserState::ObjectExpectKeyValue | ParserState::ObjectExpectComma => {
                match self.pop_object(byte_offset) {
                    Ok(()) => {
                        self.listener.handle_end_object(byte_offset);
                        self.update_state();
                    }
                    Err(err) => {
                        self.listener.handle_error(err);
                    }
                }
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_colon(&mut self, byte_offset: usize) {
        match self.state {
            ParserState::ObjectExpectColon => {
                self.state = ParserState::TopLevel;
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_comma(&mut self, byte_offset: usize) {
        match self.state {
            ParserState::ObjectExpectComma => {
                self.state = ParserState::ObjectExpectKeyValue;
            }
            ParserState::ArrayExpectComma => {
                self.state = ParserState::TopLevel;
            }
            _ => {
                self.listener.handle_error(ParseError {
                    byte_offset,
                    reason: "unexpected token",
                });
            }
        }
    }

    fn handle_comment(&mut self, byte_offset: usize, size_in_bytes: usize) {
        self.listener.handle_comment(byte_offset, size_in_bytes);
    }

    fn handle_error(&mut self, byte_offset: usize) {
        self.listener.handle_error(ParseError {
            byte_offset,
            reason: "invalid token",
        });
    }
}

#[cfg(test)]
fn collect_events(input: &str) -> Vec<crate::ParseEvent> {
    let mut listener = crate::PushToEvents::new();
    parse(input, &mut listener);
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
