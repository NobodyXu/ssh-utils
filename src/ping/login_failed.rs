use super::{logined, println_if_not_quiet, println_on_level, Level, PingArgs, SshSessionBuilder};

use clap_verbosity_flag::Verbosity;
use openssh::Error;
use owo_colors::{OwoColorize, Stream::Stdout};
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
                println_on_level!(verbose, Level::Debug, "Ctrl C signal received");
                return Ok(())
            },
        };
        let elapsed = instant.elapsed();

        match res {
            Ok(session) => {
                println_if_not_quiet!(
                    verbose,
                    "{}: seq = {seq}, time = {elapsed:#?}",
                    "Login failed".if_supports_color(Stdout, |text| text.yellow())
                );
                stats.push(elapsed);

                return logined::main_loop(args, verbose, session, stats).await;
            }
            Err(error) => match error {
                Error::Connect(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                    println_if_not_quiet!(
                        verbose,
                        "{}: seq = {seq}, time = {elapsed:#?}",
                        "Login failed".if_supports_color(Stdout, |text| text.yellow())
                    );
                    stats.push(elapsed);
                }
                error => return Err(error),
            },
        };
    }

    Ok(())
}
