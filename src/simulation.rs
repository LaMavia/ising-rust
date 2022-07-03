use std::{error::Error, ops::Div, path::Path, sync::mpsc::Sender, thread, time::Duration};

use csv::Writer;
use rand::{seq::SliceRandom, Rng};
use rand_chacha::ChaCha20Rng;

use crate::{
    child::ChildMsg,
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
    pub n: u128,
    pub spin_sum: i64,
    pub ham: f64,
    pub name: String,
    pub tx: Sender<ChildMsg>,
}

impl Simulation {
    pub fn new(
        size: usize,
        config: SimulationConfig,
        rand: &mut ChaCha20Rng,
        name: String,
        tx: Sender<ChildMsg>,
    ) -> Self {
        Simulation {
            network: Network::new(size, &config.network_type, rand),
            config,
            time: 0,
            spin_sum: 0,
            ham: 0.,
            name,
            tx,
            n: 0
        }
    }

    pub fn mag(&mut self) -> f64 {
        (self.spin_sum as f64) / (self.network.size.pow(2) as f64)
    }

    pub fn calc_h(&mut self) -> f64 {
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

        self.ham = -(self.config.h * h + self.config.j * j);
        self.ham
    }

    pub fn calc_delta_h(&self, p: (usize, usize)) -> f64 {
        let neighbour_spin_sum = self.network.get_neighbours(p).iter().sum::<i8>() as f64;
        let sk = self.network.get_spin(p) as f64;

        sk * 2f64 * (self.config.j * neighbour_spin_sum + self.config.h)
    }

    pub fn calc_magnetisation(&mut self) -> f64 {
        self.spin_sum = self.network.spins.iter().fold(0, |u, s| u + (*s as i64));

        self.mag()
    }

    fn evolve_spin(&mut self, p: (usize, usize), rng: &mut ChaCha20Rng) {
        let delta_h = self.calc_delta_h(p);
        let distortion = rng.gen::<f64>();

        if delta_h <= 0f64 || distortion < (-delta_h / (self.config.kb * self.config.temp)).exp() {
            self.spin_sum += 2 * self.network.flip_spin(p) as i64;
            self.ham += delta_h;
        }
    }

    pub fn mc_iter(&mut self, rng: &mut ChaCha20Rng) {
        let mut indices = (0..self.network.size.pow(2)).collect::<Vec<usize>>();
        indices.shuffle(rng);

        for i in indices {
            self.evolve_spin(pos_of_index(self.network.size, i), rng)
        }
    }

    pub fn snapshot_hysteresis(&mut self) -> Result<Vec<f64>, Box<dyn Error>> {
        let h = self.config.h;
        let m = self.mag();

        self.tx.send(ChildMsg::make(
            self.name.to_owned(),
            format!(
                "H: {}, M: {}, deg_MSE: {}, deg_avg: {}, t: {}",
                h, m, self.network.deg_mse, self.network.deg_avg, self.time
            ),
            false
        ))?;

        Ok(vec![h, m])
    }

    pub fn snapshot_phase(&mut self) -> Result<Vec<f64>, Box<dyn Error>> {
        let temp = self.config.temp;
        let m = self.mag();

        self.tx.send(ChildMsg::make(
            self.name.to_owned(),
            format!(
                "T: {}, M: {}, deg_MSE: {}, deg_avg: {}, t: {}, n: {}",
                temp, m, self.network.deg_mse, self.network.deg_avg, self.time, self.n
            ),
            false,
        ))?;

        thread::sleep(Duration::from_nanos(1));

        Ok(vec![self.time as f64, self.n as f64, temp, m, self.ham])
    }

    pub fn snapshot_relaxation(&mut self, h_prev: f64) -> Result<Vec<f64>, Box<dyn Error>> {
        let temp = self.config.temp;
        let m = self.mag();
        let eta = self.measure_equilibrium(h_prev, self.ham);

        eprintln!(
            "T: {}, M: {}, deg_MSE: {}, deg_avg: {}, η: {}",
            temp, m, self.network.deg_mse, self.network.deg_avg, eta
        );

        Ok(vec![self.config.temp, self.time as f64, eta])
    }

    pub fn measure_equilibrium(&self, h_prev: f64, h_new: f64) -> f64 {
        (h_new - h_prev).div(h_prev).abs()
    }

    pub fn is_at_equilibrium(&self, eq_measure: f64) -> bool {
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
        data_writer.write_record(&["T", "t", "η"])?;
        data_writer.flush()?;

        for x in 0..self.network.size {
            for y in 0..self.network.size {
                self.network.spins[(x, y)] = 1;
            }
        }

        self.calc_h();
        self.calc_magnetisation();

        while self.config.temp <= config.t_max {
            self.calc_h();
            self.calc_magnetisation();

            self.time = 0;

            for _ in 0..self.config.equilibrium_steps {
                self.time += 1;

                let h = self.ham;

                self.mc_iter(rand);

                data_writer.serialize(self.snapshot_relaxation(h)?)?;
            }

            self.config.temp += config.t_step;
            // self.regress(rand);
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

        self.calc_h();
        self.calc_magnetisation();

        let mut saw_max = false;
        let mut step_direction = 1f64;

        let precision = 1e9f64;

        let mut h = self.calc_h();

        loop {
            self.mc_iter(rand);

            let h_new = self.calc_h();

            if self.is_at_equilibrium(self.measure_equilibrium(h, h_new)) {
                break;
            }

            h = h_new;
        }

        while !(self.config.h >= config.h_max && saw_max) {
            let is_max = self.config.h >= config.h_max || self.config.h <= config.h_min;
            self.time = 0;

            if is_max {
                step_direction *= -1f64;
            }

            saw_max |= is_max;

            // simulate
            let mut h = self.calc_h();

            loop {
                self.mc_iter(rand);
                self.time += 1;

                let h_new = self.calc_h();

                if self.network.deg_avg != 4. {
                    self.network.plot_spins()?;
                }

                if self.is_at_equilibrium(self.measure_equilibrium(h, h_new)) || self.time > (1e8 as u128) {
                    break;
                }

                h = h_new;
            }

            // save
            data_writer.serialize(self.snapshot_hysteresis()?)?;

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
        data_writer.write_record(&["t", "n", "T", "M", "E"])?;
        data_writer.flush()?;

        for x in 0..self.network.size {
            for y in 0..self.network.size {
                self.network.spins[(x, y)] = 1;
            }
        }

        self.calc_h();
        self.calc_magnetisation();

        while self.mag() >= 0. {
            // simulate
            let mut h_prev = self.ham;
            self.n = 0;
            loop {
                self.time += 1;
                self.n += 1;
                self.mc_iter(rand);

                let h_new = self.ham;

                let m = self.measure_equilibrium(h_prev, h_new);

                if self.is_at_equilibrium(m) || self.time >= (1e8 as u128) {
                    break;
                }

                h_prev = h_new;
            }

            // save
            data_writer.serialize(self.snapshot_phase()?)?;

            // step
            self.config.temp += config.t_step;
        }

        data_writer.flush()?;

        Ok(())
    }
}
