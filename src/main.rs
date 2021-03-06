mod ping;
use ping::PingArgs;

mod interval;
use interval::Interval;

mod utility;
use utility::eprintln_error;

mod ssh_session_builder;
use ssh_session_builder::SshSessionBuilder;

mod speedtest;
use speedtest::SpeedTestArgs;

use clap::{IntoApp, Parser};
use clap_verbosity_flag::Verbosity;
use openssh::SessionBuilder;
use std::path::PathBuf;
use std::process::exit;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Verbosity.
    #[clap(flatten)]
    verbose: Verbosity,

    #[clap(short, long, default_value_t = Interval::from_secs(10))]
    timeout: Interval,

    #[clap(short = 'F', long)]
    config_file: Option<PathBuf>,

    #[clap(subcommand)]
    subcommand: SubCommand,

    /// hostname to ping.
    #[clap(global(true))]
    hostname: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum SubCommand {
    Ping(PingArgs),
    SpeedTest(SpeedTestArgs),
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let hostname = match args.hostname.as_ref() {
        Some(hostname) => hostname,
        None => {
            eprintln_error!("ERROR: Expected positional argument hostname!\n");
            Args::command().print_long_help().unwrap();
            exit(1)
        }
    };

    let mut builder = SessionBuilder::default();

    builder.connect_timeout(args.timeout.0);
    if let Some(config_file) = args.config_file.as_ref() {
        builder.config_file(config_file);
    }

    let builder = SshSessionBuilder::new(builder, hostname);

    let res = match args.subcommand {
        SubCommand::Ping(ping_args) => ping::run(ping_args, args.verbose, builder).await,
        SubCommand::SpeedTest(speedtest_args) => {
            speedtest::run(speedtest_args, args.verbose, builder).await
        }
    };

    if let Err(error) = res {
        eprintln_error!("Failed to login to {}: {:#?}!", hostname, error);
        exit(1);
    }
}
