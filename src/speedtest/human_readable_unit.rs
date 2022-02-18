use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct HumanReadableUnit(u64);

impl fmt::Display for HumanReadableUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num = self.0 as f32;

        let (num, unit) = match num.log10() as u64 {
            0..=2 => (num, 'B'),
            3..=5 => (num / 1_000.0, 'K'),
            6..=8 => (num / 1_000_000.0, 'M'),
            _ => (num / 1_000_000_000.0, 'G'),
        };

        write!(f, "{num}{unit}")
    }
}

impl HumanReadableUnit {
    pub fn new(bytes: u64) -> Self {
        Self(bytes)
    }
}
