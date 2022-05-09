use std::{error::Error, fs::File, io::Write, path::Path};

use crate::{cli::ArgsPhase, matrix::Matrix};

use serde::Serialize;

#[derive(Serialize)]
pub struct PhaseDescriptor {
    config: ArgsPhase,
    lattice: Matrix<Vec<usize>>,
    seed: usize,
    data_path: Box<Path>,
    path: Box<Path>,
}

impl PhaseDescriptor {
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut f = File::create(path)?;

        f.write_all(serde_json::to_string(&self)?.as_bytes())?;
        f.flush()?;

        Ok(())
    }
}
