use std::fmt::Display;

/// Represents a size in bytes
/// Can be used to display sizes of files
/// and folders in the most appropriate unit.
pub struct Size(pub u64);

impl Size {
    const UNITS: [&'static str; 5] = ["B", "KB", "MB", "GB", "TB"];
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut i = 0;
        let mut size = self.0 as f32;

        while size >= 1024.0 && i < Size::UNITS.len() - 1 {
            size /= 1024.0;
            i += 1;
        }

        let decimal = ((size * 100.0) as u64) % 100;
        let precison = f.precision().unwrap_or(if decimal > 0 { 1 } else { 0 });

        write!(f, "{:.*} {}", precison, size, Size::UNITS[i])
    }
}
