// Copyright (c) 2015 gdbwire crate developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

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
