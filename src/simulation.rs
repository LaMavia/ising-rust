use std::{error::Error, ops::Div, path::Path};

use csv::Writer;
use rand::{seq::SliceRandom, Rng};
use rand_chacha::ChaCha20Rng;

use crate::{
    matrix::pos_of_index,
    network::{Network, NetworkType},
};

#[derive(Debug)]
pub struct SimulationConfig {
    pub temp: f64,
    pub h: f64,
    pub j: f64,
    pub kb: f64,

    pub equilibrium_steps: usize,
    pub network_type: NetworkType,
    pub eq_threshold: f64,
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
    pub s0: f64,
}

#[derive(Debug)]
pub struct Simulation {
    pub network: Network,
    pub config: SimulationConfig,
    pub time: u128,
    pub mag: f64,
}

impl Simulation {
    pub fn new(size: usize, config: SimulationConfig, rand: &mut ChaCha20Rng) -> Self {
        Simulation {
            network: Network::new(size, &config.network_type, rand),
            config,
            time: 0,
            mag: 0.,
        }
    }

    pub fn calc_h(&self) -> f64 {
        let j: f64 = self
            .network
            .spins
            .enumerator()
            .map(|(p, s)| {
                let neighbour_spin_sum = self.network.get_neighbours(p).iter().sum::<i8>() as f64;

                (*s as f64) * neighbour_spin_sum
            })
            .sum();

        let h = self.network.spins.iter().sum::<i8>() as f64;

        -(self.config.h * h + self.config.j * j)
    }

    pub fn calc_delta_h(&self, p: (usize, usize)) -> f64 {
        let neighbour_spin_sum = self.network.get_neighbours(p).iter().sum::<i8>() as f64;
        let sk = self.network.get_spin(p) as f64;

        sk * 2f64 * (self.config.j * neighbour_spin_sum + self.config.h)
    }

    pub fn calc_magnetisation(&mut self) -> f64 {
        self.mag = self.network.spins.iter().fold(0f64, |u, s| u + (*s as f64))
            / ((self.network.size * self.network.size) as f64);

        self.mag
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
        let mut indices = (0..self.network.size.pow(2)).collect::<Vec<usize>>();
        indices.shuffle(rng);

        for i in indices {
            self.evolve_spin(pos_of_index(self.network.size, i), rng)
        }

        // self.mc_step_chb(rng)
    }

    pub fn snapshot_hysteresis(&mut self) -> Vec<f64> {
        let h = self.config.h;
        let m = self.calc_magnetisation();

        eprintln!(
            "H: {}, M: {}, deg_MSE: {}, deg_avg: {}",
            h, m, self.network.deg_mse, self.network.deg_avg
        );

        vec![h, m]
    }

    pub fn snapshot_phase(&mut self) -> Vec<f64> {
        let temp = self.config.temp;
        let m = self.calc_magnetisation();

        eprintln!(
            "T: {}, M: {}, deg_MSE: {}, deg_avg: {}",
            temp, m, self.network.deg_mse, self.network.deg_avg
        );

        vec![self.time as f64, temp, m]
    }

    pub fn snapshot_relaxation(&mut self) -> Vec<f64> {
        let temp = self.config.temp;
        let m = self.calc_magnetisation();

        eprintln!(
            "T: {}, M: {}, deg_MSE: {}, deg_avg: {}",
            temp, m, self.network.deg_mse, self.network.deg_avg
        );

        vec![self.config.temp, self.time as f64, self.calc_h()]
    }

    pub fn measure_equilibrium(&self, h_prev: f64, h_new: f64) -> f64 {
        (h_new - h_prev).div(h_prev).abs()
    }

    pub fn is_at_equilibrium(&self, eq_measure: f64) -> bool {
        // eprintln!("{}", eq_measure);
        eq_measure.abs() < self.config.eq_threshold
    }

    fn regress(&mut self, rand: &mut ChaCha20Rng) {
        for i in 0..self.network.size.pow(2) {
            self.network.spins[i] = if rand.gen_bool(0.5) { 1 } else { -1 };
        }
    }

    pub fn simulate_relaxation(
        &mut self,
        data_dist_path: &Path,
        config: PhaseConfig,
        rand: &mut ChaCha20Rng,
    ) -> Result<(), Box<dyn Error>> {
        let mut data_writer = Writer::from_path(data_dist_path)?;
        // Write header
        data_writer.write_record(&["T", "t", "E"])?;
        data_writer.flush()?;

        let precision = 1e5;

        let mut h_prev = self.calc_h();

        while self.config.temp <= config.t_max {
            self.time = 0;

            for _ in 0..self.config.equilibrium_steps {
                self.time += 1;
                self.mc_iter(rand);

                data_writer.serialize(self.snapshot_relaxation())?;
            }

            self.config.temp = ((self.config.temp + config.t_step) * precision).floor() / precision;
        }

        data_writer.flush()?;

        Ok(())
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

        let precision = 1e9f64;

        let mut h = self.calc_h();

        loop {
            self.mc_iter(rand);

            let h_new = self.calc_h();

            if self.network.deg_avg != 4. {
                self.network.plot_spins()?;
                eprintln!("{}", self.measure_equilibrium(h, h_new));
            }

            if self.is_at_equilibrium(self.measure_equilibrium(h, h_new)) {
                break;
            }

            h = h_new;
        }

        while !(self.config.h >= config.h_max && saw_max) {
            let is_max = self.config.h >= config.h_max || self.config.h <= config.h_min;

            if is_max {
                step_direction *= -1f64;
            }

            saw_max |= is_max;

            // simulate
            let mut h = self.calc_h();

            loop {
                self.mc_iter(rand);

                let h_new = self.calc_h();

                if self.network.deg_avg != 4. {
                    self.network.plot_spins()?;
                    // eprintln!("{}", self.measure_equilibrium(h, h_new));
                }

                if self.is_at_equilibrium(self.measure_equilibrium(h, h_new)) {
                    break;
                }

                h = h_new;
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
        data_writer.write_record(&["t", "T", "M"])?;
        data_writer.flush()?;

        let precision = 1e8;

        for x in 0..self.network.size {
            for y in 0..self.network.size {
                self.network.spins[(x, y)] = 1;
            }
        }

        while self.mag >= 0. {
            // simulate
            let mut h_prev = self.calc_h();
            loop {
                self.mc_iter(rand);

                let h_new = self.calc_h();

                let m = self.measure_equilibrium(h_prev, h_new);

                if self.is_at_equilibrium(m) {
                    break;
                }

                h_prev = h_new;
            }

            // save
            data_writer.serialize(self.snapshot_phase())?;

            // step
            self.config.temp = ((self.config.temp + config.t_step) * precision).floor() / precision;
        }

        data_writer.flush()?;

        Ok(())
    }
}
