use std::error::Error;

use crate::matrix::{index_of_pos, Matrix};
use plotters::{prelude::*, style::RGBAColor};
use rand::prelude::*;

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

    pub fn new(size: usize) -> Self {
        Network {
            size,
            spins: Network::make_spins(size),
            lattice: Network::make_lattice_regular(size),
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

    pub fn plot_spins(&self) -> Result<(), Box<dyn Error>> {
        let root = BitMapBackend::new("out.png", (1000, 1000));
        let root_area = root.into_drawing_area();

        root_area.fill(&WHITE)?;

        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .caption("Bar Demo", ("sans-serif", 40))
            .build_cartesian_2d(0..self.size, 0..self.size)?;

        ctx.configure_mesh().draw()?;

        ctx.draw_series(
            self.spins
                .enumerator()
                .filter(|(_, s)| **s == 1)
                .map(|(p, _)| {
                    Circle::new(
                        p,
                        8,
                        BLACK.filled(),
                    )
                }),
        )?;

        Ok(())
    }
}
