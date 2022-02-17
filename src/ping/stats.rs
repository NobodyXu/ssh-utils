use num_integer::Roots;
use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub struct Stats {
    min: Duration,
    max: Duration,
    avg: Duration,
    sd: Duration,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rtt min/avg/max/stdev = {:#?}/{:#?}/{:#?}/{:#?}",
            self.min, self.avg, self.max, self.sd
        )
    }
}

fn from_micros(micros: u128) -> Duration {
    Duration::from_micros(micros.try_into().unwrap_or(u64::MAX))
}

impl Stats {
    pub fn new(elapseds: &[Duration]) -> Option<Self> {
        if elapseds.is_empty() {
            return None;
        }

        let len = elapseds.len() as u128;

        // Convert duration to micros to avoid overflow.
        //
        // Most networks has millisecond level latency,
        // so using microseconds here is more than enough.
        let iter = elapseds.iter().map(Duration::as_micros);

        let min = iter.clone().min().map(from_micros).unwrap();
        let max = iter.clone().max().map(from_micros).unwrap();

        let avg = iter.clone().sum::<u128>() / len;

        let sum: u128 = iter
            .clone()
            .map(|micros| {
                let diff = if micros >= avg {
                    micros - avg
                } else {
                    avg - micros
                };

                // It is extremely unlikely for u128 to be overflown
                diff.pow(2)
            })
            .sum();
        let variance = sum / len;

        let sd = from_micros(variance.sqrt());

        Some(Self {
            min,
            max,
            sd,
            avg: from_micros(avg),
        })
    }
}
