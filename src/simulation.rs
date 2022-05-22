use std::{error::Error, path::Path};

use csv::Writer;
use rand::Rng;
use rand_chacha::ChaCha20Rng;

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
pub struct PhaseConfig {
    pub t_min: f64,
    pub t_max: f64,
    pub t_step: f64,
}

#[derive(Debug)]
pub struct Simulation {
    pub network: Network,
    pub config: SimulationConfig,
}

impl Simulation {
    pub fn new(size: usize, config: SimulationConfig, rand: &mut ChaCha20Rng) -> Self {
        Simulation {
            network: Network::new(size, &config.network_type, rand),
            config,
        }
    }

    pub fn calc_delta_h(&self, p: (usize, usize)) -> f64 {
        let neighbour_spin_sum = self.network.get_neighbours(p).iter().sum::<i8>() as f64;
        let sk = self.network.get_spin(p) as f64;

        sk * 2f64 * (self.config.j * neighbour_spin_sum + self.config.h)
    }

    pub fn calc_magnetisation(&self) -> f64 {
        self.network.spins.iter().fold(0f64, |u, s| u + (*s as f64))
            / ((self.network.size * self.network.size) as f64)
    }

    fn evolve_spin(&mut self, p: (usize, usize), rng: &mut ChaCha20Rng) {
        let delta_h = self.calc_delta_h(p);
        let distortion = rng.gen::<f64>();

        if delta_h <= 0f64 || distortion < (-delta_h / (self.config.kb * self.config.temp)).exp() {
            // println!("flipping ({x}, {y})!");
            self.network.flip_spin(p)
        }
    }

    pub fn mc_step(&mut self, rng: &mut ChaCha20Rng) {
        let x = rng.gen_range(0..self.network.size);
        let y = rng.gen_range(0..self.network.size);

        self.evolve_spin((x, y), rng);
    }

    pub fn mc_step_chb(&mut self, rng: &mut ChaCha20Rng) {
        for x in 0..self.network.size {
            for y in 0..self.network.size {
                if x.rem_euclid(2) == y.rem_euclid(2) {
                    self.evolve_spin((x, y), rng);
                }
            }
        }

        for x in 0..self.network.size {
            for y in 0..self.network.size {
                if x.rem_euclid(2) != y.rem_euclid(2) {
                    self.evolve_spin((x, y), rng);
                }
            }
        }
    }

    pub fn mc_iter(&mut self, rng: &mut ChaCha20Rng) {
        for _ in 0..self.network.size.pow(2) {
            self.mc_step(rng)
        } 
    }

    pub fn snapshot_hysteresis(&self) -> Vec<f64> {
        let h = self.config.h;
        let m = self.calc_magnetisation();

        eprintln!(
            "H: {}, M: {}, deg_MSE: {}, deg_avg: {}",
            h, m, self.network.deg_mse, self.network.deg_avg
        );

        vec![h, m]
    }

    pub fn snapshot_phase(&self) -> Vec<f64> {
        let temp = self.config.temp;
        let m = self.calc_magnetisation();

        eprintln!(
            "T: {}, M: {}, deg_MSE: {}, deg_avg: {}",
            temp, m, self.network.deg_mse, self.network.deg_avg
        );

        vec![temp, m]
    }

    pub fn simulate_hysteresis(
        &mut self,
        data_dist_path: &Path,
        config: HysteresisConfig,
        rand: &mut ChaCha20Rng,
    ) -> Result<(), Box<dyn Error>> {
        let mut data_writer = Writer::from_path(data_dist_path)?;
        // Write header
        data_writer.write_record(&["H", "M"])?;
        data_writer.flush()?;

        let mut saw_max = false;
        let mut step_direction = 1f64;

        let precision = 1e4f64;

        while !(self.config.h >= config.h_max && saw_max) {
            let is_max = self.config.h >= config.h_max || self.config.h <= config.h_min;

            if is_max {
                step_direction *= -1f64;
            }

            saw_max |= is_max;

            // simulate
            for _ in 0..self.config.equilibrium_steps {
                self.mc_iter(rand);
            }

            // self.network.plot_spins()?;

            // save
            data_writer.serialize(self.snapshot_hysteresis())?;

            // step
            self.config.h =
                ((self.config.h + step_direction * config.h_step) * precision).floor() / precision;
        }

        data_writer.flush()?;

        Ok(())
    }

    pub fn simulate_phase(
        &mut self,
        data_dist_path: &Path,
        config: PhaseConfig,
        rand: &mut ChaCha20Rng,
    ) -> Result<(), Box<dyn Error>> {
        let mut data_writer = Writer::from_path(data_dist_path)?;
        // Write header
        data_writer.write_record(&["T", "M"])?;
        data_writer.flush()?;

        let e_initial = -config.t_min.log10().ceil() as i32;
        let e_step = -config.t_step.log10().ceil() as i32;
        let precision = 10f64.powi(e_initial.max(e_step));

        eprintln!("in: {e_initial}, step: {e_step}, precision: {precision}");

        for x in 0..self.network.size {
            for y in 0..self.network.size {
                self.network.spins[(x, y)] = 1;
            }
        }

        while self.config.temp <= config.t_max {
            // simulate
            for _ in 0..self.config.equilibrium_steps {
                self.mc_iter(rand);
            }

            // self.network.plot_spins()?;

            // save
            data_writer.serialize(self.snapshot_phase())?;

            // step
            self.config.temp = ((self.config.temp + config.t_step) * precision).floor() / precision;
        }

        data_writer.flush()?;

        Ok(())
    }
}
