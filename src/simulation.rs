use std::{error::Error, path::Path, sync::mpsc::Sender};

use csv::Writer;
use rand::{seq::SliceRandom, Rng};
use rand_chacha::ChaCha20Rng;

use crate::{
    child::{send, ChildMsg},
    frame,
    mathy::{round_to, SIMULATION_PRECISION},
    matrix::pos_of_index,
    network::{Network, NetworkType},
};

macro_rules! round {
    ($x:expr) => {
        round_to($x, SIMULATION_PRECISION)
    };
    ($target:expr, +, $($value: expr),+) => {
        $target = round_to($target + $( $value )+, SIMULATION_PRECISION)
    }
}

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
    pub ham_internal: f64,
    pub ham_external: f64,
    pub ham_agr_internal: f64,
    pub ham_agr_external: f64,
    pub name: String,
    pub tx: Sender<ChildMsg>,
    pub dist: String,
    pub free_count: i64,
    action_log: Vec<String>,
}

#[derive(Debug)]
pub struct StateSnapshot {
    pub spin_sum: i64,
    pub ham_internal: f64,
    pub ham_external: f64,
    pub ham_agr_internal: f64,
    pub ham_agr_external: f64,
    pub mag: f64,
}

impl StateSnapshot {
    pub fn of_simulation<'a>(simulation: &'a Simulation) -> Self {
        StateSnapshot {
            spin_sum: simulation.spin_sum,
            ham_internal: simulation.ham_internal,
            ham_external: simulation.ham_external,
            ham_agr_internal: simulation.ham_agr_internal,
            ham_agr_external: simulation.ham_agr_external,
            mag: simulation.mag(),
        }
    }

    pub fn ham(&self) -> f64 {
        round_to(self.ham_internal + self.ham_external, SIMULATION_PRECISION)
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
            ham_internal: 0.,
            ham_external: 0.,
            ham_agr_internal: 0.,
            ham_agr_external: 0.,
            name,
            tx,
            n: 0,
            dist,
            free_count: 0,
            action_log: vec![],
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
        round_to(
            (self.spin_sum as f64) / (self.network.size.pow(2) as f64),
            SIMULATION_PRECISION,
        )
    }

    pub fn ham(&self) -> f64 {
        round_to(self.ham_internal + self.ham_external, SIMULATION_PRECISION)
    }

    pub fn ham_agr(&self) -> f64 {
        round_to(
            self.ham_agr_internal + self.ham_agr_external,
            SIMULATION_PRECISION,
        )
    }

    fn calc_h_internal(&self) -> f64 {
        round_to(
            -self.config.j / 2.
                * self
                    .network
                    .spins
                    .enumerator()
                    .map(|(p, &s)| {
                        let neighbour_spin_sum =
                            self.network.get_neighbours(p).iter().sum::<i8>() as i64;

                        (s as i64) * neighbour_spin_sum
                    })
                    .sum::<i64>() as f64,
            SIMULATION_PRECISION,
        )
    }

    fn calc_h_external(&self) -> f64 {
        assert_eq!(
            self.network.spins.iter().count() as i64,
            self.network.size2 as i64
        );

        round_to(
            -self.config.h * self.network.spins.enumerator().map(|(_, &s)| s).sum::<i8>() as f64,
            SIMULATION_PRECISION,
        )
    }

    fn calc_delta_h_internal(&self, p: (usize, usize)) -> f64 {
        let sk = self.network.get_spin(p) as f64;

        round_to(
            sk * 2. * self.config.j * (self.network.get_neighbours(p).iter().sum::<i8>() as f64),
            SIMULATION_PRECISION,
        )
    }

    fn calc_delta_h_external(&mut self, p: (usize, usize)) -> f64 {
        // let sk = self.network.get_spin(p) as f64;

        let prev = StateSnapshot::of_simulation(&self);

        self.network.flip_spin(p);

        let d = round!(round!(self.calc_h_external()) - prev.ham_external);

        self.network.flip_spin(p);

        d
        // round_to(sk * 2. * self.config.h, SIMULATION_PRECISION)
    }

    pub fn calc_delta_h(&mut self, p: (usize, usize)) -> f64 {
        // let prev = StateSnapshot::of_simulation(&self);

        let d_int = self.calc_delta_h_internal(p);
        let d_ext = self.calc_delta_h_external(p);
        let d_ham = round_to(d_int + d_ext, SIMULATION_PRECISION);

        // round!(self.ham_agr_internal, + , self.calc_delta_h_internal(p));

        d_ham
    }

    pub fn calc_magnetisation(&mut self) -> f64 {
        self.spin_sum = self.network.spins.iter().fold(0, |u, &s| u + (s as i64));

        self.mag()
    }

    fn assert_correctness(
        &self,
        prev: &StateSnapshot,
        p: (usize, usize),
        d_int: f64,
        d_ext: f64,
        d_ham: f64,
    ) {
        assert_eq!(
            round!(self.ham_internal - prev.ham_internal),
            d_int,
            "Invalid ham internal {:?} (={}) {:?}; logs: {:?}",
            p,
            self.network.get_spin(p),
            self.network.get_neighbours(p),
            self.action_log
        );
        assert_eq!(
            round!(self.ham_external - prev.ham_external),
            d_ext,
            "Invalid ham external {:?} (={}) {:?}; Σ: {}, Σprev: {}; logs: {:?}",
            p,
            self.network.get_spin(p),
            self.network.get_neighbours(p),
            self.network.spins.iter().sum::<i8>(),
            prev.spin_sum,
            self.action_log
                .iter()
                .fold("".to_string(), |u, l| { format!("{}\n{}", u, l) })
        );
        assert_eq!(
            round!(self.ham() - prev.ham()),
            d_ham,
            "Invalid ham: {:?} (={}) {:?}; logs: {:?}",
            p,
            self.network.get_spin(p),
            self.network.get_neighbours(p),
            self.action_log
        );
    }

    fn evolve_spin(&mut self, p: (usize, usize), rng: &mut ChaCha20Rng) {
        let d_int = self.calc_delta_h_internal(p);
        let d_ext = self.calc_delta_h_external(p);
        let d_ham = self.calc_delta_h(p);
        let distortion = rng.gen::<f64>();
        let v = (-d_ham / (self.config.kb * self.config.temp)).exp();

        let prev = StateSnapshot::of_simulation(&self);

        if d_ham <= 0. || distortion < v {
            self.spin_sum += 2 * self.network.flip_spin(p) as i64;

            self.ham_agr_internal = round_to(self.ham_agr_internal + d_int, SIMULATION_PRECISION);
            self.ham_agr_external = round_to(self.ham_agr_external + d_ext, SIMULATION_PRECISION);

            self.ham_internal = self.calc_h_internal(); // round_to(self.ham_internal + d_int, SIMULATION_PRECISION);
            self.ham_external = self.calc_h_external(); // round_to(self.ham_external + d_ext, SIMULATION_PRECISION);

            self.action_log.push(format!(
                "Flipping {:?} {} -> {} ([ρ/Δ] Ham: {}/{}, Int: {}/{}, Ext: {}/{})",
                p,
                -self.network.get_spin(p),
                self.network.get_spin(p),
                round!(self.ham() - prev.ham()),
                round!(d_ham),
                round!(self.ham_internal - prev.ham_internal),
                round!(d_int),
                round!(self.ham_external - prev.ham_external),
                round!(d_ext)
            ));

            self.assert_correctness(&prev, p.to_owned(), d_int, d_ext, d_ham);
        }
    }

    pub fn mc_iter(&mut self, rng: &mut ChaCha20Rng) {
        let mut indices = (0..self.network.size.pow(2)).collect::<Vec<usize>>();
        indices.shuffle(rng);

        for i in indices {
            self.evolve_spin(pos_of_index(self.network.size, i), rng)
        }
    }

    pub fn snapshot_hysteresis(&self) -> Result<Vec<f64>, Box<dyn Error>> {
        let h = self.config.h;
        let m = self.mag();

        send!(
            self.tx,
            self.name,
            format!(
                "H: {}, M: {}, deg_MSE: {}, deg_avg: {}, t: {}, n: {}, E: {}, αE: {}",
                h,
                m,
                self.network.deg_mse,
                self.network.deg_avg,
                self.time,
                self.n,
                self.ham(),
                self.ham_agr()
            )
        );

        Ok(vec![
            self.time as f64,
            self.n as f64,
            h,
            m,
            self.ham(),
            self.ham_agr(),
        ])
    }

    pub fn snapshot_phase(&self) -> Result<Vec<f64>, Box<dyn Error>> {
        let temp = self.config.temp;
        let m = self.mag();

        send!(
            self.tx,
            self.name,
            format!(
                "T: {}, M: {}, deg_MSE: {}, deg_avg: {}, t: {}, n: {}, E: {}, αE: {}",
                temp,
                m,
                self.network.deg_mse,
                self.network.deg_avg,
                self.time,
                self.n,
                self.ham(),
                self.ham_agr()
            )
        );

        Ok(vec![
            self.time as f64,
            self.n as f64,
            temp,
            m,
            self.ham(),
            self.ham_agr(),
        ])
    }

    pub fn is_at_equilibrium(&self, prev_state: &StateSnapshot) -> bool {
        let d_count = (prev_state.spin_sum - self.spin_sum).abs();
        let d_ham = (prev_state.ham() - self.ham()).abs();

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
        data_writer.write_record(&["t", "n", "H", "M", "E", "aE"])?;
        data_writer.flush()?;

        self.ham_internal = self.calc_h_internal();
        self.ham_agr_internal = self.ham_internal;

        self.ham_external = self.calc_h_external();
        self.ham_agr_external = self.ham_external;

        self.calc_magnetisation();

        let mut saw_max = false;
        let mut step_direction = 1f64;

        let mut prev_time = 0;
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

            self.ham_internal = self.calc_h_internal();
            self.ham_agr_internal = self.ham_internal;

            self.ham_external = self.calc_h_external();
            self.ham_agr_external = self.ham_external;

            self.calc_magnetisation();

            // save
            data_writer.serialize(self.snapshot_hysteresis()?)?;
            // plot frame
            frame!(
                self,
                &format!(
                    "H: {}, t: {} (Δt: {}), M: {}, T: {}, N: {}",
                    self.config.h,
                    self.time,
                    self.time - prev_time,
                    self.mag(),
                    self.config.temp,
                    self.network.size
                )
            );

            prev_time = self.time;

            // step
            self.config.h = round_to(
                self.config.h + round_to(step_direction * config.h_step, SIMULATION_PRECISION),
                SIMULATION_PRECISION,
            );
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
        data_writer.write_record(&["t", "n", "T", "M", "E", "aE"])?;
        data_writer.flush()?;

        for x in 0..self.network.size {
            for y in 0..self.network.size {
                self.network.spins[(x, y)] = 1;
            }
        }

        self.ham_internal = self.calc_h_internal();
        self.ham_agr_internal = self.ham_internal;

        self.ham_external = self.calc_h_external();
        self.ham_agr_external = self.ham_external;

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
                            (prev_state.ham() - self.ham()).abs(),
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
            let out_path = format!("{}/frames/{:016}.png", self.dist, self.time);

            self.network.plot_spins(
                &out_path,
                &format!("T: {}, t: {}", self.config.temp, self.time),
            )?;

            // step
            self.config.temp = round_to(self.config.temp + config.t_step, SIMULATION_PRECISION);
        }

        data_writer.flush()?;

        Ok(())
    }
}
