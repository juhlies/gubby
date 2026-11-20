use clap::{Parser, Subcommand};
use dirs;
use std::fs;

#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Commands>
}
#[derive(Subcommand)]
enum Commands {
    /// checks packages for updates
    Check {},

    /// opens gubby config file
    Config {},

    /// installs a package
    Install {
        #[arg(short)]
        url: String
    },

    /// lists all installed packages
    List {},

    /// installs the gubby binary
    SelfInstall {},

    /// installs the latest version of gubby
    SelfUpgrade {},

    /// uninstalls a package
    Uninstall {
        #[arg(short)]
        url: String
    },

    /// updates all installed packages
    Update {
        #[arg(short)]
        url: String
    },
}

fn main() {
    let args = Arguments::parse();
    match &args.command {
        Some(Commands::SelfInstall {  }) => {
            println!("gubby installed yay")
        }

        Some(Commands::SelfUpgrade {  }) => {
            println!("gubby updated yay")
        }

        Some(Commands::List {  }) => {
            let user_folder = dirs::home_dir().expect("could not find user directory.");
            let config_file = fs::read_to_string(user_folder.join(".gubby").join("gubby.toml")).expect("could not read config file");
            println!("{}", config_file)
        }

        Some(Commands::Install { url }) => {
            println!("installing: {}", url)
        }

        Some(Commands::Uninstall { url }) => {
            println!("uninstalling: {}", url)
        }

        Some(Commands::Check {  }) => {
            println!("checking packages for updates")
        }

        Some(Commands::Update { url }) => {
            println!("updating: {}", url)
        }

        None => {}
    }
}