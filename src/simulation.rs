use std::{error::Error, ops::Div, path::Path, sync::mpsc::Sender, thread, time::Duration};

use csv::Writer;
use rand::{seq::SliceRandom, Rng};
use rand_chacha::ChaCha20Rng;

use crate::{
    child::{send, ChildMsg},
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
    pub dist: String,
    pub free_count: i64,
}

#[derive(Debug)]
pub struct StateSnapshot {
    pub spin_sum: i64,
    pub ham: f64,
    pub mag: f64,
}

impl StateSnapshot {
    pub fn of_simulation<'a>(simulation: &'a Simulation) -> Self {
        StateSnapshot {
            spin_sum: simulation.spin_sum,
            ham: simulation.ham,
            mag: simulation.mag(),
        }
    }
}

impl Simulation {
    pub fn new(
        size: usize,
        config: SimulationConfig,
        rand: &mut ChaCha20Rng,
        name: String,
        tx: Sender<ChildMsg>,
        dist: String,
    ) -> Self {
        let network = Network::new(size, &config.network_type, rand);
        let mut s = Simulation {
            network,
            config,
            time: 0,
            spin_sum: 0,
            ham: 0.,
            name,
            tx,
            n: 0,
            dist,
            free_count: 0,
        };

        s.free_count = s
            .network
            .lattice
            .enumerator()
            .filter(|(_, ns)| ns.len() == 0)
            .count() as i64;

        s
    }

    pub fn mag(&self) -> f64 {
        (self.spin_sum as f64) / (self.network.size.pow(2) as f64)
    }

    pub fn calc_h(&mut self) -> f64 {
        // eprintln!("start:");
        let j: f64 = self
            .network
            .spins
            .enumerator()
            .map(|(p, &s)| {
                // eprintln!("{:?}", p);
                let neighbour_spin_sum = self.network.get_neighbours(p).iter().sum::<i8>() as f64;

                (s as f64) * neighbour_spin_sum
            })
            .sum();
        // eprintln!("end");

        let h = self.network.spins.iter().sum::<i8>() as f64;

        self.ham = -(self.config.h * h + 0.5 * self.config.j * j);
        self.ham
    }

    pub fn calc_delta_h(&self, p: (usize, usize)) -> f64 {
        let sk = self.network.get_spin(p) as f64;

        sk * 2f64
            * (self.config.j * (self.network.get_neighbours(p).iter().sum::<i8>() as f64)
                + self.config.h)

        // sk * 2f64 * (self.config.j * neighbour_spin_sum + self.config.h)
    }

    pub fn calc_magnetisation(&mut self) -> f64 {
        self.spin_sum = self.network.spins.iter().fold(0, |u, &s| u + (s as i64));

        self.mag()
    }

    fn evolve_spin(&mut self, p: (usize, usize), rng: &mut ChaCha20Rng) {
        let delta_h = self.calc_delta_h(p);
        let distortion = rng.gen::<f64>();
        let v = (-delta_h / (self.config.kb * self.config.temp)).exp();

        if delta_h <= 0f64 || distortion < v {
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
        self.ham = self.calc_h();

        send!(
            self.tx,
            self.name,
            format!(
                "H: {}, M: {}, deg_MSE: {}, deg_avg: {}, t: {}, n: {}, E: {}",
                h, m, self.network.deg_mse, self.network.deg_avg, self.time, self.n, self.ham
            )
        );

        Ok(vec![self.time as f64, self.n as f64, h, m, self.ham])
    }

    pub fn snapshot_phase(&mut self) -> Result<Vec<f64>, Box<dyn Error>> {
        let temp = self.config.temp;
        let m = self.mag();

        send!(
            self.tx,
            self.name,
            format!(
                "T: {}, M: {}, deg_MSE: {}, deg_avg: {}, t: {}, n: {}",
                temp, m, self.network.deg_mse, self.network.deg_avg, self.time, self.n
            )
        );

        thread::sleep(Duration::from_nanos(1));

        Ok(vec![self.time as f64, self.n as f64, temp, m, self.ham])
    }

    pub fn is_at_equilibrium(&self, prev_state: &StateSnapshot) -> bool {
        let d_count = (prev_state.spin_sum - self.spin_sum).abs();
        let d_ham = (prev_state.ham - self.ham).abs();

        let ham_relax = d_ham < f64::MIN_POSITIVE;
        let mag_relax = d_count == 2 * self.free_count
            || (prev_state.mag - self.mag()).abs() <= f64::MIN_POSITIVE;

        if false && self.network.deg_mse != 0. {
            send!(
                self.tx,
                self.name,
                format!("ΔΣs: {}, δΣs: {}", d_count, self.free_count)
            );
        }

        mag_relax && ham_relax
    }

    pub fn simulate_hysteresis(
        &mut self,
        data_dist_path: &Path,
        config: HysteresisConfig,
        rand: &mut ChaCha20Rng,
    ) -> Result<(), Box<dyn Error>> {
        let mut data_writer = Writer::from_path(data_dist_path)?;
        // Write header
        data_writer.write_record(&["t", "n", "H", "M", "E"])?;
        data_writer.flush()?;

        self.calc_h();
        self.calc_magnetisation();

        let mut saw_max = false;
        let mut step_direction = 1f64;

        let precision = 1e9f64;
        self.time = 0;

        loop {
            let prev_state = StateSnapshot::of_simulation(&self);

            self.mc_iter(rand);

            if self.is_at_equilibrium(&prev_state) {
                break;
            }
        }

        while !(self.config.h >= config.h_max && saw_max) {
            let is_max = self.config.h >= config.h_max || self.config.h <= config.h_min;
            self.n = 0;

            if is_max {
                step_direction *= -1f64;
            }

            saw_max |= is_max;

            // simulate
            loop {
                let prev_state = StateSnapshot::of_simulation(&self);

                self.mc_iter(rand);

                self.time += 1;
                self.n += 1;

                if self.is_at_equilibrium(&prev_state) || self.n > (1e8 as u128) {
                    break;
                }
            }

            // save
            data_writer.serialize(self.snapshot_hysteresis()?)?;
            // plot frame
            let out_path = format!("{}/frames/{}.png", self.dist, self.time);

            self.network.plot_spins(
                &out_path,
                &format!("H: {}, t: {}", self.config.h, self.time),
            )?;

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
            self.n = 0;
            loop {
                self.time += 1;
                self.n += 1;

                let prev_state = StateSnapshot::of_simulation(&self);

                self.mc_iter(rand);
                if self.network.deg_mse != 0. && self.time.rem_euclid(201) == 0 {
                    self.network.plot_spins(
                        &format!(
                            "view_size={}_avg={}.png",
                            self.network.size, self.network.deg_avg
                        ),
                        &format!(
                            "T: {}, t: {}, ΔH: {}, ΔM: {}, δM: {}",
                            self.config.temp,
                            self.time,
                            (prev_state.ham - self.ham).abs(),
                            (prev_state.mag - self.mag()).abs(),
                            (self.free_count as f64 / self.network.size2)
                        ),
                    )?;
                }

                if self.is_at_equilibrium(&prev_state) {
                    break;
                }
            }

            // save
            data_writer.serialize(self.snapshot_phase()?)?;

            // plot frame
            let out_path = format!("{}/frames/{}.png", self.dist, self.time);

            self.network.plot_spins(
                &out_path,
                &format!("T: {}, t: {}", self.config.temp, self.time),
            )?;

            // step
            self.config.temp += config.t_step;
        }

        data_writer.flush()?;

        Ok(())
    }
}
