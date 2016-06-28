extern crate gdbwire;

use gdbwire::*;

fn main() {
    let callback = ParserCallback::<()>::new(None, |ctx, out| {
        assert!(ctx.is_none());
        assert!(out.len() == 1);
        if out[0].kind == OutputKind::ParseError {
            println!("\n  Parse Error {}", out[0].line);
        }
        assert!(out[0].kind != OutputKind::ParseError);
    });
    let parser = Parser::new(callback);
    main_loop(&parser);
}

fn main_loop<T>(parser: &Parser<T>) {
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
