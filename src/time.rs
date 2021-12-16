
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Duration {
    nanos: u64,
}

impl Duration {
    pub fn from_nanos(nanos: u64) -> Self {
        Self {
            nanos,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd)]
pub struct Instant {
    nanos: u64,
    process_steps: u64,
    deltas: u64,
}

impl Instant {
    pub const START: Instant = Instant::nanos_from_start(0);

    pub const fn nanos_from_start(nanos: u64) -> Self {
        Self {
            nanos,
            process_steps: 0,
            deltas: 0,
        }
    }

    pub fn after(&self, d: &Duration) -> Instant {
        Instant {
            nanos: self.nanos + d.nanos,
            process_steps: 0,
            deltas: 0
        }
    }

    pub fn add_delta(&self) -> Instant {
        Self {
            nanos: self.nanos,
            process_steps: self.process_steps,
            deltas: self.deltas + 1
        }
    }

    pub fn add_process_step(&self) -> Instant {
        Self {
            nanos: self.nanos,
            process_steps: self.process_steps + 1,
            deltas: 0
        }
    }
}