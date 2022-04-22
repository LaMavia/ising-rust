mod matrix;
mod network;
mod simulation;

use std::{env, path::Path};

use simulation::{Simulation, SimulationConfig};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [ _, name ] if name.len() > 0 => {
            let mut s = Simulation::new(
                500,
                SimulationConfig {
                    temp: 0.5f64,
                    h: 0f64,
                    j: 1f64,
                    kb: 1f64,
                    equilibrium_steps: 5,
                },
            );

            match s.simulate_hysteresis(
                Path::new(&format!("{}.csv", name)),
                simulation::HysteresisConfig {
                    h_min: -2.2f64,
                    h_max: 2.2_f64,
                    h_step: 0.01f64, // 0.001_f64,
                },
            ) {
                Ok(_) => println!("simulation done!"),
                Err(e) => println!("{e}"),
            }
        }
        x => println!("error: {:?}", x),
    }
}
