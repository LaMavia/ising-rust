mod cli;
mod descriptor;
mod matrix;
mod network;
mod simulation;

use std::{env, error::Error, fs::create_dir_all, io, path::Path};

use clap::*;
use rand::SeedableRng;
use simulation::{Simulation, SimulationConfig};

use crate::cli::ArgError;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut rand = rand_chacha::ChaCha20Rng::seed_from_u64(2);

    let result: Result<String, Box<dyn Error>> = match args.get(1) {
        Some(simulation_type) if simulation_type.as_str() == "hys" => {
            let args = cli::ArgsHysteresis::parse_from(env::args().skip(1));

            let mut s = Simulation::new(
                args.size,
                SimulationConfig {
                    temp: args.temp,
                    h: 0f64,
                    j: 1f64,
                    kb: 1f64,
                    equilibrium_steps: args.eq_steps,
                    network_type: args.network_type,
                },
                &mut rand,
            );

            match s.simulate_hysteresis(
                Path::new(&format!("{}.csv", args.name)),
                simulation::HysteresisConfig {
                    h_min: args.h_min,
                    h_max: args.h_max,
                    h_step: args.h_step,
                },
                &mut rand,
            ) {
                Ok(_) => {
                    eprintln!("simulation done!");
                    Ok(format!("hys {name} {plot_title}", name = args.name, plot_title=format!("size={size},T={temp},eq_steps={eq_steps},network_type={network_type},H_step={h_step}", size=args.size, temp=args.temp, eq_steps=args.eq_steps, network_type=args.network_type.to_string(),h_step=args.h_step)))
                }
                Err(e) => Err(e),
            }
        }
        Some(simulation_type) if simulation_type.as_str() == "phase" => {
            let args = cli::ArgsPhase::parse_from(env::args().skip(1));
            let mut s = Simulation::new(
                args.size,
                SimulationConfig {
                    temp: args.t_min,
                    h: 0f64,
                    j: 1f64,
                    kb: 1f64,
                    equilibrium_steps: args.eq_steps,
                    network_type: args.network_type,
                },
                &mut rand,
            );

            /* Todo: 
                1. join into a single process for comparison;
                2. think of a reasonable directory structure
            */

            let data_path_str = format!(
                "data/{}/simulation_type/size={}_step={}_max={}/data.csv",
                args.name, args.size, args.t_step, args.t_max
            );
            let data_path = Path::new(&data_path_str);

            match s.simulate_phase(
                data_path,
                simulation::PhaseConfig {
                    t_min: args.t_min,
                    t_max: args.t_max,
                    t_step: args.t_step,
                },
                &mut rand,
            ) {
                Ok(_) => {
                    create_dir_all(data_path)?;
                    eprintln!("simulation done!");
                    Ok(format!(
                        "phase {name} {plot_title}",
                        name = args.name,
                        plot_title = format!("size={size},eq_steps={eq_steps},network_type={network_type},T_step={t_step}", size=args.size, eq_steps=args.eq_steps, network_type=args.network_type.to_string(), t_step=args.t_step)
                    ))
                }
                Err(e) => Err(e),
            }
        }
        x => {
            eprintln!("unknown simulation type {:?}", x);
            Err(Box::new(ArgError {}))
        }
    };

    match result {
        Err(e) => Err(e),
        Ok(r) => Ok(println!("{}", r)),
    }
}
