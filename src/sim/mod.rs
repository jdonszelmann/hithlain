use crate::sim::instantiate::instantiate_program;
use crate::sim::link::link_program;
use crate::parse::desugared_ast;
use std::collections::BinaryHeap;
use crate::sim::signal::Signal;
use crate::sim::linked_ast::{Program, Process};

pub mod link;
pub mod linked_ast;
pub mod instantiate;
pub mod instantiated_ast;
pub mod signal;



pub struct Simulator {
    program: Program,

    prioque: BinaryHeap<Signal>
}

impl Simulator {
    pub fn new(program: desugared_ast::Program) -> Self {
        let program = link_program(instantiate_program(program));



        Self {
            program,
            prioque: BinaryHeap::new()
        }
    }

    pub fn run_test(&self, name: impl AsRef<str>) {
        for i in &self.program.tests {
            if i.name.0 == name.as_ref() {
                self._run_test(i)
            }
        }
    }

    fn _run_test(&self, test: &Process) {
        self.execute_process(test)
    }

    pub fn run_all_tests(&self) {
        for i in &self.program.tests {
            self._run_test(i)
        }
    }

    fn execute_process(&self, test: &Process) {

    }
}


#[cfg(test)]
mod tests {
    use crate::error::NiceUnwrap;
    use crate::parse::desugar::desugar_program;
    use crate::parse::lexer::lex;
    use crate::parse::parser::Parser;
    use crate::parse::source::Source;
    use crate::sim::Simulator;

    #[test]
    fn test_smoke() {
        let src = "
        circuit something: a b c -> d e {
            d = a and b;
            e = b or c;
        }

        test main {
            x, y = something(a, b, 0);

            at 0ns:
                a = 1;
                b = 1;

                // assert x == 1

            after 5ns:
                a = 1;
                b = 0;

                // assert x == 0
        }
        ";

        let lexed = lex(Source::test(src)).nice_unwrap();
        let mut parser = Parser::new(lexed);

        let parsed = parser.parse_program().nice_unwrap();

        let desugared = desugar_program(parsed).nice_unwrap();

        let s = Simulator::new(desugared);
        s.run_all_tests();
    }
}
