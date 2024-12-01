use crate::{EventListener, ParseError};

use std::iter::Peekable;
use std::str::CharIndices;

/// Parse input to events, call [EventListener] callbacks with the events.
pub fn parse<L: EventListener>(input: &str, listener: &mut L) {
    let mut iter = input.char_indices().peekable();
    let input_size = input.len();

    if !parse_single(&mut iter, input_size, listener) {
        return;
    }

    skip_trivia(&mut iter, listener);

    if let Some((byte_offset, _)) = iter.next() {
        // We should return the parsed object with this error, but it's OK for the purposes of this
        // post.
        listener.handle_error(ParseError {
            byte_offset,
            reason: "trailing characters after parsing",
        });
    }
}

macro_rules! skip_trivia {
    ($iter:expr, $listener:expr) => {
        if !skip_trivia($iter, $listener) {
            return false;
        }
    };
}

fn parse_single<L: EventListener>(
    iter: &mut Peekable<CharIndices>,
    input_size: usize,
    listener: &mut L,
) -> bool {
    skip_trivia!(iter, listener);

    let (byte_offset, char) = match iter.next() {
        Some(next) => next,
        None => {
            listener.handle_error(ParseError {
                byte_offset: input_size,
                reason: "unexpected end of input",
            });
            return false;
        }
    };

    if char == '[' {
        listener.handle_start_array(byte_offset);
        let mut array_is_empty = true;
        loop {
            skip_trivia!(iter, listener);

            match iter.peek().copied() {
                Some((comma_byte_offset, ',')) => {
                    if array_is_empty {
                        listener.handle_error(ParseError {
                            byte_offset: comma_byte_offset,
                            reason: "unexpected character while parsing array",
                        });
                        return false;
                    }

                    // Consume ','
                    iter.next();
                    if !parse_single(iter, input_size, listener) {
                        return false;
                    }
                }

                Some((_, ']')) => {
                    // Consume ']'
                    iter.next();
                    listener.handle_end_array(byte_offset);
                    return true;
                }

                Some((byte_offset, _)) => {
                    if !array_is_empty {
                        // Need to see a ',' before the next element.
                        listener.handle_error(ParseError {
                            byte_offset,
                            reason: "unexpected character while parsing array",
                        });
                        return false;
                    }

                    if !parse_single(iter, byte_offset, listener) {
                        return false;
                    }

                    array_is_empty = false;
                }

                None => {
                    listener.handle_error(ParseError {
                        byte_offset: input_size,
                        reason: "end of input while parsing array",
                    });
                    return false;
                }
            }
        }
    }

    if char == '{' {
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
            skip_trivia!(iter, listener);

            match std::mem::replace(&mut state, State::Done) {
                State::Done => {
                    match iter.peek().copied() {
                        Some((byte_offset, ',')) => {
                            if object_is_empty {
                                listener.handle_error(ParseError {
                                    byte_offset,
                                    reason: "unexpected comma while parsing object",
                                });
                                return false;
                            }
                            iter.next(); // consume ','
                            state = State::ExpectKey;
                        }

                        Some((_, '}')) => {
                            iter.next(); // consume '}'
                            listener.handle_end_object(byte_offset);
                            return true;
                        }

                        Some((_, '"')) => {
                            if !parse_single(iter, byte_offset, listener) {
                                return false;
                            }
                            state = State::ExpectColon;
                        }

                        Some((byte_offset, _)) => {
                            listener.handle_error(ParseError {
                                byte_offset,
                                reason: "unexpected char while parsing object",
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
                    if !parse_string(iter, input_size, listener) {
                        return false;
                    }
                    state = State::ExpectColon;
                }

                State::ExpectColon => match iter.next() {
                    Some((_, ':')) => {
                        state = State::ExpectValue;
                    }

                    Some((byte_offset, _)) => {
                        listener.handle_error(ParseError {
                            byte_offset,
                            reason: "unexpected char while parsing object",
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
                },

                State::ExpectValue => {
                    if !parse_single(iter, input_size, listener) {
                        return false;
                    }
                    object_is_empty = false;
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
            listener.handle_bool(byte_offset, true);
            return true;
        }
        listener.handle_error(ParseError {
            byte_offset,
            reason: "unexpected keyword",
        });
        return false;
    }

    if char == 'f' {
        if next_char(iter) == Some('a')
            && next_char(iter) == Some('l')
            && next_char(iter) == Some('s')
            && next_char(iter) == Some('e')
        {
            listener.handle_bool(byte_offset, false);
            return true;
        }
        listener.handle_error(ParseError {
            byte_offset,
            reason: "unexpected keyword",
        });
        return false;
    }

    if char == 'n' {
        if next_char(iter) == Some('u')
            && next_char(iter) == Some('l')
            && next_char(iter) == Some('l')
        {
            listener.handle_null(byte_offset);
            return true;
        }
        listener.handle_error(ParseError {
            byte_offset,
            reason: "unexpected keyword",
        });
        return false;
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

        listener.handle_int(byte_offset, i);
        return true;
    }

    if char == '"' {
        for (byte_offset_, next) in iter.by_ref() {
            if next == '"' {
                listener.handle_str(byte_offset + 1, byte_offset_ - byte_offset - 1);
                return true;
            }
        }

        listener.handle_error(ParseError {
            byte_offset: input_size,
            reason: "unexpected end of input while parsing string",
        });
        return false;
    }

    listener.handle_error(ParseError {
        byte_offset,
        reason: "unexpected character",
    });
    false
}

fn parse_string<L: EventListener>(
    iter: &mut Peekable<CharIndices>,
    input_size: usize,
    listener: &mut L,
) -> bool {
    let (byte_offset, char) = match iter.next() {
        Some(next) => next,
        None => {
            listener.handle_error(ParseError {
                byte_offset: input_size,
                reason: "unexpected end of input",
            });
            return false;
        }
    };

    if char == '"' {
        for (byte_offset_, next) in iter.by_ref() {
            if next == '"' {
                listener.handle_str(byte_offset + 1, byte_offset_ - byte_offset - 1);
                return true;
            }
        }
    }

    listener.handle_error(ParseError {
        byte_offset: input_size,
        reason: "unexpected end of input while parsing string",
    });
    false
}

fn next_char(iter: &mut Peekable<CharIndices>) -> Option<char> {
    iter.next().map(|(_, char)| char)
}

fn skip_trivia<L: EventListener>(iter: &mut Peekable<CharIndices>, listener: &mut L) -> bool {
    'outer: while let Some((byte_offset, char)) = iter.peek().copied() {
        if char.is_ascii_whitespace() {
            iter.next(); // consume peeked whitespace
            continue;
        }

        if char == '/' {
            iter.next(); // consume peeked '/'
            match iter.next() {
                Some((_, '/')) => {
                    for (byte_offset_, char) in iter.by_ref() {
                        if char == '\n' {
                            listener.handle_comment(byte_offset, byte_offset_ - byte_offset + 1);
                            continue 'outer;
                        }
                    }
                }

                Some(_) => {
                    listener.handle_error(ParseError {
                        byte_offset,
                        reason: "unexpected '/'",
                    });
                    return false;
                }

                None => {
                    listener.handle_error(ParseError {
                        byte_offset,
                        reason: "unexpected end of input",
                    });
                    return false;
                }
            }
        }

        break;
    }

    true
}
