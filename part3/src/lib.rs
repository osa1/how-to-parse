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

/// Implements an event listener that builds simple AST.
mod ast_builder_listener;

/// Implements a recursive-descent AST parser.
mod recursive_descent;

/// Implements parsing directly to the AST, non-recursively.
mod direct_non_recursive;

/// Implements a parser that extracts timestamps from events, without building an AST.
mod timestamp_parser;

/// Implements generating an AST from an event parser.
mod event_to_tree;

/// Implements collecting parse events from a "push" event parser.
mod push_to_events;

/// Implements input generation for benchmarks.
mod input_gen;

/// Implements an iterator event parser using the iterator lexer.
mod event_parser_2;

/// Implements an event push parser using the push lexer.
mod event_push_parser_2;

/// Implements an event push parser using the iterator lexer.
mod event_push_parser_3;

/// Implements an event push parser using the iterator lexer, without recursion.
mod event_push_parser_3_non_recursive;

/// Implements an iterator lexer.
mod lexer;

/// Implements a push lexer.
mod lexer_push;

/// Implements a lexer that generates a vector of tokens.
mod lexer_list;

/// Lexer tokens.
mod token;

#[cfg(test)]
mod test_common;

pub use ast_builder_listener::AstBuilderListener;
pub use direct_non_recursive::parse as parse_ast_non_recursive;
pub use event::{ParseEvent, ParseEventKind};
pub use event_listener::EventListener;
pub use event_to_tree::event_to_tree;
pub use push_to_events::PushToEvents;
pub use recursive_descent::parse as parse_ast_recursive;
pub use simple_ast::Json;
pub use timestamp_parser::{parse_timestamp, TimestampParserListener};
pub use token::Token;

pub use lexer::{tokenize_iter, Lexer};
pub use lexer_list::tokenize_list;
pub use lexer_push::{tokenize_push, LexerEventListener, PushToTokens};

pub use event_parser::parse_events_iter;
pub use event_parser_2::parse_events_iter_using_lexer_iter;
pub use event_push_parser::parse as parse_events_push;
pub use event_push_parser_2::parse as parse_events_push_using_lexer_push;
pub use event_push_parser_3::parse as parse_events_push_using_lexer_iter;
pub use event_push_parser_3_non_recursive::parse as parse_events_push_using_lexer_iter_non_recursive;

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
