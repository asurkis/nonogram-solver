use std::*;

use std::io::Read;
use std::iter::repeat;

struct Nonogram {
    width: usize,
    height: usize,
    input_cols: Vec<Vec<usize>>,
    input_rows: Vec<Vec<usize>>,
    matrix: Vec<Vec<Option<bool>>>,
}

impl Nonogram {
    fn new() -> Self {
        Nonogram {
            width: 0,
            height: 0,
            input_cols: Vec::new(),
            input_rows: Vec::new(),
            matrix: Vec::new(),
        }
    }

    fn read_from(reader: &mut Read) -> io::Result<Self> {
        let mut result = Self::new();
        let mut input_string = String::new();
        reader.read_to_string(&mut input_string)?;
        let numbers: Vec<Vec<usize>> = input_string
            .split('\n')
            .map(|s| s.split_whitespace()
                .map(|word| word.parse())
                .filter(|result| result.is_ok())
                .map(|result| result.unwrap())
                .collect())
            .collect();
        result.width = numbers[0][0];
        result.height = numbers[0][1];
        for i in 0..result.width {
            result.input_cols.push(numbers[i + 1].clone());
        }
        for i in 0..result.height {
            result.input_rows.push(numbers[i + result.width + 1].clone());
        }
        result.matrix.resize(result.width, repeat(None).take(result.height).collect());
        Ok(result)
    }

    fn check_suggestion(line: &Vec<Option<bool>>, input: &Vec<usize>, shifts: &Vec<usize>) -> bool {
        let mut merge: Vec<_> = line.iter().map(|&e| e == Some(true)).collect();
        let mut shift = 0;
        for i in 0..shifts.len() {
            shift += shifts[i];
            for j in 0..input[i] {
                if line[shift + j] == Some(false) {
                    return false;
                }
                merge[shift + j] = true;
            }
            shift += input[i] + 1;
        }

        let mut encode = Vec::new();
        let mut last = 0;
        for i in 0..merge.len() {
            if merge[i] {
                last += 1;
            } else {
                if last != 0 {
                    encode.push(last);
                }
                last = 0;
            }
        }
        if last != 0 {
            encode.push(last);
        }

        encode == *input
    }

    fn suggest(line: &mut Vec<Option<bool>>, input: &Vec<usize>, queue: &mut Vec<usize>) {
        let mut correct_suggests = 0;
        let mut correctness: Vec<_> = repeat(0).take(line.len()).collect();

        let mut shifts: Vec<_> = repeat(0).take(input.len()).collect();
        let mut shifts_left = line.len() - input.iter().fold(0, |a, b| a + b) - input.len() + 1;

        let mut variants_left = true;

        while variants_left {
            // println!("{} {:?}", shifts_left, shifts);
            if Self::check_suggestion(&line, &input, &shifts) {
                correct_suggests += 1;
                let mut shift = 0;
                for i in 0..input.len() {
                    shift += shifts[i];
                    for j in 0..input[i] {
                        correctness[shift + j] += 1;
                    }
                    shift += input[i] + 1;
                }
            }

            if shifts_left > 0 {
                match shifts.pop() {
                    Some(shift) => {
                        shifts.push(shift + 1);
                        shifts_left -= 1;
                    }
                    None => {}
                }
            } else {
                while {
                    match shifts.pop() {
                        Some(0) => true,
                        Some(shift) => {
                            shifts_left += shift;
                            false
                        }
                        None => {
                            variants_left = false;
                            false
                        }
                    }
                } {}
                match shifts.pop() {
                    Some(shift) => {
                        shifts_left -= 1;
                        shifts.push(shift + 1);
                    }
                    None => variants_left = false,
                }

                while variants_left && shifts.len() < input.len() {
                    shifts.push(0);
                }
            }
        }

        for i in 0..line.len() {
            if line[i].is_some() {
                continue;
            }

            if correctness[i] == 0 {
                line[i] = Some(false);
                queue.push(i);
            } else if correctness[i] == correct_suggests {
                line[i] = Some(true);
                queue.push(i);
            }
        }
    }

    fn suggest_row(&mut self, row_id: usize, mut queue_cols: &mut Vec<usize>) {
        let mut line = (0..self.width)
            .map(|i| self.matrix[i][row_id])
            .collect();
        Self::suggest(&mut line, &self.input_rows[row_id], &mut queue_cols);
        for i in 0..self.width {
            self.matrix[i][row_id] = line[i];
        }
    }

    fn suggest_col(&mut self, col_id: usize, mut queue_rows: &mut Vec<usize>) {
        Self::suggest(&mut self.matrix[col_id], &self.input_cols[col_id], &mut queue_rows);
    }

    fn solve(&mut self) {
        let mut queue_cols = Vec::new();
        let mut queue_rows = Vec::new();

        for i in 0..self.width {
            queue_cols.push(i);
        }
        for i in 0..self.height {
            queue_rows.push(i);
        }
        // queue_cols.push(0);

        let mut iteration_number = 0;
        while !queue_cols.is_empty() || !queue_rows.is_empty() {
            println!("iteration #{}", iteration_number);
            if iteration_number % 10 == 0 {
                println!("{:?}", self);
            }
            iteration_number += 1;
            match queue_cols.pop() {
                Some(col) => self.suggest_col(col, &mut queue_rows),
                _ => {}
            }
            match queue_rows.pop() {
                Some(row) => self.suggest_row(row, &mut queue_cols),
                _ => {}
            }
        }
    }
}

impl fmt::Debug for Nonogram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(format!("{}x{}, solved: {} tiles\n",
            self.width, self.height,
            self.matrix.iter()
                .map(|v| v.iter()
                    .filter(|tile| tile.is_some())
                    .count())
                .fold(0, |a, b| a + b)).as_str())?;
        for i in 0..self.height {
            if i % 5 == 0 {
                for j in 0..self.width {
                    if j % 5 == 0 {
                        f.write_str("+")?;
                    }
                    f.write_str("-")?;
                }
                f.write_str("+\n")?;
            }
            for j in 0..self.width {
                if j % 5 == 0 {
                    f.write_str("|")?;
                }
                f.write_str(match self.matrix[j][i] {
                    Some(true) => "*",
                    Some(false) => "x",
                    None => " ",
                })?;
            }
            f.write_str("|\n")?;
        }
        for i in 0..self.width {
            if i % 5 == 0 {
                f.write_str("+")?;
            }
            f.write_str("-")?;
        }
        f.write_str("+\n")?;
        Ok(())
    }
}

fn main() {
    let mut nonogram = Nonogram::read_from(&mut io::stdin()).unwrap();
    nonogram.solve();
    println!("{:?}", nonogram);
}
