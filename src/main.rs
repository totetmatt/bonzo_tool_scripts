pub mod recorder;
use clap::{AppSettings, Parser, Subcommand};
use recorder::Client;
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
            let running = Arc::new(AtomicBool::new(true));
            let r = running.clone();
            ctrlc::set_handler(move || {
                r.store(false, Ordering::SeqCst);
            })
            .expect("Error setting Ctrl-C handler");

            Client::init(protocol, host, room, handle, running.clone());
            println!("End")
        }
    }
}
