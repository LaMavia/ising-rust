use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Matrix<T> {
    width: usize,
    xs: Vec<T>,
}

impl<T> Matrix<T> {
    pub fn new(width: usize, height: usize, f: fn(((usize, usize), (usize, usize))) -> T) -> Self {
        Matrix {
            width,
            xs: vec![0; width * height]
                .into_iter()
                .enumerate()
                .map(|(i, _)| f(((width, height), pos_of_index(width, i))))
                .collect(),
        }
    }

    pub fn iter<'a>(&'a self) -> MatrixIterator<'a, T> {
        MatrixIterator::new(self)
    }

    pub fn enumerator<'a>(&'a self) -> MatrixEnumerator<'a, T> {
      MatrixEnumerator::new(self)
    }
}

impl<T> Index<usize> for Matrix<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.xs[index]
    }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self[x + y * self.width]
    }
}

impl<T> IndexMut<usize> for Matrix<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.xs[i]
    }
}

impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.xs[x + y * self.width]
    }
}

pub struct MatrixEnumerator<'a, T> {
  matrix: &'a Matrix<T>,
  i: usize,
}

impl<'a, T> MatrixEnumerator<'a, T> {
  pub fn new(matrix: &'a Matrix<T>) -> Self {
    MatrixEnumerator { matrix, i: 0 }
  }
}

impl<'a, T> Iterator for MatrixEnumerator<'a, T> {
  type Item = ((usize, usize), &'a T);

  fn next(&mut self) -> Option<Self::Item> {
    if self.i >= self.matrix.xs.len() {
      None
  } else {
    let result = (pos_of_index(self.matrix.width, self.i), &self.matrix[self.i]);
    self.i += 1;

    Some(result)
  }
  }
}

pub struct MatrixIterator<'a, T> {
    matrix: &'a Matrix<T>,
    i: usize,
}

impl<'a, T> MatrixIterator<'a, T> {
    pub fn new(matrix: &'a Matrix<T>) -> Self {
        MatrixIterator { matrix, i: 0 }
    }
}

impl<'a, T> Iterator for MatrixIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.matrix.xs.len() {
            None
        } else {
            let result = &self.matrix[self.i];
            self.i += 1;

            Some(result)
        }
    }
}

pub fn index_of_pos(width: usize, (x, y): (usize, usize)) -> usize {
    x + y * width
}

pub fn pos_of_index(width: usize, index: usize) -> (usize, usize) {
    let x = index.rem_euclid(width);
    let y = (index - x) / width;

    (x, y)
}
