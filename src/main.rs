use clap::{Parser, Subcommand, CommandFactory, builder::styling::{AnsiColor, Effects, Styles}};
mod commands;
mod github;
mod paths;

#[derive(Parser)]
#[command(version, disable_version_flag = true, disable_help_flag = true, disable_help_subcommand = true, styles = styles())]
struct Arguments {
    #[command(subcommand)]
    command: Option<Commands>,

    /// print version
    #[arg(short = 'v', long, action = clap::ArgAction::Version)]
    version: (),

    /// print this message
    #[arg(short = 'h', long, action = clap::ArgAction::Help)]
    help: (),
}

#[derive(Subcommand)]
enum Commands {
    /// checks packages for updates
    Check {
        url: Option<String>
    },

    /// opens gubby config file
    Config {},

    /// installs a package
    Install {
        url: String,
        version: Option<String>
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
        url: String
    },

    /// updates all installed packages
    Update {
        url: Option<String>
    },

    /// print this message
    Help {}
}

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .placeholder(AnsiColor::Cyan.on_default())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Red.on_default())
}

fn handle_error(result: anyhow::Result<()>) {
    if let Err(error) = result {
        eprintln!("{error}");
    }
}

#[tokio::main]
async fn main() {
    let args = Arguments::parse();

    match &args.command {
        Some(Commands::Check { url }) => handle_error(commands::check::run(url.as_deref()).await),
        Some(Commands::Config {}) => commands::config::run(),
        Some(Commands::Install { url, version }) => handle_error(commands::install::run(url, version.as_deref(), "installing", "installed").await),
        Some(Commands::List {}) => handle_error(commands::list::run()),
        Some(Commands::SelfInstall {}) => handle_error(commands::self_install::run().await),
        Some(Commands::SelfUninstall {}) => handle_error(commands::self_uninstall::run()),
        Some(Commands::SelfUpgrade {}) => handle_error(commands::self_upgrade::run().await),
        Some(Commands::Uninstall { url }) => handle_error(commands::uninstall::run(url)),
        Some(Commands::Update { url }) => handle_error(commands::update::run(url.as_deref()).await),
        Some(Commands::Help {}) => {Arguments::command().print_help().unwrap(); println!()},
        None => {Arguments::command().print_help().unwrap(); println!()}
    }
}