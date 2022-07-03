use std::{collections::HashMap, error::Error, hash::Hash, io, iter::Map, str::FromStr};

use crate::matrix::{index_of_pos, Matrix};
use plotters::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum NetworkType {
    Regular,
    Irregular,
}

impl From<&str> for NetworkType {
    fn from(s: &str) -> Self {
        match s {
            "regular" | "r" | "reg" => NetworkType::Regular,
            "irregular" | "ir" | "irreg" => NetworkType::Irregular,
            _ => NetworkType::Regular,
        }
    }
}

impl FromStr for NetworkType {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NetworkType::from(s))
    }
}

impl ToString for NetworkType {
    fn to_string(&self) -> String {
        match self {
            NetworkType::Irregular => String::from("irregular"),
            NetworkType::Regular => String::from("regular"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Network {
    pub size: usize,
    pub spins: Matrix<i8>,
    pub lattice: Matrix<Vec<usize>>,
    pub deg_mse: f64,
    pub deg_avg: f64,
}

impl Network {
    fn make_spins(size: usize, rand: &mut ChaCha20Rng) -> Matrix<i8> {
        let mut m = Matrix::new(size, size, |_| 1);

        for x in 0..size {
            for y in 0..size {
                m[(x, y)] = if rand.gen_bool(0.5) { 1 } else { -1 };
            }
        }

        m
    }

    fn make_lattice_regular(size: usize, rng: &mut ChaCha20Rng) -> Matrix<Vec<usize>> {
        let mut m = Matrix::new(size, size, |_| vec![]);

        for ix in 0..size {
            for iy in 0..size {
                for _ in 0..8 {
                    rng.gen::<i64>();
                }

                let x = ix as i64;
                let y = iy as i64;

                m[(ix, iy)] = vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                    .into_iter()
                    .map(|(x, y)| {
                        index_of_pos(
                            size,
                            (
                                x.rem_euclid(size as i64) as usize,
                                y.rem_euclid(size as i64) as usize,
                            ),
                        )
                    })
                    .collect();
            }
        }

        m
    }

    fn make_lattice_irregular(size: usize, rng: &mut ChaCha20Rng) -> Matrix<Vec<usize>> {
        let mut m_conn = Matrix::new(size, size, |((width, _), (xi, yi))| {
            let mut map = HashMap::new() as HashMap<usize, bool>;

            let x = xi as i64;
            let y = yi as i64;

            for (pos_x, pos_y) in vec![
                (x - 1, y),
                (x + 1, y),
                (x, y - 1),
                (x, y + 1),
                (x - 1, y - 1),
                (x - 1, y + 1),
                (x + 1, y - 1),
                (x + 1, y + 1),
            ]
            .into_iter()
            {
                map.insert(index_of_pos(
                    width,
                    (
                        pos_x.rem_euclid(width as i64) as usize,
                        pos_y.rem_euclid(width as i64) as usize,
                    ),
                ), false);
            }

            map
        });
        let mut m = Matrix::new(size, size, |_| vec![]);

        for ix in 0..size {
            for iy in 0..size {
                let x = ix as i64;
                let y = iy as i64;

                let i = index_of_pos(size, (ix, iy));

                for j in vec![
                    (x - 1, y),
                    (x + 1, y),
                    (x, y - 1),
                    (x, y + 1),
                    (x - 1, y - 1),
                    (x - 1, y + 1),
                    (x + 1, y - 1),
                    (x + 1, y + 1),
                ]
                .into_iter()
                .map(|(pos_x, pos_y)| {
                    index_of_pos(
                        size,
                        (
                            pos_x.rem_euclid(size as i64) as usize,
                            pos_y.rem_euclid(size as i64) as usize,
                        ),
                    )
                }) {
                    let r = rng.gen_bool(0.5f64);
                    let &tried = m_conn[j].get(&i).unwrap_or(&false);

                    m_conn[j].insert(i, true);
                    m_conn[i].insert(j, true);

                    if tried || r {
                        continue;
                    } else {
                        
                    }

                    m[i].push(j);
                    m[j].push(i);
                }
            }
        }

        m
    }

    pub fn new(size: usize, network_type: &NetworkType, rand: &mut ChaCha20Rng) -> Self {
        let mut m = Network {
            size,
            spins: Network::make_spins(size, rand),
            lattice: match network_type {
                NetworkType::Regular => Network::make_lattice_regular(size, rand),
                NetworkType::Irregular => Network::make_lattice_irregular(size, rand),
            },
            deg_mse: 0f64,
            deg_avg: 0f64,
        };

        m.deg_mse = m.get_deg_mse(4f64);
        m.deg_avg = m.get_avg_deg();

        m
    }

    pub fn get_neighbours(&self, (x, y): (usize, usize)) -> Vec<i8> {
        self.lattice[(x, y)]
            .iter()
            .map(|i: &usize| self.spins[*i])
            .collect()
    }

    pub fn get_spin(&self, (x, y): (usize, usize)) -> i8 {
        self.spins[(x, y)]
    }

    pub fn flip_spin(&mut self, (x, y): (usize, usize)) -> i8 {
        self.spins[(x, y)] *= -1;
        self.spins[(x, y)]
    }

    pub fn get_deg_mse(&self, expected_deg: f64) -> f64 {
        self.lattice.iter().fold(0f64, |u, x| {
            let deg = x.len() as f64;

            u + (expected_deg - deg).powi(2)
        }) / (self.size.pow(2) as f64)
    }

    pub fn get_avg_deg(&self) -> f64 {
        self.lattice.iter().fold(0f64, |u, x| u + x.len() as f64) / (self.size.pow(2) as f64)
    }

    pub fn plot_spins(&self) -> Result<(), Box<dyn Error>> {
        let root = BitMapBackend::new("out.png", (1000, 1000));
        let root_area = root.into_drawing_area();

        root_area.fill(&WHITE)?;

        let mut ctx =
            ChartBuilder::on(&root_area).build_cartesian_2d(0..self.size, 0..self.size)?;

        ctx.configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        ctx.draw_series(self.spins.enumerator().filter(|(_, s)| **s == 1).map(
            |((x, y), _)| Rectangle::new([(x, y), (x + 1, y + 1)], RGBColor(0, 0, 0).filled()), // Circle::new(p, 8, BLACK.filled())
        ))?;

        Ok(())
    }
}
