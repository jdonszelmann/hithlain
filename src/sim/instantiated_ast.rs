use crate::parse::ast::{Constant, Variable};
use crate::time::Instant;
use crate::parse::desugared_ast as d;

use crate::sim::instantiate::UniqueVariableRef;
use crate::parse::span::Span;
use std::rc::Rc;
use derive_more::From;

#[derive(From, Clone)]
pub enum Package {
    Circuit(Rc<d::Circuit>),
    Test(Rc<d::Process>),
}

impl Package {
    pub fn name(&self) -> &Variable {
        match self {
            Package::Circuit(c) => &c.name,
            Package::Test(p) => &p.name,
        }
    }
}

#[derive(Clone)]
pub struct LocalizedVariable {
    pub(crate) variable: Variable,
    pub(crate) path: Rc<Vec<Package>>,
}

#[derive(Clone)]
pub struct BinaryBuiltin {
    pub(crate) a: UniqueVariableRef,
    pub(crate) b: UniqueVariableRef,
    pub(crate) into: UniqueVariableRef,
}

#[derive(Clone)]
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
    CreateInstance(Circuit),
}

#[derive(Clone)]
pub struct Circuit {
    pub(crate) inputs: Vec<UniqueVariableRef>,
    pub(crate) outputs: Vec<UniqueVariableRef>,

    pub(crate) body: Vec<Statement>,
}

pub struct TimedBlock {
    pub(crate) time: Instant,
    pub(crate) block: Vec<Statement>,
}

pub struct Process {
    pub(crate) name: Variable,
    pub(crate) timed_blocks: Vec<TimedBlock>,

    pub(crate) inputs: Vec<UniqueVariableRef>,
    pub(crate) outputs: Vec<UniqueVariableRef>,
}

pub struct Program {
    pub(crate) tests: Vec<Process>,
}

