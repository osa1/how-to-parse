use parsing_post as lib;

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let contents = std::fs::read_to_string(file).unwrap();
    let mut event_parser =
        lib::parse_events_iter_using_lexer_iter(lib::tokenize_iter(&contents), contents.len());
    lib::event_to_tree(&mut event_parser, &contents).unwrap();
}
