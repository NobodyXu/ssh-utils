mod login_failed;
mod logined;

mod stats;
use stats::Stats;

use super::{Interval, SshSessionBuilder};

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use openssh::Error;
use std::io;
use std::num::NonZeroU64;

#[derive(Debug, Parser, Copy, Clone)]
pub struct PingArgs {
    /// Interval of pinging in seconds (can be float).
    #[clap(short, long, default_value_t = Interval::from_secs(1))]
    interval: Interval,

    /// Number of packets to sent.
    #[clap(short, long, default_value_t = u64::MAX)]
    count: u64,

    /// Size of the packet.
    #[clap(short, long, default_value_t = NonZeroU64::new(56).unwrap())]
    size: NonZeroU64,
}

pub async fn run(
    args: PingArgs,
    verbose: Verbosity,
    builder: SshSessionBuilder<'_>,
) -> Result<(), Error> {
    let res = builder.connect().await;

    let mut stats = Vec::new();
    let dest = builder.dest();

    let res = match res {
        Ok(session) => logined::main_loop(args, verbose, session, &mut stats).await,
        Err(error) => match error {
            Error::Connect(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                login_failed::main_loop(args, verbose, builder, &mut stats).await
            }
            error => Err(error),
        },
    };

    if let Some(stats) = Stats::new(&stats) {
        println!("--- {dest} ping statistics ---\n{stats}");
    }

    res
}
