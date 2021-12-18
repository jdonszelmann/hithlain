use crate::sim::instantiate::UniqueVariableRef;
use crate::sim::instantiated_ast::{Process, TimedBlock, Statement, BinaryBuiltin, Circuit};
use crate::parse::ast::Variable;
use std::collections::HashSet;

pub struct VcdModule {
    pub(crate) name: Variable,
    pub(crate) variables: Vec<UniqueVariableRef>,
    pub(crate) submodules: Vec<VcdModule>,
}

pub fn process_to_vcd_ast(process: &Process) -> VcdModule {
    let mut submodules = Vec::new();
    let mut variables = HashSet::new();

    for i in &process.inputs {
        variables.insert(i.clone());
    }

    for i in &process.outputs {
        variables.insert(i.clone());
    }

    for i in &process.timed_blocks {
        analyze_timed_block(i, &mut variables, &mut submodules);
    }

    VcdModule {
        name: process.name.clone(),
        variables: variables.into_iter()
            .filter(|i| !i.generated)
            .filter(|i| i.original.path.len() == 1)
            .collect(),
        submodules,
    }
}

pub fn circuit_to_vcd_ast(circuit: &Circuit) -> VcdModule {
    let mut submodules = Vec::new();
    let mut variables = HashSet::new();

    for i in &circuit.inputs {
        variables.insert(i.clone());
    }

    for i in &circuit.outputs {
        variables.insert(i.clone());
    }

    for i in &circuit.body {
        analyze_statement(i, &mut variables, &mut submodules)
    }


    VcdModule {
        name: circuit.name.clone(),
        variables: variables.into_iter()
            .filter(|i| !i.generated)
            .filter(|i| if let Some(i) = i.original.path.last() {
                if i.name() == &circuit.name {
                    true
                } else {
                    false
                }
            } else {
                false
            })
            .collect(),
        submodules,
    }
}

fn analyze_timed_block(t: &TimedBlock, variables: &mut HashSet<UniqueVariableRef>, submodules: &mut Vec<VcdModule>){
    for i in &t.block {
        analyze_statement(i, variables, submodules)
    }
}

fn analyze_statement(s: &Statement, variables: &mut HashSet<UniqueVariableRef>, submodules: &mut Vec<VcdModule>) {
    match s {
        Statement::Assert(v, _) => {
            variables.insert(v.clone());
        },
        Statement::Not { input, into } => {
            variables.insert(input.clone());
            variables.insert(into.clone());
        },
        Statement::And(a)
        | Statement::Or(a)
        | Statement::Nand(a)
        | Statement::Nor(a)
        | Statement::Xor(a)
        | Statement::Xnor(a)
        => {
            let BinaryBuiltin{ a, b, into } = a;
            variables.insert(a.clone());
            variables.insert(b.clone());
            variables.insert(into.clone());
        }
        Statement::Move(a, b) => {
            variables.insert(a.clone());
            variables.insert(b.clone());
        }
        Statement::Set(a, _) => {
            variables.insert(a.clone());
        }
        Statement::CreateCircuitInstance(a) => {
            submodules.push(circuit_to_vcd_ast(a));
        }
    }
}