use std::rc::Rc;
use crate::parse::ast::{Constant, Variable};
use crate::time::Instant;
use crate::sim::instantiate::UniqueVariableRef;

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

pub struct BinaryBuiltin {
    pub(crate) a: UniqueVariableRef,
    pub(crate) b: UniqueVariableRef,
    pub(crate) into: UniqueVariableRef,
}

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

pub struct Process {
    pub(crate) name: Variable,

    pub(crate) conditions: Vec<Condition>,
}


pub struct Program {
    pub(crate) conditions: Vec<Condition>,
    pub(crate) tests: Vec<Process>,
}

