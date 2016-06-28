extern crate gdbwire;

use gdbwire::*;

fn main() {
    let parser = Parser::new(|out: Vec<Output>| {
        assert!(out.len() == 1);
        if out[0].kind == OutputKind::ParseError {
            println!("\n  Parse Error {}", out[0].line);
        }
        assert!(out[0].kind != OutputKind::ParseError);
    });
    main_loop(&parser);
}

fn main_loop(parser: &Parser) {
    let mut line = String::new();
    loop {
        if let Ok(n) = std::io::stdin().read_line(&mut line) {
            if n == 0 {
                break;
            }
            let result = parser.push(&line);
            assert!(result == Result::Ok);
        } else {
            break;
        }
        line.clear();
    }
}
