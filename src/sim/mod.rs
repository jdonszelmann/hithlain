use crate::sim::instantiate::instantiate_program;
use crate::sim::link::link_program;
use crate::parse::desugared_ast;
use std::collections::{BinaryHeap, HashMap};
use crate::sim::signal::Signal;
use crate::sim::linked_ast::{Program, Process, Statement, Condition, BinaryBuiltin};
use std::rc::Rc;
use crate::parse::ast::Constant;
use std::cmp::Reverse;

pub mod link;
pub mod linked_ast;
pub mod instantiate;
pub mod instantiated_ast;
pub mod signal;
use thiserror::Error;
use miette::{Diagnostic, NamedSource, SourceSpan};

#[derive(Debug, Error, Diagnostic)]
pub enum InterpError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    AssertionError(#[from] AssertionError)
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

    pub fn run_test(&self, name: impl AsRef<str>) -> Result<(), InterpError> {
        for i in &self.program.tests {
            if i.name.0 == name.as_ref() {
                self._run_test(i)?;
            }
        }
        Ok(())
    }

    fn _run_test(&self, test: &Process) -> Result<(), InterpError> {
        self.execute_process(test)
    }

    pub fn run_all_tests(&self) -> Result<(), InterpError> {
        for i in &self.program.tests {
            self._run_test(i)?;
        }

        Ok(())
    }

    fn execute_process(&self, test: &Process) -> Result<(), InterpError> {

        let mut map = HashMap::new();
        let mut pq = BinaryHeap::new();

        for i in test.conditions.iter() {
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


        let mut store: HashMap<_, bool> = HashMap::new();

        macro_rules! get_var {
            ($($tt: tt)*) => {
                {
                    let name = $($tt)*;
                    if let Some(&i) = store.get(name) {
                        i
                    } else {
                        // eprintln!("not set: {:?}, {:?}", name, store);
                        continue;
                    }
                }
            };
        }

        while let Some(Reverse(Signal{ time, action })) = pq.pop() {
            let mut res = false;

            // eprintln!("{:?}, {:?}", time, action);

            let mut modified_variables = Vec::new();
            match &*action {
                Statement::Not { input, into } => {
                    modified_variables.push(into);
                    res = !get_var!(input);
                    store.insert(into.clone(), res);
                }
                Statement::And(BinaryBuiltin{ a, b, into }) => {
                    modified_variables.push(into);
                    res = get_var!(a) && get_var!(b);
                    store.insert(into.clone(), res);
                }
                Statement::Or(BinaryBuiltin{ a, b, into }) => {
                    modified_variables.push(into);
                    res = get_var!(a) || get_var!(b);
                    store.insert(into.clone(), res);
                }
                Statement::Nand(BinaryBuiltin{ a, b, into }) => {
                    modified_variables.push(into);
                    res = !(get_var!(a) && get_var!(b));
                    store.insert(into.clone(), res);
                }
                Statement::Nor(BinaryBuiltin{ a, b, into }) => {
                    modified_variables.push(into);
                    res = !(get_var!(a) || get_var!(b));
                    store.insert(into.clone(), res);
                }
                Statement::Xor(BinaryBuiltin{ a, b, into }) => {
                    modified_variables.push(into);
                    res = get_var!(a) != get_var!(b);
                    store.insert(into.clone(), res);
                }
                Statement::Xnor(BinaryBuiltin{ a, b, into }) => {
                    modified_variables.push(into);
                    res = get_var!(a) == get_var!(b);
                    store.insert(into.clone(), res);
                }
                Statement::Move(a, b) => {
                    modified_variables.push(a);
                    res = get_var!(b);
                    store.insert(a.clone(), res);
                }
                Statement::Set(a, b) => {
                    modified_variables.push(a);
                    match b {
                        Constant::Number(n) => {
                            store.insert(a.clone(), *n > 0);
                        }
                    }
                }
                Statement::Assert(v, span) => {
                    res = get_var!(v);
                    if !res {
                        return Err(AssertionError {
                            src: span.source().clone().into(),
                            span: span.clone().into()
                        }.into())
                    }
                }
            }

            eprintln!("{:?}, {:?} --> {}", time, action, res);

            for i in modified_variables {
                for statement in map.get(i).unwrap_or(&Vec::new()) {
                    pq.push(Reverse(Signal {
                        time: time.add_delta(),
                        action: statement.clone(),
                    }))
                }
            }
        }

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

            after 1ns:
                assert x == 1;

            after 5ns:
                a = 1;
                b = 0;

            after 1ns:
                assert x == 1;
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
