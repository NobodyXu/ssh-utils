use super::{logined, PingArgs, SshSessionBuilder};

use clap_verbosity_flag::Verbosity;
use openssh::Error;
use std::io;
use std::time::{Duration, Instant};
use tokio::signal::ctrl_c;
use tokio::time::{interval, MissedTickBehavior};

pub async fn main_loop(
    args: PingArgs,
    verbose: Verbosity,
    builder: SshSessionBuilder<'_>,
    stats: &mut Vec<Duration>,
) -> Result<(), Error> {
    let mut interval = interval(args.interval.0);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let shutdown_requested = ctrl_c();
    tokio::pin!(shutdown_requested);

    for seq in 0..args.count {
        interval.tick().await;

        let instant = Instant::now();
        let res = tokio::select! {
            res = builder.connect() => res,
            _ = &mut shutdown_requested => {
                return Ok(())
            },
        };
        let elapsed = instant.elapsed();

        match res {
            Ok(session) => {
                println!("Accessible: seq = {seq}, time = {elapsed:#?}");
                stats.push(elapsed);

                return logined::main_loop(args, verbose, session, stats).await;
            }
            Err(error) => match error {
                Error::Connect(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                    println!("Accessible: seq = {seq}, time = {elapsed:#?}");
                    stats.push(elapsed);
                }
                error => return Err(error),
            },
        };
    }

    Ok(())
}
