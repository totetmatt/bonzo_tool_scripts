pub mod recorder;
pub mod replayer;
pub mod bonzomatic;
mod utils;
use clap::{AppSettings, Parser, Subcommand};

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

        /// Handle
        handle: String,


    },
    /// Replay a saved entry to a websocket entrypoint
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

        /// udpateInterval (ms)
        #[clap( long, default_value_t = 300u64)]
        update_interval: u64,
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
            update_interval
        } => {
            println!("{file}");
            replayer::replay(protocol, host, room, handle, file, update_interval);
            println!("End Replayer")
        }
    }
}
