mod logined;

use super::{Interval, SshSessionBuilder};

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use openssh::Error;
use std::io;
use std::num::NonZeroU64;

#[derive(Debug, Parser)]
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

async fn main_loop_no_login(
    _args: PingArgs,
    _verbose: Verbosity,
    _builder: SshSessionBuilder<'_>,
) -> Result<(), Error> {
    todo!()
}

pub async fn run(
    args: PingArgs,
    verbose: Verbosity,
    builder: SshSessionBuilder<'_>,
) -> Result<(), Error> {
    let res = builder.connect().await;

    match res {
        Ok(session) => logined::main_loop(args, verbose, session).await,
        Err(error) => match error {
            Error::Connect(err) if err.kind() == io::ErrorKind::ConnectionRefused => {
                main_loop_no_login(args, verbose, builder).await
            }
            error => Err(error),
        },
    }
}
