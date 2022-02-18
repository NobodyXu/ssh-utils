use super::human_readable_unit::HumanReadableUnit;
use super::{println_on_level, Level};

use clap_verbosity_flag::Verbosity;
use openssh::{ChildStdin, Error, Session, Stdio};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::signal::ctrl_c;
use tokio::time::{interval, MissedTickBehavior};

async fn upload(
    child_stdin: &mut ChildStdin,
    n: &mut u64,
    buffer: &[u8],
    verbose: Verbosity,
) -> Result<(), Error> {
    loop {
        println_on_level!(verbose, Level::Debug, "Uploading");
        let cnt = child_stdin.write(buffer).await.map_err(Error::ChildIo)?;
        let cnt: u64 = cnt.try_into().unwrap();
        *n += cnt;
    }
}

pub async fn speedtest_upload(verbose: Verbosity, session: &Session) -> Result<(), Error> {
    let mut interval = interval(Duration::from_secs(5));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    println_on_level!(verbose, Level::Debug, "Spawning process dd on remote");
    let mut child = session
        .command("dd")
        .arg("of=/dev/null")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .await?;

    let mut child_stdin = child.stdin().take().unwrap();
    let mut n = 0;

    // Buffer is 4096 bytes large, the size of pipe.
    let buffer: Vec<u8> = (0..255_u8).cycle().take(4096).collect();

    let shutdown_requested = ctrl_c();
    tokio::pin!(shutdown_requested);

    let instant = Instant::now();

    while n < 4096 * 100 {
        tokio::select! {
            res = upload(&mut child_stdin, &mut n, &buffer, verbose.clone()) => res?,
            _ = interval.tick() => (),
            _ = &mut shutdown_requested => {
                println_on_level!(verbose, Level::Debug, "Ctrl C signal received");
                break
            }
        }
    }

    drop(child_stdin);

    // Wait for all bytes to be read by dd
    let exit_status = child.wait().await?;
    let elapsed = instant.elapsed();

    if !exit_status.success() {
        println_on_level!(
            verbose,
            Level::Error,
            "Failed to execute cat on remote: {exit_status:#?}"
        );
    }

    println!("{n} bytes is uploaded in {elapsed:#?}");

    Ok(())
}
