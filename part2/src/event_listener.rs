use crate::ParseError;

/// The trait for push-based event parsing. Used by
/// [parse_events_push][crate::event_push_parser::parse].
///
/// Methods return a `bool` for whether to keep parsing.
///
/// Default implementations of the methods return `true` to continue parsing.
pub trait EventListener {
    fn handle_start_object(&mut self, _byte_offset: usize) -> bool {
        true
    }

    fn handle_end_object(&mut self, _byte_offset: usize) -> bool {
        true
    }

    fn handle_start_array(&mut self, _byte_offset: usize) -> bool {
        true
    }

    fn handle_end_array(&mut self, _byte_offset: usize) -> bool {
        true
    }

    fn handle_int(&mut self, _byte_offset: usize, _i: u64) -> bool {
        true
    }

    fn handle_str(&mut self, _byte_offset: usize, _size_in_bytes: usize) -> bool {
        true
    }

    fn handle_bool(&mut self, _byte_offset: usize, _b: bool) -> bool {
        true
    }

    fn handle_null(&mut self, _byte_offset: usize) -> bool {
        true
    }

    fn handle_comment(&mut self, _byte_offset: usize, _size_in_bytes: usize) -> bool {
        true
    }

    fn handle_error(&mut self, _error: ParseError);
}
