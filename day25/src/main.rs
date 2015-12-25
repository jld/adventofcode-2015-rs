use std::env;

struct KeyGen {
    next: u32
}
impl KeyGen {
    fn new() -> Self { KeyGen { next: 20151125 } }
}
impl Iterator for KeyGen {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        let this = self.next;
        self.next = (((this as u64) * 252533) % 33554393) as u32;
        Some(this)
    }
}

struct Cantor {
    row: usize,
    col: usize,
}
impl Cantor {
    fn new () -> Self { Cantor { row: 1, col: 1 } }
}
impl Iterator for Cantor {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let this = (self.row, self.col);
        if self.row == 1 {
            self.row = self.col + 1;
            self.col = 1;
        } else {
            self.row -= 1;
            self.col += 1;
        }
        Some(this)
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let the_row = args.next().expect("argument #1 is the row").parse().unwrap();
    let the_col = args.next().expect("argument #2 is the column").parse().unwrap();
    let (_, key) = Cantor::new().zip(KeyGen::new()).find(|&((row, col), _)| {
        row == the_row && col == the_col
    }).unwrap();
    println!("{}", key);
}

#[cfg(test)]
mod tests {
    use super::{KeyGen, Cantor};

    #[test]
    fn cantor_example() {
        let mut grid = [[0; 6]; 6];
        for (i, (row, col)) in Cantor::new().enumerate().take(21) {
            grid[row - 1][col - 1] = i + 1;
        }
        assert_eq!(grid, [[1, 3, 6, 10, 15, 21],
                          [2, 5, 9, 14, 20, 0],
                          [4, 8, 13, 19, 0, 0],
                          [7, 12, 18, 0, 0, 0],
                          [11, 17, 0, 0, 0, 0],
                          [16, 0, 0, 0, 0, 0]]);
    }

    #[test]
    fn keygen_example() {
        let mut kg = KeyGen::new();
        assert_eq!(kg.next(), Some(20151125));
        assert_eq!(kg.next(), Some(31916031));
    }

    #[test]
    fn keygen_grid() {
        let mut grid = [[!0; 6]; 6];
        let stream = Cantor::new().zip(KeyGen::new());
        let within = stream.filter(|&((row, col), _)| row <= 6 && col <= 6).take(36);
        for ((row, col), key) in within {
            grid[row - 1][col - 1] = key;
        }
        assert_eq!(grid, [
            [ 20151125,  18749137,  17289845,  30943339,  10071777,  33511524 ],
            [ 31916031,  21629792,  16929656,   7726640,  15514188,   4041754 ],
            [ 16080970,   8057251,   1601130,   7981243,  11661866,  16474243 ],
            [ 24592653,  32451966,  21345942,   9380097,  10600672,  31527494 ],
            [    77061,  17552253,  28094349,   6899651,   9250759,  31663883 ],
            [ 33071741,   6796745,  25397450,  24659492,   1534922,  27995004 ]]);
    }
}
