mod upload;
use upload::speedtest_upload;

use super::utility::{println_on_level, Level};
use super::SshSessionBuilder;

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use openssh::Error;

#[derive(Debug, Parser, Copy, Clone)]
pub struct SpeedTestArgs {
    /// Disable testing upload speed.
    #[clap(long)]
    no_upload: bool,

    /// Disable testing download speed.
    #[clap(long)]
    no_download: bool,
}

pub async fn run(
    args: SpeedTestArgs,
    verbose: Verbosity,
    builder: SshSessionBuilder<'_>,
) -> Result<(), Error> {
    let dest = builder.dest();

    println_on_level!(verbose, Level::Debug, "Attempting to connect to {dest}");
    let session = builder.connect().await?;

    if !args.no_upload {
        speedtest_upload(verbose.clone(), &session).await?;
    }

    session.close().await
}
