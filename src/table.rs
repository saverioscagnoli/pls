use std::fmt::Display;

pub struct Table<T: Display> {
    // Matrix of rows, made of vectors of type T
    // T must be printable (Display trait)
    rows: Vec<Vec<T>>,

    // Minimum space between columns
    padding: usize,
}

impl<T: Display> Table<T> {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            padding: 1,
        }
    }

    pub fn add_row<R: Into<Vec<T>>>(&mut self, row: R) {
        let row: Vec<T> = row.into();

        if row.is_empty() {
            return;
        }

        self.rows.push(row);
    }

    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }
}

impl<T: Display> Display for Table<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.rows.is_empty() {
            return write!(f, "No rows");
        }

        #[rustfmt::skip]
        let widths_length = self.rows.iter()
            .map(|row| row.len())
            .max()
            .unwrap_or(0);

        let mut widths = vec![0; widths_length];

        for row in self.rows.iter() {
            for (i, cell) in row.iter().enumerate() {
                let len = cell.to_string().len();

                if len > widths[i] {
                    widths[i] = len;
                }
            }
        }

        for row in self.rows.iter() {
            for (i, cell) in row.iter().enumerate() {
                let cell = cell.to_string();

                write!(
                    f,
                    "{:<width$}{:<padding$}",
                    cell,
                    "",
                    width = widths[i],
                    padding = self.padding
                )?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
