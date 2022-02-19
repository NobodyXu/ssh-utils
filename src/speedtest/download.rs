use super::human_readable_unit::HumanReadableUnit;
use super::{println_on_level, Level};

use clap_verbosity_flag::Verbosity;
use openssh::{ChildStdout, Error, Session, Stdio};
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::signal::ctrl_c;
use tokio::time::{interval, MissedTickBehavior};

async fn download(
    child_stdout: &mut ChildStdout,
    n: &mut u64,
    buffer: &mut [u8],
    verbose: Verbosity,
) -> Result<(), Error> {
    loop {
        println_on_level!(verbose, Level::Debug, "Downloading");
        let cnt = child_stdout.read(buffer).await.map_err(Error::ChildIo)?;
        let cnt: u64 = cnt.try_into().unwrap();

        if cnt == 0 {
            // break on EOF
            break Ok(());
        }

        *n += cnt;
    }
}

pub async fn speedtest_download(verbose: Verbosity, session: &Session) -> Result<(), Error> {
    let mut interval = interval(Duration::from_secs(5));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    println_on_level!(verbose, Level::Debug, "Spawning process seq on remote");
    let mut child = session
        .command("seq")
        .arg("0")
        .arg("1e9")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .await?;

    let mut child_stdout = child.stdout().take().unwrap();
    let mut n = 0;

    // Buffer is 4096 bytes large, the size of pipe.
    let mut buffer: Vec<u8> = vec![0; 4096];

    let shutdown_requested = ctrl_c();
    tokio::pin!(shutdown_requested);

    let instant = Instant::now();

    while n < 4096 * 100 {
        tokio::select! {
            res = download(&mut child_stdout, &mut n, &mut buffer, verbose.clone()) => {
                res?;
                // break on EOF
                break
            },
            _ = interval.tick() => (),
            _ = &mut shutdown_requested => {
                println_on_level!(verbose, Level::Debug, "Ctrl C signal received");
                break
            }
        }
    }
    let elapsed = instant.elapsed();
    drop(child_stdout);

    println!(
        "{} is downloaded in {elapsed:#?}, download speed = {}/s",
        HumanReadableUnit::new(n),
        HumanReadableUnit::new(n / elapsed.as_secs())
    );

    // Wait for remote process seq
    match child.wait().await {
        Ok(exit_status) => {
            if !exit_status.success() {
                println_on_level!(
                    verbose,
                    Level::Error,
                    "Failed to execute seq on remote: {exit_status:#?}"
                );
            }
        }

        Err(Error::RemoteProcessTerminated) => {
            println_on_level!(
                verbose,
                Level::Debug,
                "remote process seq terminated due to child_stdout closed"
            )
        }

        Err(err) => return Err(err),
    };

    Ok(())
}
