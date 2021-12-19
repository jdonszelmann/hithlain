use crate::sim::instantiated_ast as inst;
use crate::sim::linked_ast as l;
use crate::sim::linked_ast::Condition;
use std::rc::Rc;
use crate::time::Instant;
use crate::sim::instantiated_ast::Statement;


#[must_use]
pub fn link_statement_list(statements: Vec<Statement>, do_sets: bool) -> Vec<Condition> {
    macro_rules! binary_stmt {
        ($path: path, $($tt: tt)*) => {
            {
                let inst::BinaryBuiltin{a, b, into} = $($tt)*;
                let stmt = Rc::new($path(l::BinaryBuiltin{a: a.clone(), b: b.clone(), into}));

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
        .flat_map(|i| match i {
            inst::Statement::Not { input, into } => {
                vec![Condition::WhenChanges { variable: input.clone(), run:  Rc::new(l::Statement::Not{input, into})}]
            }
            inst::Statement::And(b) => binary_stmt!(l::Statement::And, b),
            inst::Statement::Or(b) => binary_stmt!(l::Statement::Or, b),
            inst::Statement::Nand(b) => binary_stmt!(l::Statement::Nand, b),
            inst::Statement::Nor(b) => binary_stmt!(l::Statement::Nor, b),
            inst::Statement::Xor(b) => binary_stmt!(l::Statement::Xor, b),
            inst::Statement::Xnor(b) => binary_stmt!(l::Statement::Xnor, b),
            inst::Statement::Move(a, b) => {
                vec![Condition::WhenChanges { variable: b.clone(), run:  Rc::new(l::Statement::Move(a, b))}]
            },
            inst::Statement::Set(a, b) => if do_sets {
                vec![Condition::AtTime { time: Instant::START, run:  Rc::new(l::Statement::Set(a, b))}]
            } else {
                vec![]
            },
            Statement::CreateCircuitInstance(circuit) => {
                link_circuit(circuit)
            }
            Statement::Assert(_, _) => vec![], // ignore asserts in normal statements (shouldn't be parsed anyway)
        })
        .collect()
}

#[must_use]
pub fn link_circuit(circuit: inst::Circuit) -> Vec<Condition> {
    link_statement_list(circuit.body, true)
        .into_iter()
        .collect()
}

#[must_use]
pub fn link_process(p: inst::Process) -> l::Process {
    l::Process {
        name: p.name,
        conditions: p.timed_blocks.into_iter()
            .flat_map(link_timed_block)
            .collect()
    }
}

#[must_use]
pub fn link_timed_block(p: inst::TimedBlock) -> Vec<Condition> {
    let rest = link_statement_list(p.block.clone(), false).into_iter();

    // let mut time = p.time;
    // macro_rules! get_time {
    //     () => {
    //         {
    //             let orig = time;
    //             time = time.add_process_step();
    //             orig
    //         }
    //     };
    // }

    p.block.into_iter()
        .filter_map(|i| match i {
            Statement::Set(a, b) => Some(Condition::AtTime {
                time: p.time,
                run: Rc::new(l::Statement::Set(a, b))
            }),
            Statement::Assert(e, span) => Some(Condition::AtTime {
                time: p.time.add_process_step(),
                run: Rc::new(l::Statement::Assert(e, span))
            }),
            _ => None,
        })
        .chain(rest)
        .collect()
}