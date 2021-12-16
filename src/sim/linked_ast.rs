use std::rc::Rc;
use crate::parse::ast::{Constant, Variable};
use crate::time::Instant;
use crate::sim::instantiate::UniqueVariableRef;
use crate::parse::span::Span;

#[derive(Debug)]
pub enum Condition {
    AtTime {
        time: Instant,
        run: Rc<Statement>,
    },
    WhenChanges {
        variable: UniqueVariableRef,
        run: Rc<Statement>,
    }
}

#[derive(Debug)]
pub struct BinaryBuiltin {
    pub(crate) a: UniqueVariableRef,
    pub(crate) b: UniqueVariableRef,
    pub(crate) into: UniqueVariableRef,
}

#[derive(Debug)]
pub enum Statement {
    Assert(UniqueVariableRef, Span),
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

pub struct Process {
    pub(crate) name: Variable,

    pub(crate) conditions: Vec<Condition>,
}


pub struct Program {
    pub(crate) tests: Vec<Process>,
}

