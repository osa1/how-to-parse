/// A parse event, with location of the event in the input.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseEvent {
    pub kind: ParseEventKind,
    pub byte_offset: usize,
}

/// Details of a parse event.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseEventKind {
    StartObject,
    EndObject,
    StartArray,
    EndArray,
    Int(u64),
    Str {
        /// Size of the string, not including the double quotes.
        size_in_bytes: usize,
    },
    Bool(bool),
    Null,
    Comment {
        /// Size of the comment, including the "//" a the beginning and newline at the end.
        size_in_bytes: usize,
    },
}

impl ParseEvent {
    pub(crate) fn new(byte_offset: usize, kind: ParseEventKind) -> ParseEvent {
        ParseEvent { byte_offset, kind }
    }
}
