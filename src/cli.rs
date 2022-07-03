use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

use clap::Parser;
use serde::Serialize;

#[derive(Parser, Debug, Serialize)]
pub struct ArgsHysteresis {
    #[clap(short, long, default_value_t = 100)]
    pub size: usize,

    #[clap(short, long, multiple_values=true)]
    pub temps: Vec<f64>,

    #[clap(short, long, default_value_t = 50)]
    pub eq_steps: usize,

    #[clap(short, long, default_value_t = 2.5f64)]
    pub h_max: f64,

    #[clap(short, long, default_value_t = 0.01f64)]
    pub h_step: f64,

    #[clap(long, multiple_values=true)]
    pub seeds: Vec<u64>
}

#[derive(Parser, Debug, Serialize)]
pub struct ArgsPhase {
    #[clap(long, default_value_t = 100)]
    pub size: usize,

    #[clap(long, multiple_values=true)]
    pub eq_steps: Vec<usize>,

    #[clap(long, default_value_t = 0.0001f64)]
    pub t_min: f64,

    #[clap(long, default_value_t = 2f64)]
    pub t_max: f64,

    #[clap(long, default_value_t = 0.01f64)]
    pub t_step: f64,

    #[clap(long, multiple_values=true)]
    pub seeds: Vec<u64>
}

#[derive(Debug)]
pub struct ArgError {}

impl Display for ArgError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "")
    }
}

impl Error for ArgError {}
