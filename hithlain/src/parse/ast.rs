use crate::parse::span::Span;
use derivative::Derivative;
use crate::time::{Duration, Instant};
use crate::sim::instantiated_ast::{Package, LocalizedVariable};
use std::rc::Rc;

#[derive(Debug, Derivative, Clone)]
#[derivative(PartialEq, Hash)]
pub struct Variable (
    pub(crate) String,

    #[derivative(PartialEq="ignore")]
    #[derivative(Hash="ignore")]
    pub(crate) Option<Span>
);

impl Variable {
    pub fn localize(&self, path: Rc<Vec<Package>>) -> LocalizedVariable {
        LocalizedVariable {
            variable: self.clone(),
            path,
        }
    }
}

impl Eq for Variable {}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Constant {
    Bit(bool),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Atom {
    Variable(Variable),
    Constant(Constant),
    Expr(Box<Expr>),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum BinaryAction {
    And,
    Or,
    Nand,
    Nor,
    Xor,
    Xnor,
    Custom(Variable),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum UnaryAction {
    Not
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum NaryAction {
    UnaryAction(UnaryAction),
    BinaryAction(BinaryAction),
    Custom(Variable),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Expr {
    BinaryOp {
        a: Box<Expr>,
        b: Box<Expr>,
        action: BinaryAction,
    },
    NaryOp {
        params: Vec<Expr>,
        action: NaryAction,
    },
    Atom(Atom)
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Assignment {
    pub(crate) into: Vec<Variable>,
    pub(crate) expr: Expr,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Statement {
    Assignment(Assignment),
    Assert {
        expr: Expr,
        span: Span,
    }
}


#[derive(Debug, Eq, PartialEq, Hash)]
pub enum TimeSpec {
    After(Duration),
    At(Instant),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum StatementOrTime {
    Statement(Statement),
    Time(TimeSpec),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Circuit {
    pub(crate) name: Variable,

    pub(crate) inputs: Vec<Variable>,
    pub(crate) outputs: Vec<Variable>,

    pub(crate) body: Vec<Statement>,
}

pub struct Process {
    pub(crate) name: Variable,

    pub(crate) inputs: Vec<Variable>,
    pub(crate) outputs: Vec<Variable>,

    pub(crate) body: Vec<StatementOrTime>,
}

pub struct Test {
    pub(crate) name: Variable,
    pub(crate) body: Vec<StatementOrTime>,
}

pub struct Program {
    pub(crate) circuits: Vec<Circuit>,
    pub(crate) processes: Vec<Process>,
    pub(crate) tests: Vec<Test>,
}


