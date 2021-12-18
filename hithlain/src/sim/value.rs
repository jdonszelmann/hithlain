use derive_more::From;

use crate::sim::value::Value::Bit;
use std::ops::{BitXor, BitAnd, BitOr, Not};
use miette::{NamedSource, SourceSpan, Diagnostic};
use thiserror::Error;
use crate::parse::ast::Constant;

#[derive(Debug, Error, Diagnostic)]
pub enum ValueError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    AssertionError(#[from] TypeMismatch),
}

#[derive(Error, Debug, Diagnostic)]
#[error("type mismatch")]
#[diagnostic()]
pub struct TypeMismatch {
    #[source_code]
    src: NamedSource,

    #[label("here")]
    span: SourceSpan,
}


#[derive(From, Debug, Clone)]
pub enum Value {
    Bit(bool)
}

impl From<Constant> for Value {
    fn from(c: Constant) -> Self {
        match c {
            Constant::Bit(n) => Value::Bit(n)
        }
    }
}

impl From<&Constant> for Value {
    fn from(c: &Constant) -> Self {
        match c.clone() {
            Constant::Bit(n) => Value::Bit(n)
        }
    }
}

impl BitXor for Value {
    type Output = Result<Value, ValueError>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bit(a), Bit(b)) => Ok(Bit(a ^ b))
        }
    }
}

impl BitAnd for Value {
    type Output = Result<Value, ValueError>;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bit(a), Bit(b)) => Ok(Bit(a & b))
        }
    }
}

impl BitOr for Value {
    type Output = Result<Value, ValueError>;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bit(a), Bit(b)) => Ok(Bit(a | b))
        }
    }
}

impl Not for Value {
    type Output = Result<Value, ValueError>;

    fn not(self) -> Self::Output {
        match self {
            Bit(a) => Ok(Bit(!a))
        }
    }
}