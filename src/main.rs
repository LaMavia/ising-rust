mod child;
mod cli;
mod descriptor;
mod matrix;
mod network;
mod simulation;
mod mathy;

use std::sync::mpsc::{self, Sender};
use std::{thread};
use std::time::Duration;
use std::{env, error::Error, fs, path::Path};

use child::ChildMsg;
use clap::*;
use cli::ArgsHysteresis;
use descriptor::{Descriptor, HysteresisDescriptor, PhaseDescriptor};
use network::NetworkType;
use rand::SeedableRng;
use simulation::{Simulation, SimulationConfig};

use crate::child::Child;
use crate::cli::{ArgError, ArgsPhase};

// add extra params, split into two

fn make_data_path_phase(
    network_type: NetworkType,
    size: usize,
    step: f64,
    max: f64,
    seed: u64,
) -> String {
    format!(
        "data/{}/phase/size={}_step={}_max={}_seed={}",
        network_type.to_string(),
        size,
        step,
        max,
        seed
    )
}

fn make_data_path_hys(
    network_type: NetworkType,
    size: usize,
    step: f64,
    max: f64,
    temp: f64,
    seed: u64,
) -> String {
    format!(
        "data/{}/hys/size={}_step={}_max={}_temp={}_seed={}",
        network_type.to_string(),
        size,
        step,
        max,
        temp,
        seed
    )
}

fn prepare_data_path(data_dir: &String) -> Result<String, Box<dyn Error>> {
    let data_path_str = format!("{}/data.csv", data_dir);

    fs::remove_dir_all(&data_dir).unwrap_or(());
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&format!("{}/frames/", data_dir))?;

    Ok(data_path_str)
}

fn eq_threshold_of_type(network_type: NetworkType) -> f64 {
    1e-6
}

fn save<D: Descriptor>(
    data_dir_str: &String,
    desc: &D,
    tx: &Sender<ChildMsg>,
    name: &String,
) -> Result<String, Box<dyn Error>> {
    let desc_path_str = format!("{data_dir_str}/desc.json");

    desc.save(&desc_path_str)?;

    send!(
        final tx,
        name,
        desc_path_str.to_owned()
    );

    thread::sleep(Duration::from_millis(5));

    Ok(desc_path_str)
}

fn run_phase(
    rand_seed: u64,
    args: &ArgsPhase,
    network_type: NetworkType,
    eq_steps: usize,
    s0: f64,
    tx: Sender<ChildMsg>,
    name: String,
) -> Result<String, Box<dyn Error>> {
    let mut rand = rand_chacha::ChaCha20Rng::seed_from_u64(rand_seed);

    let data_dir_str =
        make_data_path_phase(network_type, args.size, args.t_step, args.t_max, rand_seed);
    let data_path_str = prepare_data_path(&data_dir_str)?;
    let data_path = Path::new(&data_path_str);

    let mut s = Simulation::new(
        args.size,
        SimulationConfig {
            temp: args.t_min,
            h: 0f64,
            j: 1f64,
            kb: 1f64,
            equilibrium_steps: eq_steps,
            network_type: network_type,
            eq_threshold: eq_threshold_of_type(network_type),
        },
        &mut rand,
        name,
        tx,
        data_dir_str.to_owned()
    );

    match s.simulate_phase(
        data_path,
        simulation::PhaseConfig {
            t_min: args.t_min,
            t_max: args.t_max,
            t_step: args.t_step,
            s0,
        },
        &mut rand,
    ) {
        Ok(_) => {
            let desc = PhaseDescriptor {
                config: args,
                lattice: s.network.lattice,
                seed: rand_seed,
                deg_avg: s.network.deg_avg,
                deg_mse: s.network.deg_mse,
                data_path: data_path,
            };

            save(&data_dir_str, &desc, &s.tx, &s.name)
        }
        Err(e) => Err(e),
    }
}

