use logos::Span as LogosSpan;
use miette::SourceSpan;
use crate::parse::source::Source;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    start: usize,
    length: usize,
    source: Source
}

impl Span {
    pub(crate) fn from_logos(l: LogosSpan, source: Source) -> Self {
        Self {
            start: l.start,
            length: l.end - l.start,
            source
        }
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn merge_with(&self, other: &Self) -> Self {
        let start = self.start.min(other.start);
        let end = (self.start + self.length).max(other.start + other.length);

        Span {
            start,
            length: end - start,
            source: self.source.clone()
        }
    }

    pub fn merge(spans: &[Span]) -> Self {
        assert!(spans.len() > 0);

        let mut span = spans[0].clone();

        for i in &spans[1..] {
            span = span.merge_with(&i);
        }

        span
    }
}


impl Into<SourceSpan> for Span {
    fn into(self) -> SourceSpan {
        SourceSpan::new(self.start.into(), self.length.into())
    }
}