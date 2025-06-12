use std::fmt::Display;
use strip_ansi_escapes::strip_str;
use unicode_width::UnicodeWidthStr;

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

    pub fn rows(&self) -> &Vec<Vec<T>> {
        &self.rows
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
                let len = strip_str(&cell.to_string()).width();

                if len > widths[i] {
                    widths[i] = len;
                }
            }
        }

        for (i, row) in self.rows.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let cell_str = cell.to_string();

                // For the last column, don't add padding
                if j == row.len() - 1 {
                    write!(f, "{}", cell_str)?;
                } else {
                    // Calculate visual width and required padding
                    let visual_width = strip_str(&cell_str).width();
                    let total_padding = (widths[j] - visual_width) + self.padding;

                    write!(f, "{}{:<padding$}", cell_str, "", padding = total_padding)?;
                }
            }

            // Don't write a newline after the last row
            if i < self.rows.len() - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
