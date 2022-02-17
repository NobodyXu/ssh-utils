use super::PingArgs;
use crate::eprintln_error;
use crate::utility::BorrowCell;

use clap_verbosity_flag::Verbosity;
use openssh::{ChildStdin, ChildStdout, Error, Session, Stdio};
use std::collections::HashMap;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::signal::ctrl_c;
use tokio::time::{interval, MissedTickBehavior};

async fn main_loop_impl(
    args: PingArgs,
    mut stdin: ChildStdin,
    mut stdout: ChildStdout,
) -> Result<(), Error> {
    let len = 8 + args.size.get();

    let mut interval = interval(args.interval.0);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let hashmap = BorrowCell::new(HashMap::with_capacity(10));

    // Writing and reading from the remote child is done in parallel since there
    // is no guarantee on whether the buffer is flushed for every new line.
    tokio::try_join!(
        async {
            let mut buffer: Vec<u8> = (0..len)
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
            let mut buffer: Vec<u8> = (0..len).map(|_n| 0).collect();

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

    Ok(())
}

pub async fn main_loop(args: PingArgs, _verbose: Verbosity, session: Session) -> Result<(), Error> {
    let mut child = session
        .command("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .await?;

    let stdin = child.stdin().take().unwrap();
    let stdout = child.stdout().take().unwrap();

    tokio::select! {
        res = main_loop_impl(args, stdin, stdout) => {
            res?;

            let exit_status = child.wait().await?;

            if !exit_status.success() {
                eprintln_error!("Failed to execute cut on remote: {exit_status:#?}");
            }

            Ok::<_, Error>(())
        },

        _ = ctrl_c() => {
            child.disconnect().await.map_err(Error::Remote)?;
            Ok::<_, Error>(())
        },
    }?;

    session.close().await
}