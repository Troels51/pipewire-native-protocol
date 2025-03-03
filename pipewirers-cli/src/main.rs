use std::{env, io::Write};

use clap::Parser;
use clap::Subcommand;
use pipewire_native_protocol::proxy::Proxy;
use pipewire_native_protocol::registry::RegistryEvent;
use pipewire_native_protocol::PipewireConnection;

#[derive(Debug, Parser)]
#[command(multicall = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Ping,
    Ls,
    Exit,
}

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
    let address = address.expect("Could not find pipewire socket");
    let stream = tokio::net::UnixStream::connect(address).await?;
    let  (mut core_proxy, mut client_proxy) = PipewireConnection::connect(stream).await?;

    let mut registry = core_proxy.get_registry().await?;

    // Start repl
    loop {
        let line = readline();
        match line {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Some(args) = shlex::split(line) {
                    if let Ok(cli) = Cli::try_parse_from(args) {
                        match cli.command {
                            Commands::Ping => {
                                repl_write("Pong");
                            }
                            Commands::Exit => {
                                repl_write("Exiting ....");
                                return Ok(());
                            }
                            Commands::Ls => {
                                registry.sync().await?;
                                while let Some(event) = registry.recv().await {
                                    // TODO: Store the globals we receive, so that next time around we just look at what came untill the next done event
                                    match event {
                                        RegistryEvent::Done(done) => break,
                                        _ => {println!("{}", event);}
                                    }
                                }
                            },
                        }
                    } else {
                        repl_write("Command not found");
                    }
                } else {
                    repl_write("Could not parse input");
                }
            }
            Err(error) => {
                println!("{}", error);
                break;
            }
        }
    }

    Ok(())
}

fn repl_write(line: &str) {
    write!(std::io::stdout(), "{} \n", line).expect("Could not write to std out");
    std::io::stdout()
        .flush()
        .expect("Could not write to std out");
}

fn readline() -> Result<String, std::io::Error> {
    write!(std::io::stdout(), "pw: ")?;
    std::io::stdout().flush()?;
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    Ok(buffer)
}
