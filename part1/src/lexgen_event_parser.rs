use crate::event_parser::Container;
use crate::{ParseError, ParseEvent, ParseEventKind};

use std::str::FromStr;

use lexgen_util::{LexerError, LexerErrorKind};

/// Parses input to [ParseEvent]s, using [lexgen].
pub fn parse_events(input: &str) -> LexgenIteratorAdapter {
    LexgenIteratorAdapter {
        lexer: Lexer::new(input),
    }
}

pub struct LexgenIteratorAdapter<'a> {
    lexer: Lexer<'a, std::str::Chars<'a>>,
}

impl<'a> Iterator for LexgenIteratorAdapter<'a> {
    type Item = Result<ParseEvent, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.lexer.next()? {
            Ok((_, ev, _)) => Ok(ev),
            Err(LexerError {
                location: loc,
                kind,
            }) => Err(ParseError {
                byte_offset: loc.byte_idx,
                reason: match kind {
                    LexerErrorKind::InvalidToken => "invalid token",
                    LexerErrorKind::Custom(reason) => reason,
                },
            }),
        })
    }
}

#[derive(Debug, Default)]
struct LexerState {
    container_stack: Vec<Container>,
}

lexgen::lexer! {
    Lexer(LexerState) -> ParseEvent;

    type Error = &'static str;

    let comment = "//" (_ # '\n')* '\n';

    rule Init {
        $$ascii_whitespace,

        $comment => comment,

        '[' => |lexer| {
            let (loc, _) = lexer.match_loc();
            lexer.state().container_stack.push(Container::Array);
            lexer.return_(ParseEvent::new(loc.byte_idx, ParseEventKind::StartArray))
        },

        ']' =? |lexer| {
            let (loc, _) = lexer.match_loc();
            lexer.reset_match();
            if let Some(Container::Array) = lexer.state().container_stack.pop() {
                update_state(lexer);
                lexer.return_(Ok(ParseEvent::new(loc.byte_idx, ParseEventKind::EndArray)))
            } else {
                lexer.return_(Err("unexpected ']'"))
            }
        },

        '{' => |lexer| {
            let (loc, _) = lexer.match_loc();
            lexer.state().container_stack.push(Container::Object);
            lexer.switch::<()>(LexerRule::ObjectExpectKeyValue);
            lexer.return_(ParseEvent::new(loc.byte_idx, ParseEventKind::StartObject))
        },

        "true" => |lexer| {
            let (loc, _) = lexer.match_loc();
            update_state(lexer);
            lexer.return_(ParseEvent::new(loc.byte_idx, ParseEventKind::Bool(true)))
        },

        "false" => |lexer| {
            let (loc, _) = lexer.match_loc();
            update_state(lexer);
            lexer.return_(ParseEvent::new(loc.byte_idx, ParseEventKind::Bool(false)))
        },

        "null" => |lexer| {
            let (loc, _) = lexer.match_loc();
            update_state(lexer);
            lexer.return_(ParseEvent::new(loc.byte_idx, ParseEventKind::Null))
        },

        // Ignore overflows.
        ['0'-'9']+ => |lexer| {
            let (loc, _) = lexer.match_loc();
            update_state(lexer);
            lexer.return_(ParseEvent::new(loc.byte_idx, ParseEventKind::Int(u64::from_str(lexer.match_()).unwrap())))
        },

        '"' (_ # '"')* '"' => |lexer| {
            let (match_start, match_end) = lexer.match_loc();
            update_state(lexer);
            lexer.return_(ParseEvent::new(
                match_start.byte_idx + 1,
                ParseEventKind::Str {
                    size_in_bytes: match_end.byte_idx - match_start.byte_idx - 2
                },
            ))
        },
    }

    rule Done {
        $$ascii_whitespace,

        $comment => comment,

        $,

        _ =? |lexer| lexer.return_(Err("trailing characters")),
    }

    rule ArrayExpectComma {
        $$ascii_whitespace,

        $comment => comment,

        ',' => |lexer| {
            lexer.reset_match();
            lexer.switch(LexerRule::Init)
        },

        ']' => |lexer| {
            let (loc, _) = lexer.match_loc();
            let state = lexer.state().container_stack.pop();
            debug_assert_eq!(state, Some(Container::Array));
            update_state(lexer);
            lexer.return_(ParseEvent::new(loc.byte_idx, ParseEventKind::EndArray))
        },

        _ =? |lexer|
            lexer.return_(Err("unexpected character or end of input while parsing array")),
    }

    rule ObjectExpectKeyValue {
        $$ascii_whitespace,

        $comment => comment,

        '}' => |lexer| {
            let (loc, _) = lexer.match_loc();
            let state = lexer.state().container_stack.pop();
            debug_assert_eq!(state, Some(Container::Object));
            update_state(lexer);
            lexer.return_(ParseEvent::new(loc.byte_idx, ParseEventKind::EndObject))
        },

        '"' (_ # '"')* '"' => |lexer| {
            let (match_start, match_end) = lexer.match_loc();
            lexer.switch::<()>(LexerRule::ObjectExpectColon);
            lexer.return_(ParseEvent::new(match_start.byte_idx + 1, ParseEventKind::Str {
                size_in_bytes: match_end.byte_idx - match_start.byte_idx - 2
            }))
        },

        _ =? |lexer|
            lexer.return_(Err("unexpected character or end of input while parsing object")),
    }

    rule ObjectExpectColon {
        $$ascii_whitespace,

        $comment => comment,

        ':' => |lexer| {
            lexer.reset_match();
            lexer.switch(LexerRule::Init)
        },

        _ =? |lexer|
            lexer.return_(Err("unexpected character or end of input while parsing object")),
    }

    rule ObjectExpectComma {
        $$ascii_whitespace,

        $comment => comment,

        ',' => |lexer| {
            lexer.reset_match();
            lexer.switch(LexerRule::ObjectExpectKeyValue)
        },

        '}' => |lexer| {
            let (loc, _) = lexer.match_loc();
            let state = lexer.state().container_stack.pop();
            debug_assert_eq!(state, Some(Container::Object));
            update_state(lexer);
            lexer.return_(ParseEvent::new(loc.byte_idx, ParseEventKind::EndObject))
        },
    }
}

fn comment<I: Clone + Iterator<Item = char>>(
    lexer: &mut Lexer<'_, I>,
) -> lexgen_util::SemanticActionResult<ParseEvent> {
    let (match_start, match_end) = lexer.match_loc();
    lexer.return_(ParseEvent::new(
        match_start.byte_idx,
        ParseEventKind::Comment {
            size_in_bytes: match_end.byte_idx - match_start.byte_idx,
        },
    ))
}

/// After parsing a value, update the parser state based on the current container.
fn update_state<I: Clone + Iterator<Item = char>>(lexer: &mut Lexer<'_, I>) {
    let current_container = lexer.state().container_stack.last().copied();
    lexer.switch::<()>(match current_container {
        Some(Container::Array) => LexerRule::ArrayExpectComma,
        Some(Container::Object) => LexerRule::ObjectExpectComma,
        None => LexerRule::Done,
    });
}

#[cfg(test)]
fn collect_events(input: &str) -> (Vec<ParseEventKind>, Option<LexerError<&'static str>>) {
    let mut events: Vec<ParseEventKind> = vec![];
    for event in Lexer::new(input) {
        match event {
            Ok((_, event, _)) => events.push(event.kind),
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
