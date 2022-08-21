use std::path::PathBuf;
use std::process::Command;

use log::info;

pub enum NetworkMode {
    Sender,
    Grabber,
}

fn get_network_mode(mode: NetworkMode) -> String {
    match mode {
        NetworkMode::Sender => String::from("networkMode=sender"),
        NetworkMode::Grabber => String::from("networkMode=grabber"),
    }
}

fn get_server_url(url: &String) -> String {
    format!("serverURL={url}")
}

pub fn bonzomatic_args(
    bonzo_dir: PathBuf,
    bonzo_name: String,
    skipdialog: bool,
    network_mode: NetworkMode,
    server_url: &String,
) -> std::process::Child {
    let mut v = vec![];
    if skipdialog {
        v.push(String::from("skipdialog"));
    }
    v.push(get_network_mode(network_mode));
    v.push(get_server_url(server_url));

    let mut cmd_bonzomatic = if cfg!(unix) {
        let potential_path = bonzo_dir.join(&bonzo_name);
        if potential_path.is_file() {
            Command::new(potential_path)
        } else {
            info!("Bonzomatic is not found in local folder. Fallback to PATH");
            Command::new(&bonzo_name)
        }
    } else {
        Command::new(bonzo_dir.join(bonzo_name))
    };

    cmd_bonzomatic.current_dir(bonzo_dir);
    cmd_bonzomatic.args(v).spawn().unwrap()
}
