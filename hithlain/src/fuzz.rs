use bnf::{Grammar};
use crate::parse::lexer::lex;
use crate::parse::source::Source;
use crate::parse::parser::Parser;
use crate::error::NiceUnwrap;


const GRAMMAR: &str = include_str!("grammar.bnf");

fn generate_sentence(g: &Grammar) -> String {
    loop {
        let res = g.generate_callback(|ident, value| match ident {
            "number" => value.len() < 10,
            "name" => !["test", "circuit", "at", "every", "or", "and", "nor", "nand", "xor", "xnor", "not"].contains(&value),
            _ => true
        });
        match res {
            Ok(i) => break i,
            Err(bnf::Error::RecursionLimit(_)) => continue,
            _ => panic!("aaaaa"),
        }
    }
}


#[test]
fn test_fuzz() {
    let grammar: Grammar = match GRAMMAR.parse() {
        Ok(i) => i,
        Err(e) => {
            panic!("{}", e);
        }
    };

    for _ in 0..5000 {
        let sentence = generate_sentence(&grammar);

        let lexed = lex(Source::test(&sentence)).nice_unwrap_panic();
        let mut parser = Parser::new(lexed);

        if let Err(e) = parser.parse_program() {
            println!("failed on program: {}", sentence);

            Result::<(), _>::Err(e).nice_unwrap_panic();
        }
    }
}