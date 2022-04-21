mod bonzomatic;
mod bonzomatic_launcher;
mod radio;
mod recorder;
mod replayer;
mod server;
mod utils;
use clap::{Parser, Subcommand};
use core::future::Future;

use log::info;
use log::LevelFilter;
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
    /// Start a websocket server
    Server {
        /// Host or Host:Port
        #[clap(long, default_value_t = String::from("0.0.0.0:9785"))]
        bind_addr: String,

        /// Disable shader autosave
        #[clap(long)]
        save_shader_disable: bool,

        /// Directory where shaders are saved
        #[clap(short, long, parse(from_os_str), default_value = "./shaders")]
        save_shader_dir: PathBuf,
    },
    /// Helper function to record localy
    BonzoRecord {
        /// Protocol
        #[clap( short, long, default_value_t = String::from("ws"))]
        protocol: String,

        /// Host or Host:Port
        #[clap( long, default_value_t = String::from("127.0.0.1:9785"))]
        host: String,

        /// Room
        #[clap( long, default_value_t = String::from("local_replay"))]
        room: String,

        /// Handle
        #[clap( long, default_value_t = String::from("replay"))]
        handle: String,

        /// Root of Bonzomatic Directory
        #[clap(short, long, parse(from_os_str), default_value = r#"./"#)]
        bonzomatic_path: PathBuf,

        /// Directory where shaders are saved
        #[clap(short, long, parse(from_os_str), default_value = "./shaders")]
        save_shader_dir: PathBuf,
    },
    /// Helper function to replay localy
    BonzoReplay {
        /// Protocol
        #[clap( short, long, default_value_t = String::from("ws"))]
        protocol: String,

        /// Host or Host:Port
        #[clap( long, default_value_t = String::from("127.0.0.1:9785"))]
        host: String,

        /// Room
        #[clap( long, default_value_t = String::from("local_replay"))]
        room: String,

        /// Handles
        #[clap( long, default_value_t = String::from("replay"))]
        handle: String,

        /// Input Json file
        file: String,

        /// udpateInterval (ms)
        #[clap(long, default_value_t = 300u64)]
        update_interval: u64,

        /// Root of Bonzomatic Directory
        #[clap(short, long, parse(from_os_str), default_value = r#"./"#)]
        bonzomatic_path: PathBuf,
    },
}

fn start_tokio<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

fn start_tokio_with_bonzomatic<F: Future>(
    future: F,
    bonzomatic_path: &PathBuf,
    bonzomatic_server_url: &String,
    bonzomatic_mode: bonzomatic_launcher::NetworkMode,
) -> F::Output {
    let os_string = bonzomatic_path.as_path();
    bonzomatic_launcher::bonzomatic_args(
        &String::from(os_string.as_os_str().to_str().unwrap()),
        true,
        bonzomatic_mode,
        &bonzomatic_server_url,
    );
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let cli = Bts::parse();
    match &cli.command {
        Commands::BonzoReplay {
            protocol,
            host,
            room,
            handle,
            file,
            bonzomatic_path,
            update_interval,
        } => {
            info!("Start Local Replayer");
            let bonzomatic_server_url = utils::get_ws_url(protocol, host, room, handle);
            start_tokio_with_bonzomatic(
                async {
                    let replayer =
                        replayer::replay(protocol, host, room, handle, file, update_interval);
                    let path = PathBuf::new();
                    let server = server::main(host, &true, &path);
                    tokio::join!(replayer, server)
                },
                bonzomatic_path,
                &bonzomatic_server_url,
                bonzomatic_launcher::NetworkMode::Grabber,
            );
            info!("End Local Replayer")
        }
        Commands::BonzoRecord {
            protocol,
            host,
            room,
            handle,
            bonzomatic_path,
            save_shader_dir,
        } => {
            info!("Start Local Recorder");
            let bonzomatic_server_url = utils::get_ws_url(protocol, host, room, handle);
            start_tokio_with_bonzomatic(
                server::main(host, &false, save_shader_dir),
                bonzomatic_path,
                &bonzomatic_server_url,
                bonzomatic_launcher::NetworkMode::Sender,
            );
            info!("End Local Recorder")
        }
        Commands::Recorder {
            protocol,
            host,
            room,
            handle,
        } => {
            info!("Start Recorder");
            start_tokio(recorder::record(protocol, host, room, handle));
            info!("End Recorder")
        }
        Commands::Replayer {
            protocol,
            host,
            room,
            handle,
            file,
            update_interval,
        } => {
            info!("{file}");
            start_tokio(replayer::replay(
                protocol,
                host,
                room,
                handle,
                file,
                update_interval,
            ));
            info!("End Replayer")
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
            info!("Starting Radio");
            start_tokio(radio::radio(
                protocol,
                host,
                room,
                handle,
                path,
                update_interval,
                time_per_entry,
            ));
        }
        Commands::Server {
            bind_addr,
            save_shader_disable,
            save_shader_dir,
        } => start_tokio(server::main(
            bind_addr,
            save_shader_disable,
            save_shader_dir,
        )),
    }
}
