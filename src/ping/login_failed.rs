use super::{PingArgs, SshSessionBuilder};

use clap_verbosity_flag::Verbosity;
use openssh::Error;
use std::time::Duration;

pub async fn main_loop(
    _args: PingArgs,
    _verbose: Verbosity,
    _builder: SshSessionBuilder<'_>,
    _stats: &mut Vec<Duration>,
) -> Result<(), Error> {
    todo!()
}
