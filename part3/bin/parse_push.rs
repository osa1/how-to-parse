use parsing_post as lib;

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let contents = std::fs::read_to_string(file).unwrap();
    let mut listener = lib::AstBuilderListener::new(&contents);
    lib::parse_events_push_using_lexer_push(&contents, &mut listener);
    let (ast, error) = listener.into_ast();
    if error.is_some() {
        panic!();
    }
    ast.unwrap();
}
