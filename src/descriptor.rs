use std::{error::Error, fs::File, io::Write, path::Path};

use crate::{cli::ArgsPhase, matrix::Matrix};

use serde::Serialize;

#[derive(Serialize)]
pub struct PhaseDescriptor<'a> {
    pub config: &'a ArgsPhase,
    pub lattice: Matrix<Vec<usize>>,
    pub deg_mse: f64,
    pub deg_avg: f64,
    pub seed: u64,
    pub data_path: &'a Path,
    pub path: &'a Path,
}

impl<'a> PhaseDescriptor<'a> {
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let mut f = File::create(self.path)?;

        f.write_all(serde_json::to_string(&self)?.as_bytes())?;
        f.flush()?;

        Ok(())
    }
}
