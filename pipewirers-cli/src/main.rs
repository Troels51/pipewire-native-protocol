use std::env;

use pipewire_native_protocol::PipewireClient;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut address = Option::None;
    if let Ok(pipewire_runtime_dir) = env::var("PIPEWIRE_RUNTIME_DIR") {
        address = Some(pipewire_runtime_dir + "pipewire-0");
    } else if let Ok(xdg_runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        address = Some(xdg_runtime_dir + "pipewire-0");
    } else if let Ok(userprofile) = env::var("USERPROFILE") {
        address = Some(userprofile + "pipewire-0");
    }
    if let Some(address) = address {
        let stream = tokio::net::UnixStream::connect(address).await?;
        let client = PipewireClient::connect(stream).await;
        // wait for keypress
        let mut line = String::new();
        let input = std::io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");
    } else {
        println!("Could not find pipewire socket");
    }
    Ok(())
}
