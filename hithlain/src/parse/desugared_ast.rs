use std::rc::Rc;
use crate::parse::scope::VariableRef;
use crate::parse::ast::{Constant, Variable};
use crate::time::Instant;
use crate::parse::span::Span;

#[derive(Clone)]
pub struct BinaryBuiltin {
    pub(crate) a: VariableRef,
    pub(crate) b: VariableRef,
    pub(crate) into: VariableRef,
}

#[derive(Clone)]
pub enum Statement {
    Assert(VariableRef, Span),
    Not {
        input: VariableRef,
        into: VariableRef,
    },
    And(BinaryBuiltin),
    Or(BinaryBuiltin),
    Nand(BinaryBuiltin),
    Nor(BinaryBuiltin),
    Xor(BinaryBuiltin),
    Xnor(BinaryBuiltin),
    Custom{
        inputs: Vec<VariableRef>,
        circuit: Rc<Circuit>,
        into: Vec<VariableRef>,
    },
    Move(VariableRef, VariableRef),
    Set(VariableRef, Constant),
}

pub struct Circuit {
    pub(crate) name: Variable,

    pub(crate) inputs: Vec<VariableRef>,
    pub(crate) outputs: Vec<VariableRef>,

    pub(crate) body: Vec<Statement>,
}

pub struct TimedBlock {
    pub(crate) time: Instant,
    pub(crate) block: Vec<Statement>,
}

pub struct Process {
    pub(crate) name: Variable,
    pub(crate) timed_blocks: Vec<TimedBlock>,

    pub(crate) inputs: Vec<VariableRef>,
    pub(crate) outputs: Vec<VariableRef>,
}

pub struct Program {

    pub(crate) circuits: Vec<Rc<Circuit>>,
    pub(crate) tests: Vec<Rc<Process>>,
}

