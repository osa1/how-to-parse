#![allow(clippy::new_without_default, clippy::should_implement_trait)]

/// Defines the parse event types.
mod event;

/// Defines the listener type, for the "push" parsing.
mod event_listener;

/// Defines the AST without comments and locations.
mod simple_ast;

/// Implements an event parser.
mod event_parser;

/// Implements "push" event parser.
mod event_push_parser;

/// Implement an event parser using `lexgen`.
mod lexgen_event_parser;

/// Implements an event listener that builds simple AST.
mod listener_impl;

/// Implements an AST parser.
mod simple_parser;

/// Implements a parser that extracts timestamps from events, without building an AST.
mod timestamp_parser;

/// Implements generating an AST from an event parser.
mod event_to_tree;

/// Implements collecting parse events from a "push" event parser.
mod push_to_events;

/// Implements input generation for benchmarks.
mod input_gen;

#[cfg(test)]
mod test_common;

pub use event::{ParseEvent, ParseEventKind};
pub use event_listener::EventListener;
pub use event_parser::parse_events;
pub use event_push_parser::parse as parse_events_push;
pub use event_to_tree::event_to_tree;
pub use lexgen_event_parser::parse_events as parse_events_lexgen;
pub use listener_impl::AstBuilderListener;
pub use push_to_events::PushToEvents;
pub use simple_ast::Json;
pub use simple_parser::parse as parse_ast;
pub use timestamp_parser::{parse_timestamp, TimestampParserListener};

#[doc(hidden)]
pub use input_gen::gen_input;

/// A parse error, common for both event and AST parsers.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    /// Byte offset of the parse error in the input.
    pub byte_offset: usize,

    /// The error message.
    pub reason: &'static str,
}

#[test]
fn event_ast_eq() {
    for input_size in [10, 100, 1_000, 2_000, 5_000, 10_000] {
        let input = gen_input(input_size);
        let event_ast = event_to_tree(&mut parse_events(&input), &input).unwrap();
        let ast = simple_parser::parse(&input).unwrap();
        assert_eq!(event_ast, ast);
    }
}

#[test]
fn lexgen_ast_eq() {
    for input_size in [10, 100, 1_000, 2_000, 5_000, 10_000] {
        let input = gen_input(input_size);

        let mut lexgen_events: Vec<event::ParseEvent> = vec![];
        for event in parse_events_lexgen(&input) {
            lexgen_events.push(event.unwrap());
        }

        let mut event_parser_events: Vec<event::ParseEvent> = vec![];
        for event in parse_events(&input) {
            event_parser_events.push(event.unwrap());
        }

        assert_eq!(lexgen_events, event_parser_events);
    }
}