fn run_hysteresis(
    rand_seed: u64,
    args: &ArgsHysteresis,
    network_type: NetworkType,
    temp: f64,
    tx: Sender<ChildMsg>,
    name: String,
) -> Result<String, Box<dyn Error>> {
    let mut rand = rand_chacha::ChaCha20Rng::seed_from_u64(rand_seed);

    let data_dir_str = &make_data_path_hys(
        network_type,
        args.size,
        args.h_step,
        args.h_max,
        temp,
        rand_seed,
    );
    let data_path_str = prepare_data_path(data_dir_str)?;
    let data_path = Path::new(&data_path_str);

    let mut s = Simulation::new(
        args.size,
        SimulationConfig {
            temp,
            h: 0f64,
            j: 1f64,
            kb: 1f64,
            equilibrium_steps: args.eq_steps,
            network_type: network_type,
            eq_threshold: eq_threshold_of_type(network_type),
        },
        &mut rand,
        name,
        tx,
        data_dir_str.to_owned()
    );

    match s.simulate_hysteresis(
        &data_path,
        simulation::HysteresisConfig {
            h_min: -args.h_max,
            h_max: args.h_max,
            h_step: args.h_step,
        },
        &mut rand,
    ) {
        Ok(_) => {
            let desc = HysteresisDescriptor {
                config: args,
                lattice: s.network.lattice,
                seed: rand_seed,
                deg_avg: s.network.deg_avg,
                deg_mse: s.network.deg_mse,
                data_path: data_path,
            };

            save(&data_dir_str, &desc, &s.tx, &s.name)
        }

        Err(e) => Err(e),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut children = vec![];
    let (tx, rx) = mpsc::channel::<ChildMsg>();

    let result: Result<String, Box<dyn Error>> = match args.get(1) {
        Some(simulation_type) if simulation_type.as_str() == "hys" => {
            for network_type in vec![NetworkType::Regular, NetworkType::Irregular] {
                let args = cli::ArgsHysteresis::parse_from(env::args().skip(1));

                for seed in args.seeds.into_iter() {
                    let temps = args.temps.to_owned();

                    for temp in temps.into_iter() {
                        let args = cli::ArgsHysteresis::parse_from(env::args().skip(1));

                        let name =
                            format!("{}, seed={}, T={}", network_type.to_string(), seed, temp);
                        let tx_ = tx.clone();

                        children.push(Child::make(&name.to_owned(), move || {
                            run_hysteresis(seed, &args, network_type, temp.to_owned(), tx_, name)
                                .unwrap();
                        }));
                    }
                }
            }

            print!("{} ", simulation_type);

            Ok("".to_string())
        }
        Some(simulation_type) if simulation_type.as_str() == "phase" => {
            for network_type in vec![NetworkType::Regular, NetworkType::Irregular] {
                let args = cli::ArgsPhase::parse_from(env::args().skip(1));

                for rand_seed in args.seeds {
                    let args = cli::ArgsPhase::parse_from(env::args().skip(1));

                    for eq_steps in args.eq_steps {
                        let args = cli::ArgsPhase::parse_from(env::args().skip(1));

                        let tx_ = tx.clone();
                        let name = format!("{}, {}", network_type.to_string(), rand_seed);

                        children.push(Child::make(&name.to_owned(), move || {
                            run_phase(rand_seed, &args, network_type, eq_steps, -1., tx_, name)
                                .unwrap();
                        }));
                    }
                }
            }

            Ok(simulation_type.to_string())
        }
        
        x => {
            eprintln!("unknown simulation type {:?}", x);
            Err(Box::new(ArgError {}))
        }
    };

    match result {
        Err(e) => Err(e),
        Ok(r) => {
            for r in rx.iter() {
                // clear the screen
                eprint!("\x1B[2J\x1B[1;1H");

                let mut all_done = true;

                for child in children.iter_mut() {
                    child.update(&r);
                    eprint!("[{}]: {}\r\n", child.name, child.msg);

                    all_done = all_done && child.done;
                }

                if all_done {
                    break;
                }
            }

            print!("{} ", r);

            for child in children.iter() {
                print!("{} ", child.msg);
            }

            Ok(())
        }
    }
}
