use miette::{Diagnostic, GraphicalReportHandler};
use thiserror::Error;
use crate::parse::parser::ParseError;
use crate::parse::desugar::DesugarError;
use crate::vcd::VcdError;
use crate::sim::SimulationError;
use crate::parse::source::SourceError;
use crate::parse::lexer::LexError;
use crate::time::TimespecError;
#[cfg(not(test))]
use std::process::exit;

pub trait NiceUnwrap {
    type T;

    fn nice_unwrap_panic(self) -> Self::T;
    fn nice_unwrap(self) -> Self::T;
}

impl<T, E> NiceUnwrap for Result<T, E> where E: Diagnostic {
    type T = T;

    fn nice_unwrap_panic(self) -> Self::T {
        match self {
            Ok(i) => i,
            Err(e) => {
                let mut s = String::new();
                GraphicalReportHandler::new().with_links(true).render_report(&mut s, &e).unwrap();
                panic!("{}", s)
            },
        }
    }

    fn nice_unwrap(self) -> Self::T {
        match self {
            Ok(i) => i,
            Err(e) => {
                let mut s = String::new();
                GraphicalReportHandler::new().with_links(true).render_report(&mut s, &e).unwrap();
                #[cfg(not(test))]
                eprintln!("{}", s);
                #[cfg(not(test))]
                exit(1);
                #[cfg(test)]
                panic!("{}", s)
            },
        }
    }
}

pub trait Warn {
    fn warn(&self);
}

impl<E> Warn for E where E: Diagnostic {
    fn warn(&self) {
        let mut s = String::new();
        GraphicalReportHandler::new().render_report(&mut s, self).unwrap();
        eprintln!("{}", s);
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum HithlainError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseError(#[from] ParseError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    LexError(#[from] LexError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    TimespecError(#[from] TimespecError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    DesugarError(#[from] DesugarError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    VcdError(#[from] VcdError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    SimulationError(#[from] SimulationError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    SourceError(#[from] SourceError),
}