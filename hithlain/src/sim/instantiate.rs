use crate::parse::desugared_ast as d;
use crate::sim::instantiated_ast as inst;
use std::collections::HashMap;
use std::rc::Rc;
use crate::parse::scope::{VariableRef, VariableType};

use derivative::Derivative;
use std::fmt::{Formatter, Debug};
use crate::parse::desugared_ast::Statement;
use crate::sim::instantiated_ast::{LocalizedVariable, Package};
use std::ops::Deref;

#[derive(Clone, Derivative)]
#[derivative(Hash, PartialEq)]
pub struct UniqueVariableRef {
    pub(crate) identifier: usize,

    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    pub(crate) generated: bool,

    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    pub(crate) original: LocalizedVariable,
}

impl UniqueVariableRef {
    pub fn name(&self) -> String {
        let mut res = String::new();
        for i in self.original.path.deref() {
            res.push_str(&i.name().0);
            res.push_str(".");
        }

        res.push_str(&self.original.variable.0);

        res
    }
}

impl Debug for UniqueVariableRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in self.original.path.deref() {
            write!(f, "{}.", i.name().0)?
        }

        write!(f, "{}", self.original.variable.0)
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

    pub fn new_var(&mut self, variable: LocalizedVariable) -> UniqueVariableRef {
        let res = UniqueVariableRef {
            identifier: self.cur,
            generated: false,
            original: variable,
        };
        self.cur += 1;

        res
    }
}

pub fn rename(a: VariableRef, mapping: &mut HashMap<VariableRef, UniqueVariableRef>, gen: &mut UniqueVariableRefGenerator, package_path: &Rc<Vec<Package>>) -> UniqueVariableRef {
    if let Some(i) = mapping.get(&a) {
        i.clone()
    } else {
        let mut v = gen.new_var(a.0.variable.localize(package_path.clone()));
        if a.0.variable_type == VariableType::Temp {
            v.generated = true;
        }

        mapping.insert(a.clone(), v.clone());
        v
    }
}

pub fn instantiate_program(p: Rc<d::Process>) -> inst::Process {
    let mut gen = UniqueVariableRefGenerator::new();

    instantiate_process(p, &mut gen, vec![])
}

pub fn instantiate_process(c: Rc<d::Process>, gen: &mut UniqueVariableRefGenerator, mut package_path: Vec<Package>) -> inst::Process {
    package_path.push(c.clone().into());
    let local_package_path = Rc::new(package_path);

    let mut mapping = HashMap::new();

    let inputs = c.inputs.iter()
        .map(|i| rename(i.clone(), &mut mapping, gen, &local_package_path))
        .collect();

    let outputs = c.outputs.iter()
        .map(|i| rename(i.clone(), &mut mapping, gen, &local_package_path))
        .collect();


    inst::Process {
        name: c.name.clone(),
        timed_blocks: c.timed_blocks.iter()
            .map(|i| instantiate_timed_block(i, &mut mapping, gen, &local_package_path))
            .collect(),
        inputs,
        outputs,
    }
}

pub fn instantiate_timed_block(block: &d::TimedBlock, mapping: &mut HashMap<VariableRef, UniqueVariableRef>, gen: &mut UniqueVariableRefGenerator, package_path: &Rc<Vec<Package>>) -> inst::TimedBlock {
    inst::TimedBlock {
        time: block.time,
        block: block.block.iter()
            .map(|i| instantiate_statement(i.clone(), mapping, gen, &package_path))
            .flatten()
            .collect()
    }
}

pub fn instantiate_circuit(c: Rc<d::Circuit>, gen: &mut UniqueVariableRefGenerator, mut package_path: Vec<Package>) -> inst::Circuit {
    package_path.push(c.clone().into());
    let local_package_path = Rc::new(package_path);

    let mut mapping = HashMap::new();

    let inputs = c.inputs.iter()
        .cloned()
        .map(|i| rename(i, &mut mapping, gen, &local_package_path))
        .collect();

    let outputs = c.outputs.iter()
        .cloned()
        .map(|i| rename(i, &mut mapping, gen, &local_package_path))
        .collect();

    inst::Circuit {
        name: c.name.clone(),
        inputs,
        outputs,
        body: c.body.iter()
            .map(|s| instantiate_statement(s.clone(), &mut mapping, gen, &local_package_path))
            .flatten()
            .collect(),
    }
}

pub fn instantiate_statement(stmt: d::Statement, mapping: &mut HashMap<VariableRef, UniqueVariableRef>, gen: &mut UniqueVariableRefGenerator, package_path: &Rc<Vec<Package>>) -> Vec<inst::Statement> {
    macro_rules! rename_builtin {
        ($($tt:tt)*) => {
            {
                let d::BinaryBuiltin {
                    a,
                    b,
                    into,
                } = $($tt)*;

                inst::BinaryBuiltin {
                    a: rename(a, mapping, gen, package_path),
                    b: rename(b, mapping, gen, package_path),
                    into: rename(into, mapping, gen, package_path),
                }
            }
        };
    }

    match stmt {
        d::Statement::Not { input, into } => vec![inst::Statement::Not {
            input: rename(input, mapping, gen, &package_path),
            into: rename(into, mapping, gen, &package_path),
        }],
        d::Statement::And(i) => vec![inst::Statement::And(rename_builtin!(i))],
        d::Statement::Or(i) => vec![inst::Statement::Or(rename_builtin!(i))],
        d::Statement::Nand(i) => vec![inst::Statement::Nand(rename_builtin!(i))],
        d::Statement::Nor(i) => vec![inst::Statement::Nor(rename_builtin!(i))],
        d::Statement::Xor(i) => vec![inst::Statement::Xor(rename_builtin!(i))],
        d::Statement::Xnor(i) => vec![inst::Statement::Xnor(rename_builtin!(i))],
        d::Statement::Custom { inputs, circuit, into } => {
            let instantiated_circuit = instantiate_circuit(circuit, gen, package_path.deref().clone());

            let mut res = Vec::new();

            for (a, b) in inputs.iter().zip(&instantiated_circuit.inputs) {
                res.push(inst::Statement::Move(b.clone(), rename(a.clone(), mapping, gen, package_path)));
            }

            for (a, b) in into.iter().zip(&instantiated_circuit.outputs) {
                res.push(inst::Statement::Move(rename(a.clone(), mapping, gen, package_path), b.clone()));
            }

            res.push(inst::Statement::CreateCircuitInstance(instantiated_circuit));

            res
        }
        d::Statement::Move(a, b) => {
            vec![inst::Statement::Move(rename(a.clone(), mapping, gen, package_path), rename(b.clone(), mapping, gen, package_path))]
        }
        d::Statement::Set(a, b) => {
            vec![inst::Statement::Set(rename(a.clone(), mapping, gen, package_path), b)]
        }
        Statement::Assert(a, span) => {
            vec![inst::Statement::Assert(rename(a.clone(), mapping, gen, package_path), span)]
        }
    }
}

