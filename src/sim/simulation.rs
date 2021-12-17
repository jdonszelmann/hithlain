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
use crate::sim::config::SimulationConfig;
use crate::vcd::VcdGenerator;
use crate::vcd::vcd_ast::VcdModule;

#[derive(Error, Debug, Diagnostic)]
#[error("assertion failed")]
#[diagnostic()]
pub struct AssertionError {
    #[source_code]
    src: NamedSource,

    #[label("here")]
    span: SourceSpan,
}


pub struct Simulation<'config> {
    pq: BinaryHeap<Reverse<Signal>>,
    map: HashMap<UniqueVariableRef, Vec<Rc<Statement>>>,
    store: HashMap<UniqueVariableRef, Value>,

    vcd: Option<VcdGenerator>,

    config: &'config SimulationConfig,

    last_instant: Instant,
}


impl<'config> Simulation<'config> {
    pub fn new(process: Process, config: &'config SimulationConfig, vcd_ast: Option<VcdModule>) -> Result<Self, SimulationError> {
        let mut map = HashMap::new();
        let mut pq = BinaryHeap::new();

        let max_time = Instant::START;

        for i in process.conditions.into_iter() {
            match i {
                Condition::AtTime { time, run } => {
                    println!("{:?} --> {:?}", time, run);
                    pq.push(Reverse(Signal {
                        time: time.clone(),
                        action: run.clone()
                    }))
                }
                Condition::WhenChanges { variable, run } => {
                    println!("{:?} --> {:?}", variable, run);
                    map.entry(variable)
                        .and_modify(|i: &mut Vec<Rc<Statement>>| i.push(run.clone()))
                        .or_insert(vec![run.clone()]);
                }
            }
        }

        let vcd = vcd_ast.map(|ast| {
            VcdGenerator::new(&config.vcd_path, max_time.vcd_scale(), ast)
        }).transpose()?;

        Ok(Self {
            pq,
            map,
            store: Default::default(),
            vcd,
            config,
            last_instant: Instant::START,
        })
    }

    pub fn finalize(&mut self) -> Result<(), SimulationError> {
        if let Some(ref mut i) = self.vcd {
            i.finalize(self.last_instant, self.config.vcd_overshoot_duration)?;
        }

        Ok(())
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

    fn handle_signal<'action>(&mut self, action: &'action Statement) -> Result<Vec<&'action UniqueVariableRef>, SimulationError> {
        let mut modified_variables = Vec::new();

        macro_rules! update {
            ($($variable: ident),* -> $result: ident $block: block) => {
                {
                    $(
                        let $variable = if let Some(i) = self.get_var($variable) {
                            i
                        } else {
                            println!("didn't exist: {:?}", $variable);
                            return Ok(modified_variables)
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
                        println!("didn't exist: {:?}", $variable);
                        return Ok(modified_variables)
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
                update!(b -> a { Result::<_, SimulationError>::Ok(b) });
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

        Ok(modified_variables)
    }

    pub fn update_queue(&mut self, modified_variables: Vec<&UniqueVariableRef>, time: Instant) -> Result<(), SimulationError> {
        for i in modified_variables {
            // println!("{:?} modified variable {:?}", time, i);
            if let Some(value) = self.get_var(i) {
                if let Some(ref mut gen) = self.vcd {
                    gen.update_wire(i, value, time)?;
                }
            }

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
            println!("{:?}", action);
            let modified_variables = self.handle_signal(&action)?;
            self.update_queue(modified_variables, time)?;

            self.last_instant = time;

            Ok(SimulationState::Continue)
        } else {
            self.finalize()?;
            Ok(SimulationState::End)
        }
    }
}

pub enum SimulationState {
    Continue,
    End,
}
