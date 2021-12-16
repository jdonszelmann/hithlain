use crate::sim::instantiated_ast as inst;
use crate::sim::linked_ast as l;
use crate::sim::linked_ast::Condition;
use std::rc::Rc;
use crate::time::Instant;
use crate::sim::instantiated_ast::Statement;

pub fn link_program(p: inst::Program) -> l::Program {
    let mut conditions = Vec::new();
    for i in p.circuits {
        conditions.extend(link_circuit(i));
    }

    l::Program {
        conditions,
        tests: p.tests.into_iter().map(|i| link_process(i)).collect()
    }
}


pub fn link_statement_list(statements: Vec<Statement>) -> Vec<Condition> {
    macro_rules! binary_stmt {
        ($path: path, $($tt: tt)*) => {
            {
                let inst::BinaryBuiltin{a, b, into} = $($tt)*;
                let stmt = Rc::new($path(l::BinaryBuiltin{a, b, into}));

                vec![
                    Condition::WhenChanges {
                        variable: a,
                        run: stmt.clone()
                    },
                    Condition::WhenChanges {
                        variable: b,
                        run: stmt.clone()
                    }
                ]
            }
        };
    }

    statements.into_iter()
        .map(|i| match i {
            inst::Statement::Not { input, into } => {
                vec![Condition::WhenChanges { variable: input, run:  Rc::new(l::Statement::Not{input, into})}]
            }
            inst::Statement::And(b) => binary_stmt!(l::Statement::And, b),
            inst::Statement::Or(b) => binary_stmt!(l::Statement::Or, b),
            inst::Statement::Nand(b) => binary_stmt!(l::Statement::Nand, b),
            inst::Statement::Nor(b) => binary_stmt!(l::Statement::Nor, b),
            inst::Statement::Xor(b) => binary_stmt!(l::Statement::Xor, b),
            inst::Statement::Xnor(b) => binary_stmt!(l::Statement::Xnor, b),
            inst::Statement::Move(a, b) => {
                vec![Condition::WhenChanges { variable: b, run:  Rc::new(l::Statement::Move(a, b))}]
            },
            inst::Statement::Set(a, b) => {
                vec![Condition::AtTime { time: Instant::START, run:  Rc::new(l::Statement::Set(a, b))}]
            }
        })
        .flatten()
        .collect()
}

pub fn link_circuit(circuit: inst::Circuit) -> Vec<Condition> {
    link_statement_list(circuit.body)
        .into_iter()
        .chain(circuit.inner_circuits.into_iter().map(link_circuit).flatten())
        .collect()
}

pub fn link_process(p: inst::Process) -> l::Process {
    l::Process {
        name: p.name,
        conditions: p.timed_blocks.into_iter()
            .map(link_timed_block)
            .flatten()
            .collect()
    }
}

pub fn link_timed_block(p: inst::TimedBlock) -> Vec<Condition> {
    let rest = link_statement_list(p.block.clone()).into_iter();
    p.block.into_iter()
        .map(|i| match i {
            Statement::Not { input, into } => Condition::AtTime {time: p.time, run: Rc::new(l::Statement::Not {input, into})},
            Statement::And(inst::BinaryBuiltin{ a, b, into }) => Condition::AtTime {time: p.time, run: Rc::new(l::Statement::And(l::BinaryBuiltin{a, b, into}))},
            Statement::Or(inst::BinaryBuiltin{ a, b, into }) => Condition::AtTime {time: p.time, run: Rc::new(l::Statement::Nand(l::BinaryBuiltin{a, b, into}))},
            Statement::Nand(inst::BinaryBuiltin{ a, b, into }) => Condition::AtTime {time: p.time, run: Rc::new(l::Statement::Or(l::BinaryBuiltin{a, b, into}))},
            Statement::Nor(inst::BinaryBuiltin{ a, b, into }) => Condition::AtTime {time: p.time, run: Rc::new(l::Statement::Nor(l::BinaryBuiltin{a, b, into}))},
            Statement::Xor(inst::BinaryBuiltin{ a, b, into }) => Condition::AtTime {time: p.time, run: Rc::new(l::Statement::Xor(l::BinaryBuiltin{a, b, into}))},
            Statement::Xnor(inst::BinaryBuiltin{ a, b, into }) => Condition::AtTime {time: p.time, run: Rc::new(l::Statement::Xnor(l::BinaryBuiltin{a, b, into}))},
            Statement::Move(a, b) => Condition::AtTime {time: p.time, run: Rc::new(l::Statement::Move(a, b))},
            Statement::Set(a, b) => Condition::AtTime {time: p.time, run: Rc::new(l::Statement::Set(a, b.clone()))}
        })
        .chain(rest)
        .collect()
}