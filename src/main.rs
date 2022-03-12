pub mod bonzomatic;
pub mod radio;
pub mod recorder;
pub mod replayer;

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
        #[clap(long, default_value_t = 300u64)]
        update_interval: u64,
    },

    /// Send multiple shader at certain interval form a given playlist to an entrypoint, like a radio
    Radio {
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

        /// Glob path of source files (playlist)
        path: String,

        /// udpateInterval (ms)
        #[clap(long, default_value_t = 500u64)]
        update_interval: u64,

        /// Time of boradcast per entry (ms)
        #[clap(long, default_value_t = 10000u64)]
        time_per_entry: u64,
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
            update_interval,
        } => {
            println!("{file}");
            replayer::replay(protocol, host, room, handle, file, update_interval);
            println!("End Replayer")
        }
        Commands::Radio {
            protocol,
            host,
            room,
            handle,
            path,
            update_interval,
            time_per_entry,
        } => {
            println!("Starting Radio");
            radio::radio(
                protocol,
                host,
                room,
                handle,
                path,
                update_interval,
                time_per_entry,
            )
        }
    }
}
