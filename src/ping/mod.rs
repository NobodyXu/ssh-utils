use super::{eprintln_error, Interval, SshSessionBuilder};

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use openssh::{Error, Session, Stdio};
use std::io;
use tokio::time;

#[derive(Debug, Parser)]
pub struct PingArgs {
    /// Interval of pinging in seconds (can be float).
    #[clap(short, long, default_value_t = Interval::from_secs(1))]
    interval: Interval,

    /// Number of packets to sent.
    #[clap(short, long, default_value_t = usize::MAX)]
    count: usize,

    /// Size of the packet.
    #[clap(short, long, default_value_t = 56)]
    size: usize,
}

async fn main_loop_logined(
    args: PingArgs,
    verbose: Verbosity,
    session: &Session,
) -> Result<(), Error> {
    let mut buffer: Vec<u8> = (0..8 + args.size)
        .map(|n| (n % (u8::MAX as usize)).try_into().unwrap())
        .collect();

    let len = buffer.len() - 1;
    buffer[len] = b'\n';

    let mut interval = time::interval(args.interval.0);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

    let mut child = session
        .command("cut")
        .arg("-b")
        .arg("-4")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .await?;

    let mut stdin = child.stdin().take().unwrap();
    let mut stdout = child.stdout().take().unwrap();

    for i in 0..args.count {
        {
            let reference: &mut [u8; 8] = (&mut buffer[..8]).try_into().unwrap();
            reference.copy_from_slice(&i.to_be_bytes());
        }

        interval.tick().await;

        todo!()
    }

    let exit_status = child.wait().await?;

    if !exit_status.success() {
        eprintln_error!("Failed to execute cut on remote: {exit_status:#?}");
    }

    Ok(())
}

async fn main_loop_no_login(
    args: PingArgs,
    verbose: Verbosity,
    builder: SshSessionBuilder<'_>,
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
        Ok(session) => {
            main_loop_logined(args, verbose, &session).await?;
            session.close().await
        }
        Err(error) => match error {
            Error::Connect(err) if err.kind() == io::ErrorKind::ConnectionRefused => {
                main_loop_no_login(args, verbose, builder).await
            }
            error => Err(error),
        },
    }
}
