use std::ops::Deref;
use std::sync::Arc;
use miette::NamedSource;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct InnerSource {
    text: String,
    name: String
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Source(Arc<InnerSource>);



impl Into<NamedSource> for Source {
    fn into(self) -> NamedSource {
        NamedSource::new(&self.0.name, self.0.text.clone())
    }
}

impl Source {
    pub fn new(text: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        Self(Arc::new(InnerSource {
            text: text.as_ref().to_string(),
            name: name.as_ref().to_string()
        }))
    }

    #[cfg(test)]
    pub(crate) fn test(text: impl AsRef<str>) -> Source {
        Self::new(text, "test")
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn text(&self) -> &str {
        self.deref()
    }
}

impl Deref for Source {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0.text
    }
}