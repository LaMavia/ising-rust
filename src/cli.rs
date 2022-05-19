use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

use crate::network::NetworkType;
use clap::Parser;
use serde::Serialize;

#[derive(Parser, Debug, Serialize)]
pub struct ArgsHysteresis {
    #[clap(short, long, default_value_t = 100)]
    pub size: usize,

    #[clap(short, long)]
    pub temp: f64,

    #[clap(short, long, default_value_t = 50)]
    pub eq_steps: usize,

    #[clap(short, long, parse(from_str), default_value = "regular")]
    pub network_type: NetworkType,

    #[clap(short, long, default_value_t=-2.5f64)]
    pub h_min: f64,

    #[clap(short, long, default_value_t = 2.5f64)]
    pub h_max: f64,

    #[clap(short, long, default_value_t = 0.01f64)]
    pub h_step: f64,
}

#[derive(Parser, Debug, Serialize)]
pub struct ArgsPhase {
    #[clap(short, long, default_value_t = 100)]
    pub size: usize,

    #[clap(short, long, default_value_t = 50)]
    pub eq_steps: usize,

    // #[clap(short, long, parse(from_str), default_value = "regular")]
    // pub network_type: NetworkType,

    #[clap(short, long, default_value_t = 0.0001f64)]
    pub t_min: f64,

    #[clap(short, long, default_value_t = 2f64)]
    pub t_max: f64,

    #[clap(short, long, default_value_t = 0.01f64)]
    pub t_step: f64,
}

#[derive(Debug)]
pub struct ArgError {}

impl Display for ArgError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "")
    }
}

impl Error for ArgError {}
