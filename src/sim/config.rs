use std::path::PathBuf;
use std::io::Write;
use std::fs::File;
use crate::vcd::VcdError;
use thiserror::Error;
use miette::Diagnostic;
use crate::time::Duration;

#[derive(Error, Debug, Diagnostic)]
#[error("couldn't open file at {:?}", self.name)]
#[diagnostic()]
pub struct FileNotFound {
    name: PathBuf
}

pub enum VcdPath {
    InMemory,
    Path(PathBuf),
}

impl VcdPath {
    pub fn writer(&self) -> Result<Box<dyn Write>, VcdError> {
        Ok(match self {
            VcdPath::InMemory => {
                Box::new(Vec::<u8>::new())
            }
            VcdPath::Path(p) => {
                Box::new(File::create(&p).map_err(|_| {
                    FileNotFound {
                        name: p.clone()
                    }
                })?)
            }
        })
    }
}

pub struct SimulationConfig {
    pub create_vcd: bool,
    pub vcd_path: VcdPath,
    pub vcd_overshoot_duration: Duration,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            create_vcd: true,
            vcd_path: VcdPath::InMemory,
            vcd_overshoot_duration: Duration::from_nanos(10),
        }
    }
}