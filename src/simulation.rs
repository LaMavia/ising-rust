use std::{error::Error, path::Path};

use csv::Writer;
use rand::Rng;

use crate::network::{Network, NetworkType};

#[derive(Debug)]
pub struct SimulationConfig {
    pub temp: f64,
    pub h: f64,
    pub j: f64,
    pub kb: f64,

    pub equilibrium_steps: usize,
    pub network_type: NetworkType,
}

#[derive(Default, Debug)]
pub struct HysteresisConfig {
    pub h_min: f64,
    pub h_max: f64,
    pub h_step: f64,
}

#[derive(Debug)]
pub struct Simulation {
    network: Network,
    size: usize,
    config: SimulationConfig,
}

impl Simulation {
    pub fn new(size: usize, config: SimulationConfig) -> Self {
        Simulation {
            network: Network::new(size, &config.network_type),
            size,
            config,
        }
    }

    pub fn calc_delta_h(&self, (x, y): (usize, usize)) -> f64 {
        let neighbour_spin_sum = self.network.get_neighbours((x, y)).iter().sum::<i8>() as f64;
        let sk = self.network.get_spin((x, y)) as f64;

        sk * (self.config.j * neighbour_spin_sum + 2f64 * self.config.h)
    }

    pub fn calc_magnetisation(&self) -> f64 {
        self.network.spins.iter().fold(0f64, |u, s| u + (*s as f64))
            / (self.network.size.pow(2) as f64)
    }

    fn evolve_spin(&mut self, (x, y): (usize, usize)) {
        let delta_h = self.calc_delta_h((x, y));

        if delta_h <= 0f64
            || rand::random::<f64>() < (-delta_h / (self.config.kb * self.config.temp)).exp()
        {
            // println!("flipping ({x}, {y})!");
            self.network.flip_spin((x, y))
        }
    }

    pub fn mc_step(&mut self) {
        let x = rand::thread_rng().gen_range(0..self.network.size);
        let y = rand::thread_rng().gen_range(0..self.network.size);

        self.evolve_spin((x, y));
    }

    pub fn mc_iter(&mut self) {
        for _ in 0..(self.network.size.pow(2)) {
            self.mc_step()
        }
    }

    pub fn snapshot_hysteresis(&self) -> Vec<f64> {
        let h = self.config.h;
        let m = self.calc_magnetisation();

        eprintln!("H: {}, M: {}, deg_MSE: {}, deg_avg: {}", h, m, self.network.get_deg_mse(4f64), self.network.get_avg_deg());

        vec![h, m]
    }

    pub fn simulate_hysteresis(
        &mut self,
        data_dist_path: &Path,
        config: HysteresisConfig,
    ) -> Result<(), Box<dyn Error>> {
        let mut data_writer = Writer::from_path(data_dist_path)?;
        // Write header
        data_writer.write_record(&["H", "M"])?;
        data_writer.flush()?;

        let mut saw_max = false;
        let mut step_direction = 1f64;

        let precision = 1e6f64;

        while !(self.config.h >= config.h_max && saw_max) {
            let is_max = self.config.h >= config.h_max || self.config.h <= config.h_min;

            if is_max {
                step_direction *= -1f64;
            }

            saw_max |= is_max;

            // simulate
            for _ in 0..self.config.equilibrium_steps {
                self.mc_iter();
            }

            self.network.plot_spins()?;

            // save
            data_writer.serialize(self.snapshot_hysteresis())?;
            data_writer.flush()?;

            // step
            self.config.h =
                ((self.config.h + step_direction * config.h_step) * precision).floor() / precision;
        }

        Ok(())
    }
}
