fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("USAGE: test_gen <output size in bytes>");
        eprintln!("NOTE: Prints output to stdout");
        std::process::exit(1);
    }

    let size_str = &args[1];
    let size = size_str
        .parse::<usize>()
        .unwrap_or_else(|err| panic!("Unable to parse output size {:?}: {}", size_str, err));

    let output = parsing_post::gen_input(size);

    print!("{}", output);
}
