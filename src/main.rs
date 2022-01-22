pub mod recorder;
pub mod replayer;
mod utils;
use clap::{AppSettings, Parser, Subcommand};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
#[derive(Parser)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
struct Bts {
    #[clap(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Record an entry from a websocket entrypoint
    Recorder {
        /// Protocol
        #[clap( short, long, default_value_t = String::from("ws"))]
        protocol: String,

        /// Host or Host:Port
        #[clap(long)]
        host: String,

        /// Room
        #[clap(long)]
        room: String,

        /// Handles
        handle: String,
    },
    Replayer {
        /// Protocol
        #[clap( short, long, default_value_t = String::from("ws"))]
        protocol: String,

        /// Host or Host:Port
        #[clap(long)]
        host: String,

        /// Room
        #[clap(long)]
        room: String,

        /// Handles
        handle: String,

        /// Input Json file
        file: String,
    },
}

fn main() {
    let cli = Bts::parse();
    match &cli.command {
        Commands::Recorder {
            protocol,
            host,
            room,
            handle,
        } => {
            println!("Start Recorder");
            recorder::record(protocol, host, room, handle);
            println!("End Recorder")
        }
        Commands::Replayer {
            protocol,
            host,
            room,
            handle,
            file,
        } => {
            println!("{file}");
            replayer::replay(protocol, host, room, handle, file);
            println!("End Replayer")
        }
    }
}
