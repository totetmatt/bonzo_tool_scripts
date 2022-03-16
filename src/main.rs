pub mod bonzomatic;
pub mod radio;
pub mod recorder;
pub mod replayer;
pub mod server;

mod utils;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
#[derive(Parser)]
#[clap(author, version, about)]
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
    Server {
        /// Host or Host:Port
        #[clap(long, default_value_t = String::from("0.0.0.0:8080"))]
        bind_addr: String,

        #[clap(long)]
        save_shader_disable: bool,

        /// Sets a custom config file
        #[clap(short, long, parse(from_os_str), default_value = "./shaders")]
        save_shader_dir: PathBuf,
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
        Commands::Server {
            bind_addr,
            save_shader_disable,
            save_shader_dir,
        } => tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(server::main(
                bind_addr,
                *save_shader_disable,
                save_shader_dir,
            )),
    }
}
