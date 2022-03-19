mod bonzomatic;
mod bonzomaticluncher;
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
use std::process::Command;

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

fn start_tokio<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

fn start_tokio_2<F: Future>(future: F) -> F::Output {
    /*Command::new("./bonzomatic")
    .current_dir("/home/totetmatt/playground/Bonzomatic")
    .arg("skipdialog")
    .arg("networkMode=sender")
    .arg(format!(
        "serverURL={}",
        &"ws://drone.alkama.com:9000/livecode/replayer"
    ))
    .spawn()
    .expect("");*/
    bonzomaticluncher::bonzomatic_args(
        &String::from(r#"C:\Users\totetmatt\Downloads\Bonzo_Network_12_x64"#),
        true,
        bonzomaticluncher::NetworkMode::Grabber,
        &String::from("ws://drone.alkama.com:9000/livecode/replay"),
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
            /*start_tokio(replayer::replay(
                protocol,
                host,
                room,
                handle,
                file,
                update_interval,
            ));*/
            start_tokio_2(replayer::replay(
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
