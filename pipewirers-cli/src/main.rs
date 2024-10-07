use std::env;

use pipewirers::PipewireClient;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // let pipewire_runtime_dir = env::var("PIPEWIRE_RUNTIME_DIR").unwrap();
    // let pipewire_runtime_dir = "/run/user/1000".to_string();
    let mut address = Option::None;
    if let Ok(xdg_runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        address = Some(xdg_runtime_dir + "pipewire-0");
    }
    let userprofile = env::var("USERPROFILE");
    dbg!(&address);
    if let Some(address) = address {
        let stream = tokio::net::UnixStream::connect(address).await?;
        let client = PipewireClient::connect(stream).await;
    } else {
        println!("Could not find pipewire socket");
    }

    // wait for keypress
    let mut line = String::new();
    let input = std::io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");
    Ok(())
}
