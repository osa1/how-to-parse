use crate::{EventListener, ParseError, ParseEvent, ParseEventKind};

/// An [EventListener] that collects parse events.
pub struct PushToEvents {
    events: Vec<ParseEvent>,
    error: Option<ParseError>,
}

impl EventListener for PushToEvents {
    fn handle_start_object(&mut self, byte_offset: usize) -> bool {
        self.events
            .push(ParseEvent::new(byte_offset, ParseEventKind::StartObject));
        true
    }

    fn handle_end_object(&mut self, byte_offset: usize) -> bool {
        self.events
            .push(ParseEvent::new(byte_offset, ParseEventKind::EndObject));
        true
    }

    fn handle_start_array(&mut self, byte_offset: usize) -> bool {
        self.events
            .push(ParseEvent::new(byte_offset, ParseEventKind::StartArray));
        true
    }

    fn handle_end_array(&mut self, byte_offset: usize) -> bool {
        self.events
            .push(ParseEvent::new(byte_offset, ParseEventKind::EndArray));
        true
    }

    fn handle_int(&mut self, byte_offset: usize, i: u64) -> bool {
        self.events
            .push(ParseEvent::new(byte_offset, ParseEventKind::Int(i)));
        true
    }

    fn handle_str(&mut self, byte_offset: usize, size_in_bytes: usize) -> bool {
        self.events.push(ParseEvent::new(
            byte_offset,
            ParseEventKind::Str { size_in_bytes },
        ));
        true
    }

    fn handle_bool(&mut self, byte_offset: usize, b: bool) -> bool {
        self.events
            .push(ParseEvent::new(byte_offset, ParseEventKind::Bool(b)));
        true
    }

    fn handle_null(&mut self, byte_offset: usize) -> bool {
        self.events
            .push(ParseEvent::new(byte_offset, ParseEventKind::Null));
        true
    }

    fn handle_comment(&mut self, byte_offset: usize, size_in_bytes: usize) -> bool {
        self.events.push(ParseEvent::new(
            byte_offset,
            ParseEventKind::Comment { size_in_bytes },
        ));
        true
    }

    fn handle_error(&mut self, error: crate::ParseError) {
        self.error = Some(error);
    }
}

impl PushToEvents {
    pub fn new() -> PushToEvents {
        PushToEvents {
            events: vec![],
            error: None,
        }
    }

    // Q: How do we implement `IntoIter` for this type? I don't know (or care)
    // about the concrete iterator type here (hence `impl Iterator`), but we
    // have to explicitly type it in the `IntoIter` implementation.
    pub fn into_iter(self) -> impl Iterator<Item = Result<ParseEvent, ParseError>> {
        self.events.into_iter().map(Ok).chain(self.error.map(Err))
    }

    pub fn into_events(self) -> (Vec<ParseEvent>, Option<ParseError>) {
        (self.events, self.error)
    }
}

#[test]
fn event_tests() {
    for (str, events) in crate::test_common::event_tests() {
        println!("Parsing {:?}", str);
        let mut push_to_events = PushToEvents::new();
        crate::event_push_parser::parse(&str, &mut push_to_events);
        let events_ = push_to_events
            .into_iter()
            .map(|ev| ev.unwrap().kind)
            .collect::<Vec<_>>();
        assert_eq!(events_, events);
    }
}
