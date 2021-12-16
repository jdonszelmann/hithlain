use crate::time::Instant;
use std::rc::Rc;
use crate::sim::linked_ast::Statement;
use std::cmp::Ordering;

pub struct Signal {
    time: Instant,
    action: Vec<Rc<Statement>>
}

impl PartialEq<Self> for Signal {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl PartialOrd for Signal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Eq for Signal {}


impl Ord for Signal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}