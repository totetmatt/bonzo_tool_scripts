use std::path::PathBuf;
use std::process::Command;

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
    bonzo_dir: &String,
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
    let path: PathBuf = [bonzo_dir, r#"Bonzomatic_W64_GLFW.exe"#].iter().collect();
    let mut cmd_bonzomatic = Command::new(path);

    cmd_bonzomatic.current_dir(bonzo_dir).args(v);
    cmd_bonzomatic.spawn().unwrap()
}
