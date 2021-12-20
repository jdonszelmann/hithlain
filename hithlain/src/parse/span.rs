use crate::parse::source::Source;
use logos::Span as LogosSpan;
use miette::SourceSpan;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    start: usize,
    length: usize,
    source: Source,
}

impl Span {
    pub(crate) fn from_logos(l: LogosSpan, source: Source) -> Self {
        Self {
            start: l.start,
            length: l.end - l.start,
            source,
        }
    }

    #[must_use]
    pub fn source(&self) -> &Source {
        &self.source
    }

    #[must_use]
    pub fn merge_with(&self, other: &Self) -> Self {
        let start = self.start.min(other.start);
        let end = (self.start + self.length).max(other.start + other.length);

        Span {
            start,
            length: end - start,
            source: self.source.clone(),
        }
    }

    /// # Panics
    /// When spans is an empty slice.
    #[must_use]
    pub fn merge(spans: &[Span]) -> Self {
        assert!(!spans.is_empty());

        let mut span = spans[0].clone();

        for i in &spans[1..] {
            span = span.merge_with(i);
        }

        span
    }
}

#[allow(clippy::from_over_into)]
impl Into<SourceSpan> for Span {
    fn into(self) -> SourceSpan {
        SourceSpan::new(self.start.into(), self.length.into())
    }
}
