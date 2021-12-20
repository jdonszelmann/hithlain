use miette::{Diagnostic, NamedSource};
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum SourceError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    SourceDoesntExistError(#[from] SourceDoesntExistError),

    #[diagnostic(transparent)]
    #[error(transparent)]
    SourceReadError(#[from] SourceReadError),
}

#[derive(Debug, Error, Diagnostic)]
#[error("file name doesnt exist: {}", name)]
pub struct SourceDoesntExistError {
    name: String,
}

#[derive(Debug, Error, Diagnostic)]
#[error("failed to read from: {}", name)]
pub struct SourceReadError {
    name: String,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct InnerSource {
    text: String,
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Source(Arc<InnerSource>);

#[allow(clippy::from_over_into)]
impl Into<NamedSource> for Source {
    fn into(self) -> NamedSource {
        NamedSource::new(&self.0.name, self.0.text.clone())
    }
}

impl Source {
    pub fn new(text: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        Self(Arc::new(InnerSource {
            text: text.as_ref().to_string(),
            name: name.as_ref().to_string(),
        }))
    }

    pub fn file(name: &str) -> Result<Source, SourceError> {
        let mut f = File::open(name).map_err(|_| SourceDoesntExistError {
            name: name.to_string(),
        })?;

        let mut buf = String::new();
        f.read_to_string(&mut buf).map_err(|_| SourceReadError {
            name: name.to_string(),
        })?;

        Ok(Self::new(buf, name))
    }

    #[cfg(test)]
    pub(crate) fn test(text: impl AsRef<str>) -> Source {
        Self::new(text, "test")
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.0.name
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &*self
    }
}

impl Deref for Source {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0.text
    }
}
