use std::fmt;
use std::num::ParseFloatError;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
pub struct Interval(pub Duration);

impl FromStr for Interval {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_suffix('s').unwrap_or(s);
        FromStr::from_str(s)
            .map(Duration::from_secs_f64)
            .map(Interval)
    }
}

impl Interval {
    pub const fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}
