use vcd::TimescaleUnit;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Duration {
    nanos: u64,
}

impl Duration {
    pub fn nanos(&self) -> u64 {
        self.nanos
    }

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

    pub fn nanos(&self) -> u64 {
        self.nanos
    }

    pub fn vcd_scale(&self) -> TimescaleUnit {
        // always ns as that's the base we measure time in (integer)
        TimescaleUnit::NS
        // match self.nanos {
        //     0..=999 => TimescaleUnit::NS,
        //     1_000..=999_999 => TimescaleUnit::US,
        //     1_000_000..=999_999_999 => TimescaleUnit::MS,
        //     _ => TimescaleUnit::S
        // }
    }
}