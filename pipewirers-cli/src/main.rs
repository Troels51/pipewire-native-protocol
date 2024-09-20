use std::env;

use pipewirers::PipewireClient;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let pipewire_runtime_dir = env::var("PIPEWIRE_RUNTIME_DIR").unwrap();
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR");
    let userprofile = env::var("USERPROFILE");
    let address = pipewire_runtime_dir + "/pipewire-0";
    let stream = tokio::net::UnixStream::connect(address).await?;
    let client = PipewireClient::connect(stream);
    Ok(())
}
