use std::process::Command;
pub enum NetworkMode {
    Sender,
    Grabber,
}
fn get_network_mode(mode: NetworkMode) -> String {
    let varname = "networkMode";
    match mode {
        NetworkMode::Sender => format!("{varname}=sender"),
        NetworkMode::Grabber => format!("{varname}=grabber"),
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
    let mut c = Command::new(r#".\Bonzomatic_W64_GLFW.exe"#);
    c.current_dir(bonzo_dir).args(v);
    println!("{:?}", c);
    c.spawn().unwrap()
    /*
    let &mut c: Command = Command::new("./bonzomatic");
    c = c.current_dir("/home/totetmatt/playground/Bonzomatic");
    if skipdialog {
        c = c.arg("skipdialog");
    }bonzomaticluncher::bonzomatic_cmd(true);
    c = c.arg("skipdialog").arg("networkMode=sender").arg(format!(
        "serverURL={}",
        &"ws://drone.alkama.com:9000/livecode/replayer"
    ));*/
}
