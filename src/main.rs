mod ping;
use ping::PingArgs;

use clap::Parser;
use clap_verbosity_flag::Verbosity;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Verbosity.
    #[clap(flatten)]
    verbose: Verbosity,

    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(clap::Subcommand, Debug)]
enum SubCommand {
    Ping(PingArgs),
}

fn main() {
    let args = Args::parse();

    println!("{args:#?}");
}
