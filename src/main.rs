mod ping;
use ping::{run, PingArgs};

mod utility;
use utility::eprintln_error;

use clap::{IntoApp, Parser};
use clap_verbosity_flag::Verbosity;
use std::process::exit;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Verbosity.
    #[clap(flatten)]
    verbose: Verbosity,

    #[clap(subcommand)]
    subcommand: SubCommand,

    /// hostname to ping.
    #[clap(global(true))]
    hostname: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum SubCommand {
    Ping(PingArgs),
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let hostname = match args.hostname.as_ref() {
        Some(hostname) => hostname,
        None => {
            eprintln_error!("ERROR: Expected positional argument hostname!\n");
            Args::into_app().print_long_help().unwrap();
            exit(1)
        }
    };

    println!("{args:#?}");
}
