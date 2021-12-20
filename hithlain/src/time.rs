use miette::Diagnostic;
use thiserror::Error;
use vcd::TimescaleUnit;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Duration {
    nanos: u64,
}

impl Duration {
    #[must_use]
    pub fn nanos(&self) -> u64 {
        self.nanos
    }

    #[must_use]
    pub fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum TimespecError {
    #[error("timespec doesn't have a valid suffix: {0}. must be 's', 'ms', 'us' or 'ns'")]
    InvalidSuffix(String),
    #[error("timespec must start with a number and end in a valid suffix: {0}.")]
    NotANumber(String),
}

pub fn parse_timespec(spec: &str) -> Result<Duration, TimespecError> {
    if let Some(i) = spec.strip_suffix("ns") {
        return Ok(Duration::from_nanos(
            i.parse::<u64>()
                .map_err(|_| TimespecError::NotANumber(spec.to_string()))?,
        ));
    }
    if let Some(i) = spec.strip_suffix("us") {
        return Ok(Duration::from_nanos(
            i.parse::<u64>()
                .map_err(|_| TimespecError::NotANumber(spec.to_string()))?
                * 1_000,
        ));
    }
    if let Some(i) = spec.strip_suffix("ms") {
        return Ok(Duration::from_nanos(
            i.parse::<u64>()
                .map_err(|_| TimespecError::NotANumber(spec.to_string()))?
                * 1_000_000,
        ));
    }
    if let Some(i) = spec.strip_suffix('s') {
        return Ok(Duration::from_nanos(
            i.parse::<u64>()
                .map_err(|_| TimespecError::NotANumber(spec.to_string()))?
                * 1_000_000_000,
        ));
    }

    Err(TimespecError::InvalidSuffix(spec.to_string()))
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd)]
pub struct Instant {
    nanos: u64,
    process_steps: u64,
    deltas: u64,
}

impl Instant {
    pub const START: Instant = Instant::nanos_from_start(0);

    #[must_use]
    pub const fn nanos_from_start(nanos: u64) -> Self {
        Self {
            nanos,
            process_steps: 0,
            deltas: 0,
        }
    }
    #[must_use]

    pub fn after(&self, d: &Duration) -> Instant {
        Instant {
            nanos: self.nanos + d.nanos,
            process_steps: 0,
            deltas: 0,
        }
    }
    #[must_use]

    pub fn add_delta(&self) -> Instant {
        Self {
            nanos: self.nanos,
            process_steps: self.process_steps,
            deltas: self.deltas + 1,
        }
    }

    #[must_use]
    pub fn add_process_step(&self) -> Instant {
        Self {
            nanos: self.nanos,
            process_steps: self.process_steps + 1,
            deltas: 0,
        }
    }

    #[must_use]
    pub fn nanos(&self) -> u64 {
        self.nanos
    }

    #[must_use]
    #[allow(clippy::unused_self)]
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
