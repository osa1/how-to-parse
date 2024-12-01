use parsing_post as lib;

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let contents = std::fs::read_to_string(file).unwrap();
    lib::parse_ast_non_recursive(&contents).unwrap();
}
