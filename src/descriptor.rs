use std::{error::Error, fs::File, io::Write, path::Path};

use crate::{
    cli::{ArgsHysteresis, ArgsPhase},
    matrix::Matrix,
};

use serde::Serialize;

#[derive(Serialize)]
pub struct PhaseDescriptor<'a> {
    pub config: &'a ArgsPhase,
    pub lattice: Matrix<Vec<usize>>,
    pub deg_mse: f64,
    pub deg_avg: f64,
    pub seed: u64,
    pub data_path: &'a Path,
}

#[derive(Serialize)]
pub struct HysteresisDescriptor<'a> {
    pub config: &'a ArgsHysteresis,
    pub lattice: Matrix<Vec<usize>>,
    pub deg_mse: f64,
    pub deg_avg: f64,
    pub seed: u64,
    pub data_path: &'a Path,
}

pub trait Descriptor: Serialize {
    fn save(&self, path: &String) -> Result<(), Box<dyn Error>> {
        let mut f = File::create(path)?;

        f.write_all(serde_json::to_string(&self)?.as_bytes())?;
        f.flush()?;

        Ok(())
    }
}

impl<'a> Descriptor for PhaseDescriptor<'a> {}

impl<'a> Descriptor for HysteresisDescriptor<'a> {}
