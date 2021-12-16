use crate::parse::ast as a;
use crate::parse::desugared_ast as d;
use std::collections::HashMap;
use crate::parse::desugared_ast::{Program, Statement, BinaryBuiltin, TimedBlock};
use std::rc::Rc;
use thiserror::Error;
use miette::{Diagnostic, NamedSource, SourceSpan};
use crate::parse::scope::{Scope, VariableType, VariableRef, DuplicateDefinition};
use crate::parse::ast::{Variable, Expr, BinaryAction, Atom, StatementOrTime, NaryAction, UnaryAction};
use std::sync::atomic::Ordering;
use crate::error::Warn;
use crate::time::Instant;

#[derive(Error, Debug, Diagnostic)]
pub enum DesugarError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    VariableNeverWritten(#[from] VariableNeverWritten),

    #[error(transparent)]
    #[diagnostic(transparent)]
    DuplicateDefinition(#[from] DuplicateDefinition),

    #[error(transparent)]
    #[diagnostic(transparent)]
    TooManyVariablesOnLHS(#[from] TooManyVariablesOnLHS),

    #[error(transparent)]
    #[diagnostic(transparent)]
    NotEnoughVariablesOnLHS(#[from] NotEnoughVariablesOnLHS),

    #[error(transparent)]
    #[diagnostic(transparent)]
    CircuitDoesntExist(#[from] CircuitDoesntExist),
}

#[derive(Error, Debug, Diagnostic)]
#[error("use of variable that is never set")]
#[diagnostic(help("set the value variable {}", variable.0))]
pub struct VariableNeverWritten {
    #[source_code]
    src: NamedSource,

    variable: Variable,

    #[label("here")]
    span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("too many variables on left hand side of assignment")]
#[diagnostic()]
pub struct TooManyVariablesOnLHS {
    #[source_code]
    src: NamedSource,

    #[label("here")]
    span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("not enough variables on left hand side of assignment")]
#[diagnostic()]
pub struct NotEnoughVariablesOnLHS {
    #[source_code]
    src: NamedSource,

    #[label("here")]
    span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("circuit with name {} does not exist", variable.0)]
pub struct CircuitDoesntExist {
    #[source_code]
    src: NamedSource,

    variable: Variable,

    #[label("circuit name used here")]
    span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("output variable unassigned in circuit {}", variable.0)]
#[diagnostic(severity = "warning")]
pub struct UnassignedOutput {
    #[source_code]
    src: NamedSource,

    variable: Variable,

    #[label("here")]
    span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("unused variable {}", variable.0)]
#[diagnostic(severity = "warning")]
pub struct UnusedVariable {
    #[source_code]
    src: NamedSource,

    variable: Variable,

    #[label("first created here")]
    span: SourceSpan,
}



pub fn desugar_program(p: a::Program) -> Result<d::Program, DesugarError> {
    let mut circuits = HashMap::new();
    for i in p.circuits.iter() {
        circuits.insert(&i.name, None);
    }

    let mut desugared_circuits = Vec::new();

    let mut delayed = HashMap::new();
    for c in p.circuits.iter() {
        match desugar_circuit(c, &mut circuits) {
            Ok(i) => {
                let i = i?;

                circuits.insert(&c.name, Some(i.clone()));

                desugared_circuits.push(i)
            },
            Err(needed) => {
                delayed.entry(c).and_modify(|i: &mut Vec<Variable>| {
                    i.extend_from_slice(&needed);
                }).or_insert(needed);
            },
        }
    }

    let mut tests = Vec::new();
    for i in &p.tests {
        tests.push(desugar_test(i, &mut circuits)?);
    }

    Ok(Program {
        circuits: desugared_circuits,
        tests,
    })
}

fn desugar_circuit(circuit: &a::Circuit, circuit_names: &mut HashMap<&a::Variable, Option<Rc<d::Circuit>>>) -> Result<Result<Rc<d::Circuit>, DesugarError>, Vec<Variable>> {
    let mut scope = Scope::new();

    let mut inputs = Vec::new();
    for i in &circuit.inputs {
        inputs.push(match scope.define_variable(i, VariableType::In) {
            Ok(i) => i,
            Err(e) => return Ok(Err(e.into())),
        });
    }

    let mut outputs = Vec::new();
    for i in &circuit.outputs {
        outputs.push(match scope.define_variable(i, VariableType::Out) {
            Ok(i) => i,
            Err(e) => return Ok(Err(e.into())),
        });
    }

    let mut body = Vec::new();
    for i in &circuit.body {
        match desugar_statement(i, circuit_names, &mut scope) {
            Ok(Ok(i)) => body.extend(i),
            Err(needed) => return Err(needed),
            Ok(Err(e)) => return Ok(Err(e)),
        }
    }

    for i in scope.variables.values() {
        if i.0.variable_type == VariableType::Out && !i.0.written.load(Ordering::SeqCst) {
            if let Some(ref span) = i.0.variable.1 {
                UnassignedOutput {
                    src: span.source().clone().into(),
                    variable: i.0.variable.clone(),
                    span: span.clone().into()
                }.warn()
            } else {
                unreachable!("out variable must have source reference")
            }
        }

        if (i.0.variable_type == VariableType::In || i.0.variable_type == VariableType::Intermediate) &&
            !i.0.read.load(Ordering::SeqCst) {
            if let Some(ref span) = i.0.variable.1 {
                UnusedVariable {
                    src: span.source().clone().into(),
                    variable: i.0.variable.clone(),
                    span: span.clone().into()
                }.warn()
            } else {
                unreachable!("out variable must have source reference")
            }
        }

        if i.0.variable_type == VariableType::Intermediate && !i.0.written.load(Ordering::SeqCst) {
            if let Some(ref span) = i.0.variable.1 {
                return Ok(Err(VariableNeverWritten {
                    src: span.source().clone().into(),
                    variable: i.0.variable.clone(),
                    span: span.clone().into()
                }.into()))
            } else {
                unreachable!("out variable must have source reference")
            }
        }
    }

    Ok(Ok(Rc::new(d::Circuit {
        inputs,
        outputs,
        body
    })))
}

fn desugar_test(test: &a::Test, circuit_names: &mut HashMap<&a::Variable, Option<Rc<d::Circuit>>>) -> Result<d::Process, DesugarError> {
    let mut scope = Scope::new();

    let mut timed_blocks = Vec::new();
    desugar_timed_blocks(&test.body, &mut timed_blocks, circuit_names, &mut scope)?;

    for i in scope.variables.values() {
        if i.0.variable_type == VariableType::Intermediate && !i.0.read.load(Ordering::SeqCst) {
            if let Some(ref span) = i.0.variable.1 {
                UnusedVariable {
                    src: span.source().clone().into(),
                    variable: i.0.variable.clone(),
                    span: span.clone().into()
                }.warn()
            } else {
                unreachable!("out variable must have source reference")
            }
        }

        if i.0.variable_type == VariableType::Intermediate && !i.0.written.load(Ordering::SeqCst) {
            if let Some(ref span) = i.0.variable.1 {
                return Err(VariableNeverWritten {
                    src: span.source().clone().into(),
                    variable: i.0.variable.clone(),
                    span: span.clone().into()
                }.into())
            } else {
                unreachable!("out variable must have source reference")
            }
        }
    }

    Ok(d::Process {
        name: test.name.clone(),
        timed_blocks,

        inputs: vec![],
        outputs: vec![]
    })
}

fn desugar_timed_blocks(statements: &Vec<a::StatementOrTime>, blocks: &mut Vec<d::TimedBlock>, circuit_names: &mut HashMap<&Variable, Option<Rc<d::Circuit>>>, scope: &mut Scope) -> Result<(), DesugarError> {
    let mut statement_iter = statements.iter();

    let mut current_block = TimedBlock {
        time: Instant::START,
        block: vec![]
    };


    loop {
        match statement_iter.next() {
            Some(StatementOrTime::Time(a::TimeSpec::After(d))) => {
                let current_time = current_block.time;
                blocks.push(current_block);
                current_block = TimedBlock {
                    time: current_time.after(d),
                    block: vec![]
                }
            },
            Some(StatementOrTime::Time(a::TimeSpec::At(d))) => {
                blocks.push(current_block);
                current_block = TimedBlock {
                    time: *d,
                    block: vec![]
                }
            },
            Some(StatementOrTime::Statement(s)) => {
                let ds = match desugar_statement(s, circuit_names, scope) {
                    Ok(i) => i,
                    Err(e) => {
                        dbg!(e);
                        unreachable!("all circuits have been resolved when tests are desugared")
                    },
                }?;

                current_block.block.extend(ds);
            }
            None => {
                blocks.push(current_block);
                break;
            }
        }
    }

    Ok(())
}

fn desugar_statement(statement: &a::Statement, circuit_names: &mut HashMap<&a::Variable, Option<Rc<d::Circuit>>>, scope: &mut Scope) -> Result<Result<Vec<d::Statement>, DesugarError>, Vec<Variable>> {
    let mut res = Vec::new();

    match statement {
        a::Statement::Assignment(a) => {
            let mut res_vars = Vec::new();
            for i in &a.into {
                let var = match scope.lookup_variable_write(i) {
                    Ok(i) => i,
                    Err(e) => return Ok(Err(e.into())),
                };

                res_vars.push(var);
            }

            match desugar_expr(&a.expr, res_vars, &mut res, circuit_names, scope) {
                Ok(Ok(_)) => Ok(Ok(res)),
                Err(needed) => Err(needed),
                Ok(Err(e)) => Ok(Err(e))
            }
        }
        a::Statement::Assert { expr, span } => {
            let a_var = match scope.define_temp_variable() {
                Ok(i) => i,
                Err(e) => return Ok(Err(e.into())),
            };

            match desugar_expr(expr, vec![a_var.clone()], &mut res, circuit_names, scope) {
                Ok(Ok(_)) => (),
                Err(needed) => return Err(needed),
                Ok(Err(e)) => return Ok(Err(e))
            }

            res.push(Statement::Assert(a_var, span.clone()));

            Ok(Ok(res))
        }
    }

}

fn desugar_expr(expr: &a::Expr, into: Vec<VariableRef>, res: &mut Vec<d::Statement>, circuit_names: &mut HashMap<&a::Variable, Option<Rc<d::Circuit>>>, scope: &mut Scope) -> Result<Result<(), DesugarError>, Vec<Variable>> {
    macro_rules! cleanup {
        ($($tt: tt)*) => {
            match $($tt)* {
                Ok(Ok(i)) => i,
                Err(needed) => return Err(needed),
                Ok(Err(e)) => return Ok(Err(e)),
            }
        };
    }

    macro_rules! get_first {
        ($($tt: tt)*) => {
            {
                let a = $($tt)*;
                if a.len() > 1 {
                    let mut spans = Vec::new();
                    for i in &a {
                        if let Some(ref i) = i.0.variable.1 {
                            spans.push(i.clone());
                        }
                    }

                    let span = $crate::parse::span::Span::merge(&spans);
                    return Ok(Err(TooManyVariablesOnLHS {
                        src: span.source().clone().into(),
                        span: span.into(),
                    }.into()));
                }

                a[0].clone()
            }
        };
    }

     match expr {
         Expr::BinaryOp { a, b, action } => {
             let a_var = match scope.define_temp_variable() {
                 Ok(i) => i,
                 Err(e) => return Ok(Err(e.into())),
             };
             let b_var = match scope.define_temp_variable() {
                 Ok(i) => i,
                 Err(e) => return Ok(Err(e.into())),
             };

             cleanup!(desugar_expr(a, vec![a_var.clone()], res, circuit_names, scope));
             cleanup!(desugar_expr(b, vec![b_var.clone()], res, circuit_names, scope));

             match action {
                 BinaryAction::And => {
                     res.push(Statement::And(BinaryBuiltin {
                         a: a_var,
                         b: b_var,
                         into: get_first!(into),
                     }))
                 }
                 BinaryAction::Or => {
                     res.push(Statement::Or(BinaryBuiltin {
                         a: a_var,
                         b: b_var,
                         into: get_first!(into),
                     }))
                 }
                 BinaryAction::Nand => {
                     res.push(Statement::Nand(BinaryBuiltin {
                         a: a_var,
                         b: b_var,
                         into: get_first!(into),
                     }))
                 }
                 BinaryAction::Nor => {
                     res.push(Statement::Nor(BinaryBuiltin {
                         a: a_var,
                         b: b_var,
                         into: get_first!(into),
                     }))
                 }
                 BinaryAction::Xor => {
                     res.push(Statement::Xor(BinaryBuiltin {
                         a: a_var,
                         b: b_var,
                         into: get_first!(into),
                     }))
                 }
                 BinaryAction::Xnor => {
                     res.push(Statement::Xnor(BinaryBuiltin {
                         a: a_var,
                         b: b_var,
                         into: get_first!(into),
                     }))
                 }
                 BinaryAction::Custom(_) => unimplemented!("TODO") // TODO
             }
         }
         Expr::NaryOp { params, action } => {
             let mut param_vars = Vec::new();
             for i in params {
                 let var = match scope.define_temp_variable() {
                     Ok(i) => i,
                     Err(e) => return Ok(Err(e.into())),
                 };

                 cleanup!(desugar_expr(i, vec![var.clone()], res, circuit_names, scope));

                 param_vars.push(var);
             }

             match action {
                 NaryAction::UnaryAction(action) => {
                     match action {
                         UnaryAction::Not => {
                             res.push(Statement::Not {
                                 input: get_first!(param_vars),
                                 into: get_first!(into),
                             })
                         }
                     }
                 },
                 NaryAction::BinaryAction(_) => unimplemented!(),
                 NaryAction::Custom(c) => {
                     if let Some(circuit_exists) = circuit_names.get(c) {
                         if let Some(circuit) = circuit_exists {
                             res.push(Statement::Custom {
                                 inputs: param_vars,
                                 circuit: circuit.clone(),
                                 into,
                             });
                         } else {
                            return Err(vec![c.clone()])
                         }
                     } else {
                         if let Some(ref i) = c.1 {
                             return Ok(Err(CircuitDoesntExist {
                                 src: i.source().clone().into(),
                                 variable: c.clone(),
                                 span: i.clone().into()
                             }.into()))
                         } else {
                             unreachable!("circuit name must have a source location")
                         }
                     }
                 }
             }
         }
         Expr::Atom(a) => {
             match a {
                 Atom::Variable(v) => {
                     res.push(Statement::Move(get_first!(into), match scope.lookup_variable_read(v) {
                         Ok(i) => i,
                         Err(e) => return Ok(Err(e.into())),
                     }));
                 }
                 Atom::Constant(v) => {
                     res.push(Statement::Set(get_first!(into), v.clone()));
                 }
                 Atom::Expr(e) => {
                     return desugar_expr(e, into, res, circuit_names, scope);
                 }
             }
         }
     }

    Ok(Ok(()))
}

#[cfg(test)]
mod tests {
    use crate::parse::lexer::lex;
    use crate::error::NiceUnwrap;
    use crate::parse::source::Source;
    use crate::parse::parser::Parser;
    use crate::parse::desugar::desugar_program;

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

        desugar_program(parsed).nice_unwrap();
    }
}