use clap::{Parser, Subcommand};
mod commands;
mod paths;

#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// checks packages for updates
    Check {},

    /// opens gubby config file
    Config {},

    /// installs a package
    Install {
        url: String,
    },

    /// lists all installed packages
    List {},

    /// installs the gubby binary
    SelfInstall {},

    /// uninstalls gubby
    SelfUninstall {},

    /// installs the latest version of gubby
    SelfUpgrade {},

    /// uninstalls a package
    Uninstall {
        url: String,
    },

    /// updates all installed packages
    Update {
        url: Option<String>,
    },
}

fn main() {
    let args = Arguments::parse();

    match &args.command {
        Some(Commands::Check {}) => commands::check::run(),
        Some(Commands::Config {}) => commands::config::run(),
        Some(Commands::Install { url }) => commands::install::run(url),
        Some(Commands::List {}) => commands::list::run(),
        Some(Commands::SelfInstall {}) => commands::self_install::run(),
        Some(Commands::SelfUninstall {}) => commands::self_uninstall::run(),
        Some(Commands::SelfUpgrade {}) => commands::self_upgrade::run(),
        Some(Commands::Uninstall { url }) => commands::uninstall::run(url),
        Some(Commands::Update { url }) => commands::update::run(url.as_deref()),
        None => {}
    }
}