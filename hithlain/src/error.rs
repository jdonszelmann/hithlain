use miette::{Diagnostic, GraphicalReportHandler};

pub trait NiceUnwrap {
    type T;

    fn nice_unwrap(self) -> Self::T;
}

impl<T, E> NiceUnwrap for Result<T, E> where E: Diagnostic {
    type T = T;

    fn nice_unwrap(self) -> Self::T {
        match self {
            Ok(i) => i,
            Err(e) => {
                let mut s = String::new();
                GraphicalReportHandler::new().render_report(&mut s, &e).unwrap();
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