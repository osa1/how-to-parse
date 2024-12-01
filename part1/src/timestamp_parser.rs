use crate::{parse_events, EventListener, ParseError, ParseEvent, ParseEventKind};

/// Parse the "timestamp" field at the top-level map of the JSON.
pub fn parse_timestamp(log_line: &str) -> Result<Option<u64>, ParseError> {
    let mut container_depth: u32 = 0;
    let mut expect_timestamp = false;

    for event in parse_events(log_line) {
        let ParseEvent { kind, byte_offset } = match event {
            Ok(event) => event,
            Err(err) => return Err(err),
        };

        let expect_timestamp_ = expect_timestamp;
        expect_timestamp = false;

        match kind {
            ParseEventKind::StartObject => {
                container_depth += 1;
            }

            ParseEventKind::EndObject => {
                container_depth -= 1;
            }

            ParseEventKind::StartArray => {
                if container_depth == 0 {
                    // Array at the top level, the line does not contain the field.
                    return Ok(None);
                }
                container_depth += 1;
            }

            ParseEventKind::EndArray => {
                container_depth -= 1;
            }

            ParseEventKind::Str { size_in_bytes } => {
                if container_depth != 1 {
                    continue;
                }
                let str = &log_line[byte_offset..byte_offset + size_in_bytes];
                expect_timestamp = str == "timestamp";
            }

            ParseEventKind::Int(i) => {
                if expect_timestamp_ {
                    return Ok(Some(i));
                }
            }

            ParseEventKind::Bool(_) | ParseEventKind::Null | ParseEventKind::Comment { .. } => {}
        }
    }

    Ok(None)
}

/// A timestamp parser similar to [parse_timestamp], but implements [EventListener].
pub struct TimestampParserListener<'a> {
    container_depth: u32,

    /// Whether the next `Int` event is the timestamp. Set after seeing a `Str("timestamp")` at
    /// container depth 1.
    expect_timestamp: bool,

    input: &'a str,

    /// The parsed value.
    timestamp_value: Option<u64>,

    error: Option<ParseError>,
}

impl<'a> TimestampParserListener<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            container_depth: 0,
            expect_timestamp: false,
            input,
            timestamp_value: None,
            error: None,
        }
    }
}

impl<'a> EventListener for TimestampParserListener<'a> {
    fn handle_start_object(&mut self, _byte_offset: usize) -> bool {
        self.container_depth += 1;
        true
    }

    fn handle_end_object(&mut self, _byte_offset: usize) -> bool {
        self.container_depth -= 1;
        true
    }

    fn handle_start_array(&mut self, _byte_offset: usize) -> bool {
        if self.container_depth == 0 {
            // Array at the top level, the line does not contain the field.
            return false;
        }
        self.container_depth += 1;
        true
    }

    fn handle_end_array(&mut self, _byte_offset: usize) -> bool {
        self.container_depth -= 1;
        true
    }

    fn handle_str(&mut self, byte_offset: usize, size_in_bytes: usize) -> bool {
        if self.container_depth == 1 {
            let str = &self.input[byte_offset..byte_offset + size_in_bytes];
            self.expect_timestamp = str == "timestamp";
        }
        true
    }

    fn handle_int(&mut self, _byte_offset: usize, i: u64) -> bool {
        if self.expect_timestamp {
            self.timestamp_value = Some(i);
            false
        } else {
            true
        }
    }

    fn handle_error(&mut self, error: ParseError) {
        self.error = Some(error);
    }
}

#[test]
fn parse_timestamp_test() {
    assert_eq!(parse_timestamp(r#"{"timestamp":123}"#), Ok(Some(123)));
    assert_eq!(
        parse_timestamp(r#"{"x":[],"timestamp":123}"#),
        Ok(Some(123))
    );
    assert_eq!(
        parse_timestamp(r#"{"x":["timestamp",999],"timestamp":123}"#),
        Ok(Some(123))
    );
}

#[test]
fn parse_timestamp_listener_test() {
    fn parse(input: &str) -> u64 {
        let mut listener = TimestampParserListener::new(input);
        crate::parse_events_push(input, &mut listener);
        assert_eq!(listener.error, None);
        listener.timestamp_value.unwrap()
    }

    assert_eq!(parse(r#"{"timestamp":123}"#), 123);
    assert_eq!(parse(r#"{"x":[],"timestamp":123}"#), 123);
    assert_eq!(parse(r#"{"x":["timestamp",999],"timestamp":123}"#), 123);
}
