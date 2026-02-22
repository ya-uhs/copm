use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "copm", version, about = "Package manager for AI coding assistants")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Install a package from GitHub (or all dependencies from copm.json)
    Install {
        /// Package specifier (e.g., user/repo). Omit to install all from copm.json.
        package: Option<String>,

        /// Install globally (~/.copilot/skills/ or ~/.claude/skills/)
        #[arg(short, long)]
        global: bool,
    },

    /// Uninstall a package
    Uninstall {
        /// Package name to uninstall
        package: String,

        /// Uninstall from global location
        #[arg(short, long)]
        global: bool,
    },

    /// List installed packages
    List {
        /// List globally installed packages
        #[arg(short, long)]
        global: bool,
    },

    /// Initialize copm.json in the current directory
    Init,
}
