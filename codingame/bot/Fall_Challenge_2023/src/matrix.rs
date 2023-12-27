use crate::Float;
use std::ops::Add;

// ANSWER START
pub struct Matrix {
    pub row: usize,
    pub col: usize,
    pub data: Vec<Float>,
}

impl<'a, 'b> Add<&'b Matrix> for &'a Matrix {
    type Output = Matrix;

    fn add(self, rhs: &'b Matrix) -> Self::Output {
        assert_eq!(self.data.len(), self.row * self.col);
        assert_eq!(rhs.data.len(), rhs.row * rhs.col);

        let mut data = Vec::with_capacity(self.data.len());

        for (a, b) in self.data.iter().zip(rhs.data.iter()) {
            data.push(a + b);
        }

        Matrix {
            row: self.row,
            col: self.col,
            data,
        }
    }
}

impl Matrix {
    pub fn dot(&self, other: &Matrix) -> Matrix {
        assert_eq!(self.col, other.row);

        let mut data = Vec::with_capacity(self.row * other.col);

        for x in 0..self.row {
            for y in 0..other.col {
                let mut sum = 0.0;

                for z in 0..self.col {
                    sum += self.data[x * self.col + z] * other.data[z * other.col + y];
                }

                data.push(sum);
            }
        }

        Matrix {
            row: self.row,
            col: other.col,
            data,
        }
    }
}
// ANSWER END

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let a = Matrix {
            row: 2,
            col: 2,
            data: vec![1.0, 2.0, 3.0, 4.0],
        };
        let b = Matrix {
            row: 2,
            col: 2,
            data: vec![5.0, 6.0, 7.0, 8.0],
        };

        let c = &a + &b;
        assert_eq!(c.data, vec![6.0, 8.0, 10.0, 12.0]);

        let d = &a + &c;
        assert_eq!(d.data, vec![7.0, 10.0, 13.0, 16.0]);
    }

    #[test]
    fn test_dot() {
        let a = Matrix {
            row: 2,
            col: 3,
            data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        };
        let b = Matrix {
            row: 3,
            col: 2,
            data: vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
        };

        let c = a.dot(&b);
        assert_eq!(c.data, vec![58.0, 64.0, 139.0, 154.0]);
    }
}
