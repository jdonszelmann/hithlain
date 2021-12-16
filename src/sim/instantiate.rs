use crate::parse::desugared_ast as d;
use crate::sim::instantiated_ast as inst;
use std::collections::HashMap;
use std::rc::Rc;
use crate::parse::scope::VariableRef;
use crate::parse::ast::Variable;
use derivative::Derivative;
use std::fmt::{Formatter, Debug};
use crate::parse::desugared_ast::Statement;

#[derive(Clone, Derivative)]
#[derivative(Hash, PartialEq)]
pub struct UniqueVariableRef {
    identifier: usize,

    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    original: Rc<Variable>,
}

impl Debug for UniqueVariableRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.original.0, self.identifier)
    }
}

impl Eq for UniqueVariableRef {}

pub struct UniqueVariableRefGenerator {
    cur: usize,
}

impl UniqueVariableRefGenerator {
    pub fn new() -> Self {
        Self {
            cur: 0
        }
    }

    pub fn new_var(&mut self, variable: Variable) -> UniqueVariableRef {
        let res = UniqueVariableRef {
            identifier: self.cur,
            original: Rc::new(variable),
        };
        self.cur += 1;

        res
    }
}

pub fn rename(a: VariableRef, mapping: &mut HashMap<VariableRef, UniqueVariableRef>, gen: &mut UniqueVariableRefGenerator) -> UniqueVariableRef {
    if let Some(i) = mapping.get(&a) {
        i.clone()
    } else {
        let v = gen.new_var(a.0.variable.clone());
        mapping.insert(a.clone(), v.clone());
        v
    }
}

pub fn instantiate_program(p: d::Program) -> inst::Program {
    let mut gen = UniqueVariableRefGenerator::new();

    inst::Program {
        tests: p.tests.into_iter().map(|t| instantiate_process(t, &mut gen)).collect(),
    }
}

pub fn instantiate_process(c: d::Process, gen: &mut UniqueVariableRefGenerator) -> inst::Process {
    let mut mapping = HashMap::new();

    let inputs = c.inputs.into_iter()
        .map(|i| rename(i, &mut mapping, gen))
        .collect();

    let outputs = c.outputs.into_iter()
        .map(|i| rename(i, &mut mapping, gen))
        .collect();


    inst::Process {
        name: c.name.clone(),
        timed_blocks: c.timed_blocks.iter()
            .map(|i| instantiate_timed_block(i, &mut mapping, gen))
            .collect(),
        inputs,
        outputs,
    }
}

pub fn instantiate_timed_block(block: &d::TimedBlock, mapping: &mut HashMap<VariableRef, UniqueVariableRef>, gen: &mut UniqueVariableRefGenerator) -> inst::TimedBlock {
    inst::TimedBlock {
        time: block.time,
        block: block.block.iter()
            .map(|i| instantiate_statement(i.clone(), mapping, gen))
            .flatten()
            .collect()
    }
}

pub fn instantiate_circuit(c: Rc<d::Circuit>, gen: &mut UniqueVariableRefGenerator) -> inst::Circuit {
    let mut mapping = HashMap::new();

    let inputs = c.inputs.iter()
        .cloned()
        .map(|i| rename(i, &mut mapping, gen))
        .collect();

    let outputs = c.outputs.iter()
        .cloned()
        .map(|i| rename(i, &mut mapping, gen))
        .collect();

    inst::Circuit {
        inputs,
        outputs,
        body: c.body.iter()
            .map(|s| instantiate_statement(s.clone(), &mut mapping, gen))
            .flatten()
            .collect(),
    }
}

pub fn instantiate_statement(stmt: d::Statement, mapping: &mut HashMap<VariableRef, UniqueVariableRef>, gen: &mut UniqueVariableRefGenerator) -> Vec<inst::Statement> {
    macro_rules! rename_builtin {
        ($($tt:tt)*) => {
            {
                let d::BinaryBuiltin {
                    a,
                    b,
                    into,
                } = $($tt)*;

                inst::BinaryBuiltin {
                    a: rename(a, mapping, gen),
                    b: rename(b, mapping, gen),
                    into: rename(into, mapping, gen),
                }
            }
        };
    }

    match stmt {
        d::Statement::Not { input, into } => vec![inst::Statement::Not {
            input: rename(input, mapping, gen),
            into: rename(into, mapping, gen),
        }],
        d::Statement::And(i) => vec![inst::Statement::And(rename_builtin!(i))],
        d::Statement::Or(i) => vec![inst::Statement::Or(rename_builtin!(i))],
        d::Statement::Nand(i) => vec![inst::Statement::Nand(rename_builtin!(i))],
        d::Statement::Nor(i) => vec![inst::Statement::Nor(rename_builtin!(i))],
        d::Statement::Xor(i) => vec![inst::Statement::Xor(rename_builtin!(i))],
        d::Statement::Xnor(i) => vec![inst::Statement::Xnor(rename_builtin!(i))],
        d::Statement::Custom { inputs, circuit, into } => {
            let instantiated_circuit = instantiate_circuit(circuit, gen);

            let mut res = Vec::new();

            for (a, b) in inputs.iter().zip(&instantiated_circuit.inputs) {
                res.push(inst::Statement::Move(b.clone(), rename(a.clone(), mapping, gen)));
            }

            for (a, b) in into.iter().zip(&instantiated_circuit.outputs) {
                res.push(inst::Statement::Move(rename(a.clone(), mapping, gen), b.clone()));
            }

            res.push(inst::Statement::CreateInstance(instantiated_circuit));

            res
        }
        d::Statement::Move(a, b) => {
            vec![inst::Statement::Move(rename(a.clone(), mapping, gen), rename(b.clone(), mapping, gen))]
        }
        d::Statement::Set(a, b) => {
            vec![inst::Statement::Set(rename(a.clone(), mapping, gen), b)]
        }
        Statement::Assert(a, span) => {
            vec![inst::Statement::Assert(rename(a.clone(), mapping, gen), span)]
        }
    }
}

