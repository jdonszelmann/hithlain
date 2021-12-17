use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;

use miette::{NamedSource, SourceSpan, Diagnostic};
use thiserror::Error;

use crate::sim::instantiate::UniqueVariableRef;
use crate::sim::linked_ast::{BinaryBuiltin, Condition, Process, Statement};
use crate::sim::signal::Signal;
use crate::sim::SimulationError;
use crate::sim::value::Value;
use crate::time::Instant;

#[derive(Error, Debug, Diagnostic)]
#[error("assertion failed")]
#[diagnostic()]
pub struct AssertionError {
    #[source_code]
    src: NamedSource,

    #[label("here")]
    span: SourceSpan,
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

                let _res = $block;
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
