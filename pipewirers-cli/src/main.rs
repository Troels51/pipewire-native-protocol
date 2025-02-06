use std::env;

use pipewire_native_protocol::PipewireConnection;

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
        let mut connection = PipewireConnection::connect(stream).await?;
        let mut core_proxy = connection.create_core_proxy().await?;
        let client_proxy = connection.create_client_proxy().await;
        let registry = core_proxy.get_registry().await;
        // let registry = core_proxy.sync(5).await;

        client_proxy.update_properties().await?;
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
