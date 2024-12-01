use crate::recursive_descent::{next_char, skip_trivia};
use crate::{Json, ParseError};

use std::iter::Peekable;
use std::str::CharIndices;

pub fn parse(input: &str) -> Result<Json, ParseError> {
    let mut iter = input.char_indices().peekable();
    let json = parse_single(&mut iter, input)?;
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum ParserState {
    /// Parse any kind of object, update state based on the current container.
    TopLevel,

    /// Parsing a container, parse another element on ',', or finish the
    /// container on ']' or '}'.
    ExpectComma,

    /// Parsing an object, parse a key.
    ObjectExpectKeyValue,

    /// Parsing an object, parse a key, or terminate the object.
    ObjectExpectKeyValueTerminate,

    /// Parsing an object and we've just parsed a key, expect ':'.
    ObjectExpectColon,
}

fn parse_single(iter: &mut Peekable<CharIndices>, input: &str) -> Result<Json, ParseError> {
    let mut container_stack: Vec<Container> = vec![];
    let mut state = ParserState::TopLevel;

    loop {
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

        match state {
            ParserState::TopLevel => match char {
                '{' => {
                    container_stack.push(Container::new_map());
                    state = ParserState::ObjectExpectKeyValueTerminate;
                }

                '[' => {
                    container_stack.push(Container::new_array());
                    state = ParserState::TopLevel;
                }

                ']' => match container_stack.pop() {
                    Some(Container::Array(elems)) => {
                        let object = Json::Array(elems);
                        match container_stack.last_mut() {
                            Some(container) => {
                                container.add_json(object);
                                state = ParserState::ExpectComma;
                            }
                            None => return Ok(object),
                        }
                    }
                    Some(Container::Map(_)) | None => {
                        return Err(ParseError {
                            byte_offset,
                            reason: "unexpected character",
                        })
                    }
                },

                't' if next_char(iter) == Some('r')
                    && next_char(iter) == Some('u')
                    && next_char(iter) == Some('e') =>
                {
                    let object = Json::Bool(true);
                    match container_stack.last_mut() {
                        Some(container) => {
                            container.add_json(object);
                            state = ParserState::ExpectComma;
                        }
                        None => return Ok(object),
                    }
                }

                'f' if next_char(iter) == Some('a')
                    && next_char(iter) == Some('l')
                    && next_char(iter) == Some('s')
                    && next_char(iter) == Some('e') =>
                {
                    let object = Json::Bool(false);
                    match container_stack.last_mut() {
                        Some(container) => {
                            container.add_json(object);
                            state = ParserState::ExpectComma;
                        }
                        None => return Ok(object),
                    }
                }

                'n' if next_char(iter) == Some('u')
                    && next_char(iter) == Some('l')
                    && next_char(iter) == Some('l') =>
                {
                    let object = Json::Null;
                    match container_stack.last_mut() {
                        Some(container) => {
                            container.add_json(object);
                            state = ParserState::ExpectComma;
                        }
                        None => return Ok(object),
                    }
                }

                c if c.is_ascii_digit() => {
                    let mut i: u64 = u64::from((c as u8) - b'0');

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

                    let object = Json::Int(i);
                    match container_stack.last_mut() {
                        Some(container) => container.add_json(object),
                        None => return Ok(object),
                    }
                    state = ParserState::ExpectComma;
                }

                '"' => {
                    let string = parse_string(input, byte_offset, iter)?;
                    let object = Json::String(string);
                    match container_stack.last_mut() {
                        Some(container) => {
                            container.add_json(object);
                            state = ParserState::ExpectComma;
                        }
                        None => return Ok(object),
                    }
                }

                _ => {
                    return Err(ParseError {
                        byte_offset,
                        reason: "unexpected character",
                    })
                }
            },

            ParserState::ExpectComma => match char {
                ',' => match container_stack.last() {
                    Some(Container::Array(_)) => state = ParserState::TopLevel,
                    Some(Container::Map(_)) => state = ParserState::ObjectExpectKeyValue,
                    None => unreachable!(),
                },

                '}' => {
                    let container = container_stack.pop().unwrap();

                    let map = match container {
                        Container::Array(_) => {
                            return Err(ParseError {
                                byte_offset,
                                reason: "unexpected '}'",
                            })
                        }
                        Container::Map(map) => map,
                    };

                    let object = map.finish(byte_offset)?;

                    match container_stack.last_mut() {
                        Some(container) => {
                            container.add_json(object);
                            state = ParserState::ExpectComma;
                        }
                        None => return Ok(object),
                    }
                }

                ']' => {
                    let container = container_stack.pop().unwrap();

                    match container {
                        Container::Array(array) => {
                            let object = Json::Array(array);
                            match container_stack.last_mut() {
                                Some(container) => {
                                    container.add_json(object);
                                    state = ParserState::ExpectComma;
                                }
                                None => return Ok(object),
                            }
                        }
                        Container::Map(_) => {
                            return Err(ParseError {
                                byte_offset,
                                reason: "unexpected '}'",
                            })
                        }
                    }
                }

                _ => {
                    return Err(ParseError {
                        byte_offset,
                        reason: "unexpected character",
                    })
                }
            },

            ParserState::ObjectExpectKeyValue => match char {
                '"' => {
                    let string = parse_string(input, byte_offset, iter)?;
                    let object = Json::String(string);
                    container_stack.last_mut().unwrap().add_json(object);
                    state = ParserState::ObjectExpectColon;
                }

                _ => {
                    return Err(ParseError {
                        byte_offset,
                        reason: "unexpected character",
                    })
                }
            },

            ParserState::ObjectExpectKeyValueTerminate => match char {
                '"' => {
                    let string = parse_string(input, byte_offset, iter)?;
                    let object = Json::String(string);
                    container_stack.last_mut().unwrap().add_json(object);
                    state = ParserState::ObjectExpectColon;
                }

                '}' => {
                    let object = container_stack
                        .pop()
                        .unwrap()
                        .into_map()
                        .finish(byte_offset)?;
                    match container_stack.last_mut() {
                        Some(container) => {
                            container.add_json(object);
                            state = ParserState::ExpectComma;
                        }
                        None => return Ok(object),
                    }
                }

                _ => {
                    return Err(ParseError {
                        byte_offset,
                        reason: "unexpected character",
                    })
                }
            },

            ParserState::ObjectExpectColon => match char {
                ':' => {
                    state = ParserState::TopLevel;
                }

                _ => {
                    return Err(ParseError {
                        byte_offset,
                        reason: "unexpected character",
                    })
                }
            },
        }
    }
}

// NB. Initial double quote should be consumed in `iter`, but not in
// `byte_offset`.
fn parse_string(
    input: &str,
    byte_offset: usize,
    iter: &mut Peekable<CharIndices>,
) -> Result<String, ParseError> {
    for (next_byte_offset, next) in iter.by_ref() {
        if next == '"' {
            let string = input[byte_offset + 1..next_byte_offset].to_string();
            return Ok(string);
        }
    }

    return Err(ParseError {
        byte_offset: input.len(),
        reason: "unexpected end of input while parsing string",
    });
}

enum Container {
    Array(Vec<Json>),
    Map(MapInProgress),
}

struct MapInProgress {
    built: Vec<(String, Json)>,
    next: Option<String>,
}

impl Container {
    fn new_map() -> Container {
        Container::Map(MapInProgress {
            built: vec![],
            next: None,
        })
    }

    fn new_array() -> Container {
        Container::Array(vec![])
    }

    fn into_map(self) -> MapInProgress {
        match self {
            Container::Array(_) => panic!(),
            Container::Map(map) => map,
        }
    }

    fn add_json(&mut self, object: Json) {
        match self {
            Container::Array(array) => array.push(object),
            Container::Map(map) => map.add_json(object),
        }
    }
}

impl MapInProgress {
    fn add_json(&mut self, object: Json) {
        match self.next.take() {
            Some(key) => {
                self.built.push((key, object));
            }
            None => {
                self.next = Some(object.into_string());
            }
        }
    }

    fn finish(self, byte_offset: usize) -> Result<Json, ParseError> {
        let MapInProgress { built, next } = self;
        if next.is_some() {
            Err(ParseError {
                byte_offset,
                reason: "unexpected '}'",
            })
        } else {
            Ok(Json::Object(built))
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

#[test]
fn event_to_tree_random_tests() {
    for input_size in [10, 100, 1_000, 2_000, 5_000, 10_000, 10_000_000] {
        let input = crate::gen_input(input_size);
        let ast_1 = crate::recursive_descent::parse(&input).unwrap();
        let ast_2 = parse(&input).unwrap();
        assert_eq!(ast_1, ast_2);
    }
}
