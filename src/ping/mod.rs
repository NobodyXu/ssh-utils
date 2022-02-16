mod interval;
use interval::Interval;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct PingArgs {
    /// Interval of pinging in seconds (can be float).
    #[clap(short, long, default_value_t = Interval::DEFAULT_INTERVAL)]
    interval: Interval,

    /// Number of packets to sent.
    #[clap(short, long, default_value_t = usize::MAX)]
    count: usize,

    /// Size of the packet.
    #[clap(short, long, default_value_t = 64)]
    size: usize,

    /// hostname to ping.
    hostname: String,
}
