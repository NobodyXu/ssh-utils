use super::{println_if_not_quiet, println_on_level, Level, PingArgs};

use clap_verbosity_flag::Verbosity;
use openssh::{ChildStdin, ChildStdout, Error, Session, Stdio};
use owo_colors::{OwoColorize, Stream::Stdout};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::signal::ctrl_c;
use tokio::time::{interval, MissedTickBehavior};

async fn main_loop_impl(
    args: PingArgs,
    verbose: Verbosity,
    mut stdin: ChildStdin,
    mut stdout: ChildStdout,
    stats: &mut Vec<Duration>,
) -> Result<(), Error> {
    let len = 8 + args.size.get();

    let mut interval = interval(args.interval.0);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let mut output_buffer: Vec<u8> = (0..len)
        .map(|n| (n % (u8::MAX as u64)).try_into().unwrap())
        .collect();

    *output_buffer.last_mut().unwrap() = b'\n';

    let mut input_buffer: Vec<u8> = (0..len).map(|_n| 0).collect();

    for seq in 0..args.count {
        let seq_buffer: &mut [u8; 8] = (&mut output_buffer[..8]).try_into().unwrap();
        seq_buffer.copy_from_slice(&seq.to_be_bytes());

        interval.tick().await;

        println_on_level!(
            verbose,
            Level::Debug,
            "Sending message seq = {seq} to remote"
        );

        let instant = Instant::now();
        stdin
            .write_all(&output_buffer)
            .await
            .map_err(Error::ChildIo)?;

        println_on_level!(verbose, Level::Debug, "Reading from child_stdout");
        stdout
            .read_exact(&mut input_buffer)
            .await
            .map_err(Error::ChildIo)?;

        let seq_buffer: &mut [u8; 8] = (&mut input_buffer[..8]).try_into().unwrap();
        let seq_received = u64::from_be_bytes(*seq_buffer);

        println_on_level!(
            verbose,
            Level::Debug,
            "Received message seq = {seq_received} from remote"
        );

        if seq == seq_received {
            let elapsed = instant.elapsed();
            println_if_not_quiet!(
                verbose,
                "{}: seq = {seq}, time = {elapsed:#?}",
                "Logined".if_supports_color(Stdout, |text| text.green())
            );

            stats.push(elapsed);
        } else {
            println_on_level!(verbose, Level::Warn, "Unexpected packet: seq = {seq}");
        }
    }

    Ok(())
}

/// Cancel safe, shutdown gracefully on ctrl_c
pub async fn main_loop(
    args: PingArgs,
    verbose: Verbosity,
    session: Session,
    stats: &mut Vec<Duration>,
) -> Result<(), Error> {
    println_on_level!(verbose, Level::Debug, "Spawning process cat on remote");
    let mut child = session
        .command("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .await?;

    let stdin = child.stdin().take().unwrap();
    let stdout = child.stdout().take().unwrap();

    tokio::select! {
        res = main_loop_impl(args, verbose.clone(), stdin, stdout, stats) => {
            res?;

            let exit_status = child.wait().await?;

            if !exit_status.success() {
                println_on_level!(verbose, Level::Warn, "Failed to execute cat on remote: {exit_status:#?}");
            }

            Ok::<_, Error>(())
        },

        _ = ctrl_c() => {
            println_on_level!(verbose, Level::Debug, "Ctrl C signal received");
            child.disconnect().await.map_err(Error::Remote)?;
            Ok::<_, Error>(())
        },
    }?;

    session.close().await
}
