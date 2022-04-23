use std::{error::Error, io, str::FromStr};

use crate::matrix::{index_of_pos, Matrix};
use plotters::prelude::*;
use rand::prelude::*;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug)]
pub struct Network {
    pub size: usize,
    pub spins: Matrix<i8>,
    pub lattice: Matrix<Vec<usize>>,
}

impl Network {
    fn make_spins(size: usize) -> Matrix<i8> {
        Matrix::new(size, size, |_| {
            if thread_rng().gen_bool(0.5f64) {
                1
            } else {
                -1
            }
        })
    }

    fn make_lattice_regular(size: usize) -> Matrix<Vec<usize>> {
        Matrix::new(size, size, |((size, _), (x, y))| {
            let x = x as i64;
            let y = y as i64;

            vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
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
                .collect()
        })
    }

    fn make_lattice_irregular(size: usize) -> Matrix<Vec<usize>> {
        Matrix::new(size, size, |((size, _), (x, y))| {
            let x = x as i64;
            let y = y as i64;

            vec![
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
            .filter(|_| thread_rng().gen_bool(0.5f64))
            .map(|(x, y)| {
                index_of_pos(
                    size,
                    (
                        x.rem_euclid(size as i64) as usize,
                        y.rem_euclid(size as i64) as usize,
                    ),
                )
            })
            .collect()
        })
    }

    pub fn new(size: usize, network_type: &NetworkType) -> Self {
        Network {
            size,
            spins: Network::make_spins(size),
            lattice: match network_type {
                NetworkType::Regular => Network::make_lattice_regular(size),
                NetworkType::Irregular => Network::make_lattice_irregular(size),
            },
        }
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

    pub fn flip_spin(&mut self, (x, y): (usize, usize)) {
        self.spins[(x, y)] *= -1
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
