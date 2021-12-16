use crate::parse::ast::{Constant, Variable};
use crate::time::Instant;
use std::collections::HashMap;
use crate::sim::instantiate::UniqueVariableRef;

#[derive(Clone)]
pub struct BinaryBuiltin {
    pub(crate) a: UniqueVariableRef,
    pub(crate) b: UniqueVariableRef,
    pub(crate) into: UniqueVariableRef,
}

#[derive(Clone)]
pub enum Statement {
    Not {
        input: UniqueVariableRef,
        into: UniqueVariableRef,
    },
    And(BinaryBuiltin),
    Or(BinaryBuiltin),
    Nand(BinaryBuiltin),
    Nor(BinaryBuiltin),
    Xor(BinaryBuiltin),
    Xnor(BinaryBuiltin),
    Move(UniqueVariableRef, UniqueVariableRef),
    Set(UniqueVariableRef, Constant),
}

pub struct Circuit {
    pub(crate) inputs: Vec<UniqueVariableRef>,
    pub(crate) outputs: Vec<UniqueVariableRef>,

    pub(crate) inner_circuits: Vec<Circuit>,

    pub(crate) body: Vec<Statement>,
}

pub struct TimedBlock {
    pub(crate) time: Instant,
    pub(crate) block: Vec<Statement>,
}

pub struct Process {
    pub(crate) name: Variable,
    pub(crate) timed_blocks: Vec<TimedBlock>,

    pub(crate) inner_circuits: Vec<Circuit>,

    pub(crate) inputs: Vec<UniqueVariableRef>,
    pub(crate) outputs: Vec<UniqueVariableRef>,
}

pub struct Program {
    pub(crate) circuits: Vec<Circuit>,
    pub(crate) tests: Vec<Process>,
}

