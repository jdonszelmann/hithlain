



use miette::{Diagnostic};
use thiserror::Error;

use simulation::{AssertionError, Simulation, SimulationState};

use crate::parse::desugared_ast::{Program, Process};

use crate::sim::value::ValueError;
use crate::sim::config::SimulationConfig;
use crate::vcd::{VcdError, VcdGenerator};
use std::rc::Rc;
use crate::sim::instantiate::instantiate_program;
use crate::vcd::vcd_ast::process_to_vcd_ast;
use crate::sim::link::link_process;


pub mod link;
pub mod linked_ast;
pub mod instantiate;
pub mod instantiated_ast;
pub mod value;
pub mod signal;
pub mod simulation;
pub mod config;

#[derive(Debug, Error, Diagnostic)]
pub enum SimulationError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    AssertionError(#[from] AssertionError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ValueError(#[from] ValueError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    VcdError(#[from] VcdError)
}

pub struct Simulator {
    program: Program,

    config: SimulationConfig,
}

impl Simulator {
    pub fn new(program: Program, config: SimulationConfig) -> Result<Self, SimulationError> {

        Ok(Self {
            program,
            config
        })
    }

    pub fn run_test(&self, name: impl AsRef<str>) -> Result<(), SimulationError> {
        for i in &self.program.tests {
            if i.name.0 == name.as_ref() {
                self._run_test(i.clone())?;
            }
        }
        Ok(())
    }

    fn _run_test(&self, test: Rc<Process>) -> Result<(), SimulationError> {
        self.execute_process(test)
    }

    pub fn run_all_tests(&self) -> Result<(), SimulationError> {
        for i in &self.program.tests {
            self._run_test(i.clone())?;
        }

        Ok(())
    }

    fn execute_process(&self, test: Rc<Process>) -> Result<(), SimulationError> {
        let instantiated = instantiate_program(test);

        let vcd_ast = if self.config.create_vcd {
            Some(process_to_vcd_ast(&instantiated))
        } else {
            None
        };

        let linked = link_process(instantiated);

        let mut simulation = Simulation::new(linked, &self.config, vcd_ast)?;
        while let SimulationState::Continue = simulation.step()? {}

        Ok(())
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
    use crate::sim::config::{SimulationConfig, VcdPath};

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

                assert x == 1;

            after 5ns:
                a = 1;
                b = 0;

                assert x == 0;
        }
        ";

        let lexed = lex(Source::test(src)).nice_unwrap();
        let mut parser = Parser::new(lexed);

        let parsed = parser.parse_program().nice_unwrap();

        let desugared = desugar_program(parsed).nice_unwrap();

        let s = Simulator::new(desugared, SimulationConfig::default()).nice_unwrap();
        s.run_all_tests().nice_unwrap();
    }

    #[test]
    fn test_add() {
        let src = "
        circuit add: a b c-in -> o c-out {
            o = a xor b xor c-in;
            c-out = (a and b) or ((a xor b) and c-in);
        }

        test main {
            o, c-out = add(a, b, 0);

            at 0ns:
                a = 1;
                b = 1;

                assert o == 0;
                assert c-out == 1;

            after 5ns:
                a = 0;
                b = 0;

                assert o == 0;
                assert c-out == 0;

            after 5ns:
                a = 1;
                b = 0;

                assert o == 1;
                assert c-out == 0;
        }
        ";

        let lexed = lex(Source::test(src)).nice_unwrap();
        let mut parser = Parser::new(lexed);

        let parsed = parser.parse_program().nice_unwrap();

        let desugared = desugar_program(parsed).nice_unwrap();

        let mut config = SimulationConfig::default();
        config.vcd_path = VcdPath::Path("test.vcd".into());
        let s = Simulator::new(desugared, config).nice_unwrap();
        s.run_all_tests().nice_unwrap();
    }
}
