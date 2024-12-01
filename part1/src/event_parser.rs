use crate::{ParseError, ParseEvent, ParseEventKind};

/// Parses input to [ParseEvent]s.
pub fn parse_events(input: &str) -> EventParser {
    EventParser::new(input)
}

/// A parser that generates [ParseEvent]s.
#[derive(Debug)]
pub struct EventParser<'a> {
    input: &'a str,
    byte_offset: usize,
    container_stack: Vec<Container>,
    state: ParserState,
}

impl<'a> EventParser<'a> {
    fn new(input: &'a str) -> EventParser<'a> {
        EventParser {
            input,
            byte_offset: 0,
            container_stack: vec![],
            state: ParserState::TopLevel,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Container {
    Array,
    Object,
}

#[derive(Debug)]
enum ParserState {
    /// Parse any kind of object, update state based on the current container.
    TopLevel,

    /// Finished parsing a top-level object, expect end-of-input.
    Done,

    /// Parsing an object, parse another element on ',', or finish the array on '}'.
    ObjectExpectComma,

    /// Parsing an object, parse the first element, or finish the array on ']'.
    ObjectExpectKeyValue,

    /// Parsing an object and we've just parsed a key, expect ':'.
    ObjectExpectColon,

    /// Parsing an array, parse another element on ',', or finish the array on ']'.
    ArrayExpectComma,
}

impl<'a> Iterator for EventParser<'a> {
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

macro_rules! skip_trivia {
    ($self:ident) => {
        match $self.skip_trivia() {
            Ok(None) => {}
            Ok(Some(ev)) => return Some(Ok(ev)),
            Err(err) => return Some(Err(err)),
        }
    };
}

impl<'a> EventParser<'a> {
    fn top_level(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        debug_assert!(self.byte_offset <= self.input.len());
        skip_trivia!(self);
        let mut input = self.input[self.byte_offset..].chars().peekable();
        match input.peek().copied() {
            Some('[') => {
                // Consume '['
                let loc = self.byte_offset;
                self.byte_offset += 1;
                self.state = ParserState::TopLevel;
                self.container_stack.push(Container::Array);
                Some(Ok(ParseEvent::new(loc, ParseEventKind::StartArray)))
            }

            Some(']') => {
                if let Err(err) = self.pop_array() {
                    return Some(Err(err));
                }
                let loc = self.byte_offset;
                self.byte_offset += 1;
                self.update_state();
                Some(Ok(ParseEvent::new(loc, ParseEventKind::EndArray)))
            }

            Some('{') => {
                // Consume '{'
                let loc = self.byte_offset;
                self.byte_offset += 1;
                self.state = ParserState::ObjectExpectKeyValue;
                self.container_stack.push(Container::Object);
                Some(Ok(ParseEvent::new(loc, ParseEventKind::StartObject)))
            }

            Some('t') => {
                input.next();
                if input.next() == Some('r')
                    && input.next() == Some('u')
                    && input.next() == Some('e')
                {
                    let loc = self.byte_offset;
                    self.byte_offset += 4;
                    self.update_state();
                    return Some(Ok(ParseEvent::new(loc, ParseEventKind::Bool(true))));
                }

                Some(Err(ParseError {
                    byte_offset: self.byte_offset,
                    reason: "unexpected keyword",
                }))
            }

            Some('f') => {
                input.next();
                if input.next() == Some('a')
                    && input.next() == Some('l')
                    && input.next() == Some('s')
                    && input.next() == Some('e')
                {
                    let loc = self.byte_offset;
                    self.byte_offset += 5;
                    self.update_state();
                    return Some(Ok(ParseEvent::new(loc, ParseEventKind::Bool(false))));
                }

                Some(Err(ParseError {
                    byte_offset: self.byte_offset,
                    reason: "unexpected keyword",
                }))
            }

            Some('n') => {
                input.next();
                if input.next() == Some('u')
                    && input.next() == Some('l')
                    && input.next() == Some('l')
                {
                    let loc = self.byte_offset;
                    self.byte_offset += 4;
                    self.update_state();
                    return Some(Ok(ParseEvent::new(loc, ParseEventKind::Null)));
                }

                Some(Err(ParseError {
                    byte_offset: self.byte_offset,
                    reason: "unexpected keyword",
                }))
            }

            Some(c) if c.is_ascii_digit() => {
                let loc = self.byte_offset;
                input.next();
                self.byte_offset += 1;

                let mut i: u64 = u64::from((c as u8) - b'0');

                while let Some(next) = input.peek().copied() {
                    if !next.is_ascii_digit() {
                        break;
                    }

                    // Consume the digit.
                    self.byte_offset += 1;
                    input.next();

                    // Ignore overflows for the purposes of this post.
                    i *= 10;
                    i += u64::from((next as u8) - b'0');
                }

                self.update_state();

                Some(Ok(ParseEvent::new(loc, ParseEventKind::Int(i))))
            }

            Some('"') => {
                self.byte_offset += 1;
                let loc = self.byte_offset;
                self.update_state();
                match self.skip_string() {
                    Ok(()) => {
                        let after_string = self.byte_offset;
                        Some(Ok(ParseEvent::new(
                            loc,
                            ParseEventKind::Str {
                                size_in_bytes: after_string - loc - 1,
                            },
                        )))
                    }
                    Err(err) => Some(Err(err)),
                }
            }

            Some(_) => Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected character",
            })),

            None => Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected end of input",
            })),
        }
    }

    fn done(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        skip_trivia!(self);
        if self.byte_offset == self.input.len() {
            None
        } else {
            Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "trailing characters",
            }))
        }
    }

    fn array_expect_comma(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        skip_trivia!(self);
        match self.input[self.byte_offset..].chars().next() {
            Some(',') => {
                self.byte_offset += 1;
                self.state = ParserState::TopLevel;
                self.next()
            }

            Some(']') => {
                if let Err(err) = self.pop_array() {
                    return Some(Err(err));
                }
                let loc = self.byte_offset;
                self.byte_offset += 1;
                self.update_state();
                Some(Ok(ParseEvent::new(loc, ParseEventKind::EndArray)))
            }

            Some(_) => Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected character while parsing array",
            })),

            None => Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected end of input while parsing array",
            })),
        }
    }

    fn object_expect_key_value(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        skip_trivia!(self);
        match self.input[self.byte_offset..].chars().next() {
            Some('}') => {
                if let Err(err) = self.pop_map() {
                    return Some(Err(err));
                }
                let loc = self.byte_offset;
                self.byte_offset += 1;
                self.update_state();
                Some(Ok(ParseEvent::new(loc, ParseEventKind::EndObject)))
            }

            Some('"') => {
                self.byte_offset += 1;
                let loc = self.byte_offset;
                match self.skip_string() {
                    Ok(()) => {
                        let after_string = self.byte_offset;
                        self.state = ParserState::ObjectExpectColon;
                        Some(Ok(ParseEvent::new(
                            loc,
                            ParseEventKind::Str {
                                size_in_bytes: after_string - loc - 1,
                            },
                        )))
                    }
                    Err(err) => Some(Err(err)),
                }
            }

            Some(_) => Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected character while parsing object",
            })),

            None => Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected end of input while parsing object",
            })),
        }
    }

    fn object_expect_colon(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        skip_trivia!(self);
        match self.input[self.byte_offset..].chars().next() {
            Some(':') => {
                self.byte_offset += 1;
                self.state = ParserState::TopLevel;
                self.next()
            }

            Some(_) => Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected character while parsing object",
            })),

            None => Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected end of input while parsing object",
            })),
        }
    }

    fn object_expect_comma(&mut self) -> Option<Result<ParseEvent, ParseError>> {
        skip_trivia!(self);
        match self.input[self.byte_offset..].chars().next() {
            Some(',') => {
                self.byte_offset += 1;
                self.state = ParserState::ObjectExpectKeyValue;
                self.next()
            }

            Some('}') => {
                if let Err(err) = self.pop_map() {
                    return Some(Err(err));
                }
                let loc = self.byte_offset;
                self.byte_offset += 1;
                self.update_state();
                Some(Ok(ParseEvent::new(loc, ParseEventKind::EndObject)))
            }

            Some(_) => Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected character while parsing object",
            })),

            None => Some(Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected end of input while parsing object",
            })),
        }
    }

    /// Skip until after the end of a string. Expects the opening double colon to be consumed.
    fn skip_string(&mut self) -> Result<(), ParseError> {
        self.skip_trivia()?;
        for char in self.input[self.byte_offset..].chars() {
            self.byte_offset += 1;
            if char == '"' {
                return Ok(());
            }
        }
        Err(ParseError {
            byte_offset: self.byte_offset,
            reason: "unexpected end of input while parsing string",
        })
    }

    /// After parsing a value, update the parser state based on the current container.
    fn update_state(&mut self) {
        self.state = match self.container_stack.last() {
            Some(Container::Array) => ParserState::ArrayExpectComma,
            Some(Container::Object) => ParserState::ObjectExpectComma,
            None => ParserState::Done,
        };
    }

    fn pop_map(&mut self) -> Result<(), ParseError> {
        match self.container_stack.pop() {
            Some(Container::Object) => Ok(()),

            _ => Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected '}'",
            }),
        }
    }

    fn pop_array(&mut self) -> Result<(), ParseError> {
        match self.container_stack.pop() {
            Some(Container::Array) => Ok(()),

            _ => Err(ParseError {
                byte_offset: self.byte_offset,
                reason: "unexpected ']'",
            }),
        }
    }

    fn skip_trivia(&mut self) -> Result<Option<ParseEvent>, ParseError> {
        if self.byte_offset == self.input.len() {
            return Ok(None);
        }
        let mut chars = self.input[self.byte_offset..].char_indices().peekable();
        loop {
            match chars.peek().copied() {
                Some((byte_idx, '/')) => {
                    chars.next(); // consume peeked '/'
                    match chars.next() {
                        Some((_, '/')) => loop {
                            match chars.next() {
                                Some((newline_byte_idx, '\n')) => {
                                    self.byte_offset += newline_byte_idx;
                                    return Ok(Some(ParseEvent {
                                        kind: ParseEventKind::Comment {
                                            size_in_bytes: newline_byte_idx - byte_idx + 1,
                                        },
                                        byte_offset: byte_idx,
                                    }));
                                }
                                Some(_) => {}
                                None => {
                                    return Err(ParseError {
                                        byte_offset: byte_idx,
                                        reason: "unterminated comment",
                                    });
                                }
                            }
                        },
                        _ => {
                            return Err(ParseError {
                                byte_offset: byte_idx,
                                reason: "unexpected '/'",
                            });
                        }
                    }
                }

                Some((_, c)) if c.is_ascii_whitespace() => {
                    chars.next(); // consume peeked whitespace
                }

                Some((byte_idx, _)) => {
                    self.byte_offset += byte_idx;
                    return Ok(None);
                }

                None => {
                    self.byte_offset = self.input.len();
                    return Ok(None);
                }
            }
        }
    }
}

#[cfg(test)]
fn collect_events(input: &str) -> (Vec<ParseEventKind>, Option<ParseError>) {
    let mut events: Vec<ParseEventKind> = vec![];
    for event in EventParser::new(input) {
        match event {
            Ok(event) => events.push(event.kind),
            Err(err) => return (events, Some(err)),
        }
    }
    (events, None)
}

#[test]
fn event_tests() {
    for (str, events) in crate::test_common::event_tests() {
        println!("Parsing {:?}", str);
        let (events_, error) = collect_events(&str);
        assert_eq!(events_, events);
        assert_eq!(error, None);
    }
}
