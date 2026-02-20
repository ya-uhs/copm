pub mod init;
pub mod install;
pub mod list;
pub mod uninstall;

use crate::cli::args::Command;
use crate::error::CopmError;

pub async fn dispatch(command: Command) -> Result<(), CopmError> {
    match command {
        Command::Install { package, global } => match package {
            Some(pkg) => install::run(&pkg, global).await,
            None => install::run_all().await,
        },
        Command::Uninstall { package, global } => uninstall::run(&package, global),
        Command::List { global } => list::run(global),
        Command::Init => init::run(),
    }
}
