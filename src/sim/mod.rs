use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;

use derive_more::From;
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use value::Value;

use crate::parse::ast::Constant;
use crate::parse::desugared_ast;
use crate::sim::instantiate::{instantiate_program, UniqueVariableRef};
use crate::sim::link::link_program;
use crate::sim::linked_ast::{BinaryBuiltin, Condition, Process, Program, Statement};
use crate::sim::signal::Signal;
use crate::time::Instant;
use crate::sim::value::ValueError;

pub mod link;
pub mod linked_ast;
pub mod instantiate;
pub mod instantiated_ast;
pub mod value;
pub mod signal;

#[derive(Debug, Error, Diagnostic)]
pub enum SimulationError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    AssertionError(#[from] AssertionError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ValueError(#[from] ValueError),
}

#[derive(Error, Debug, Diagnostic)]
#[error("assertion failed")]
#[diagnostic()]
pub struct AssertionError {
    #[source_code]
    src: NamedSource,

    #[label("here")]
    span: SourceSpan,
}


pub struct Simulator {
    program: Program,
}

impl Simulator {
    pub fn new(program: desugared_ast::Program) -> Self {
        let program = link_program(instantiate_program(program));

        Self {
            program,
        }
    }

    pub fn run_test(&self, name: impl AsRef<str>) -> Result<(), SimulationError> {
        for i in &self.program.tests {
            if i.name.0 == name.as_ref() {
                self._run_test(i)?;
            }
        }
        Ok(())
    }

    fn _run_test(&self, test: &Process) -> Result<(), SimulationError> {
        self.execute_process(test)
    }

    pub fn run_all_tests(&self) -> Result<(), SimulationError> {
        for i in &self.program.tests {
            self._run_test(i)?;
        }

        Ok(())
    }

    fn execute_process(&self, test: &Process) -> Result<(), SimulationError> {
        let mut simulation = Simulation::new(test);
        while let SimulationState::Continue = simulation.step()? {}
        Ok(())
    }
}

pub struct Simulation<'a> {
    pq: BinaryHeap<Reverse<Signal>>,
    map: HashMap<&'a UniqueVariableRef, Vec<Rc<Statement>>>,
    store: HashMap<UniqueVariableRef, Value>
}

impl<'a> Simulation<'a> {
    pub fn new(process: &'a Process) -> Self {
        let mut map = HashMap::new();
        let mut pq = BinaryHeap::new();

        for i in process.conditions.iter() {
            match i {
                Condition::AtTime { time, run } => {
                    // println!("{:?} --> {:?}", time, run);
                    pq.push(Reverse(Signal {
                        time: time.clone(),
                        action: run.clone()
                    }))
                }
                Condition::WhenChanges { variable, run } => {
                    // println!("{:?} --> {:?}", variable, run);
                    map.entry(variable)
                        .and_modify(|i: &mut Vec<Rc<Statement>>| i.push(run.clone()))
                        .or_insert(vec![run.clone()]);
                }
            }
        }

        Self {
            pq,
            map,
            store: Default::default()
        }
    }

    fn get_var(&self, var: &UniqueVariableRef) -> Option<Value> {
        if let Some(i) = self.store.get(var) {
            Some(i.clone())
        } else {
            // eprintln!("not set: {:?}, {:?}", name, store);
            return None
        }
    }

    fn store_var(&mut self, var: &UniqueVariableRef, value: impl Into<Value>) {
        self.store.insert(var.clone(), value.into());
    }

    fn handle_signal(&mut self, action: &Statement, time: Instant) -> Result<(), SimulationError> {
        let mut modified_variables = Vec::new();

        macro_rules! update {
            ($($variable: ident),* -> $result: ident $block: block) => {
                {
                    $(
                        let $variable = if let Some(i) = self.get_var($variable) {
                            i
                        } else {
                            return Ok(())
                        };
                    )*

                    let res = $block?;

                    modified_variables.push($result);

                    self.store_var($result, res)
                }
            };

            ($($variable: ident),* $block: block) => {
                $(
                    let $variable = if let Some(i) = self.get_var($variable) {
                        i
                    } else {
                        return Ok(())
                    };
                )*

                let res = $block;
            };
        }

        match action {
            Statement::Not { input, into } => {
                update!(input -> into {
                    !input
                })
            }
            Statement::And(BinaryBuiltin{ a, b, into }) => {
                update!(a, b -> into {
                    a & b
                });
            }
            Statement::Or(BinaryBuiltin{ a, b, into }) => {
                update!(a, b -> into {
                    a | b
                });
            }
            Statement::Nand(BinaryBuiltin{ a, b, into }) => {
                update!(a, b -> into {
                    !(a & b)?
                });
            }
            Statement::Nor(BinaryBuiltin{ a, b, into }) => {
                update!(a, b -> into {
                    !(a | b)?
                });
            }
            Statement::Xor(BinaryBuiltin{ a, b, into }) => {
                update!(a, b -> into {
                    a ^ b
                });
            }
            Statement::Xnor(BinaryBuiltin{ a, b, into }) => {
                update!(a, b -> into {
                    !(a ^ b)?
                });
            }
            Statement::Move(a, b) => {
                update!(a -> b { Result::<_, SimulationError>::Ok(a) });
            }
            Statement::Set(a, b) => {
                update!( -> a { Result::<_, SimulationError>::Ok(b) });
            }
            Statement::Assert(v, span) => {
                update!(v {
                    if let Value::Bit(true) = v {} else {
                        return Err(AssertionError {
                            src: span.source().clone().into(),
                            span: span.clone().into()
                        }.into())
                    }
                });
            }
        }

        // eprintln!("{:?}, {:?} --> {}", time, action, res);

        for i in modified_variables {
            for statement in self.map.get(i).unwrap_or(&Vec::new()) {
                self.pq.push(Reverse(Signal {
                    time: time.add_delta(),
                    action: statement.clone(),
                }))
            }
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<SimulationState, SimulationError> {
        if let Some(Reverse(Signal{ time, action })) = self.pq.pop() {
            self.handle_signal(&action, time)?;

            Ok(SimulationState::Continue)
        } else {
            Ok(SimulationState::End)
        }
    }
}

pub enum SimulationState {
    Continue,
    End,
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

        let s = Simulator::new(desugared);
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

        let s = Simulator::new(desugared);
        s.run_all_tests().nice_unwrap();
    }
}
