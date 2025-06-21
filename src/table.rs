use figura::Alignment;
use std::fmt::Display;
use strip_ansi_escapes::strip_str;
use unicode_width::UnicodeWidthStr;

pub struct Table<T: Display> {
    // Matrix of rows, made of vectors of type T
    // T must be printable (Display trait)
    rows: Vec<Vec<(T, Alignment)>>,
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

    pub fn add_row<R: Into<Vec<(T, Alignment)>>>(&mut self, row: R) {
        let row: Vec<(T, Alignment)> = row.into();

        if row.is_empty() {
            return;
        }

        self.rows.push(row);
    }

    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }

    pub fn rows(&self) -> &Vec<Vec<(T, Alignment)>> {
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
            for (i, (cell, _)) in row.iter().enumerate() {
                let len = strip_str(&cell.to_string()).width();
                if len > widths[i] {
                    widths[i] = len;
                }
            }
        }

        for (i, row) in self.rows.iter().enumerate() {
            for (j, (cell, alignment)) in row.iter().enumerate() {
                let cell_str = cell.to_string();
                let visual_width = strip_str(&cell_str).width();
                let available_width = widths[j];
                let is_last_column = j == row.len() - 1;

                match alignment {
                    Alignment::Left => {
                        if is_last_column {
                            // Last column: just write the cell content, no padding
                            write!(f, "{}", cell_str)?;
                        } else {
                            // Regular column: apply left alignment with padding
                            let total_padding = (available_width - visual_width) + self.padding;
                            write!(f, "{}{:<padding$}", cell_str, "", padding = total_padding)?;
                        }
                    }
                    Alignment::Right => {
                        let left_padding = available_width - visual_width;
                        if is_last_column {
                            // Last column: right-align within available width, no trailing padding
                            write!(f, "{:>padding$}{}", "", cell_str, padding = left_padding)?;
                        } else {
                            // Regular column: right-align with inter-column padding
                            write!(
                                f,
                                "{:>padding$}{}{:<padding2$}",
                                "",
                                cell_str,
                                "",
                                padding = left_padding,
                                padding2 = self.padding
                            )?;
                        }
                    }
                    Alignment::Center => {
                        let total_space = available_width - visual_width;
                        let left_padding = total_space / 2;
                        let right_padding = total_space - left_padding;
                        if is_last_column {
                            // Last column: center within available width, no trailing padding
                            write!(
                                f,
                                "{:>padding$}{}{:<padding2$}",
                                "",
                                cell_str,
                                "",
                                padding = left_padding,
                                padding2 = right_padding
                            )?;
                        } else {
                            // Regular column: center with inter-column padding
                            write!(
                                f,
                                "{:>padding$}{}{:<padding2$}",
                                "",
                                cell_str,
                                "",
                                padding = left_padding,
                                padding2 = right_padding + self.padding
                            )?;
                        }
                    }
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
