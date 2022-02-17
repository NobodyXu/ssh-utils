use super::utility::BorrowCell;
use super::{eprintln_error, Interval, SshSessionBuilder};

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use openssh::{Error, Session, Stdio};
use std::collections::HashMap;
use std::io;
use std::num::NonZeroU64;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{interval, MissedTickBehavior};

#[derive(Debug, Parser)]
pub struct PingArgs {
    /// Interval of pinging in seconds (can be float).
    #[clap(short, long, default_value_t = Interval::from_secs(1))]
    interval: Interval,

    /// Number of packets to sent.
    #[clap(short, long, default_value_t = u64::MAX)]
    count: u64,

    /// Size of the packet.
    #[clap(short, long, default_value_t = NonZeroU64::new(256).unwrap())]
    size: NonZeroU64,
}

async fn main_loop_logined(
    args: PingArgs,
    verbose: Verbosity,
    session: Session,
) -> Result<(), Error> {
    let len = 8 + args.size.get();

    let mut interval = interval(args.interval.0);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let mut child = session
        .command("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .await?;

    let mut stdin = child.stdin().take().unwrap();
    let mut stdout = child.stdout().take().unwrap();

    let hashmap = BorrowCell::new(HashMap::with_capacity(10));

    // Writing and reading from the remote child is done in parallel since there
    // is no guarantee on whether the buffer is flushed for every new line.
    tokio::try_join!(
        async {
            let mut buffer: Vec<u8> = (0..8 + args.size.get())
                .map(|n| (n % (u8::MAX as u64)).try_into().unwrap())
                .collect();

            *buffer.last_mut().unwrap() = b'\n';

            for seq in 0..args.count {
                let seq_buffer: &mut [u8; 8] = (&mut buffer[..8]).try_into().unwrap();
                seq_buffer.copy_from_slice(&seq.to_be_bytes());

                interval.tick().await;

                hashmap.borrow().insert(seq, Instant::now());

                stdin.write_all(&buffer).await.map_err(Error::ChildIo)?;
            }

            Ok::<_, Error>(())
        },
        async {
            let mut buffer: Vec<u8> = (0..8 + args.size.get()).map(|_n| 0).collect();

            for _ in 0..args.count {
                stdout
                    .read_exact(&mut buffer)
                    .await
                    .map_err(Error::ChildIo)?;

                let seq_buffer: &mut [u8; 8] = (&mut buffer[..8]).try_into().unwrap();
                let seq = u64::from_be_bytes(*seq_buffer);

                if let Some(instant) = hashmap.borrow().remove(&seq) {
                    let elapsed = instant.elapsed();
                    println!("Logined: seq = {seq}, time = {elapsed:#?}");
                } else {
                    eprintln_error!("Unexpected packet: seq = {seq}");
                }
            }

            Ok::<_, Error>(())
        },
    )?;

    let exit_status = child.wait().await?;

    if !exit_status.success() {
        eprintln_error!("Failed to execute cut on remote: {exit_status:#?}");
    }

    session.close().await
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
        Ok(session) => main_loop_logined(args, verbose, session).await,
        Err(error) => match error {
            Error::Connect(err) if err.kind() == io::ErrorKind::ConnectionRefused => {
                main_loop_no_login(args, verbose, builder).await
            }
            error => Err(error),
        },
    }
}
