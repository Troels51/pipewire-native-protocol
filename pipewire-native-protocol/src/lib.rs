pub mod client;
pub mod client_node;
pub mod core_proxy;
pub mod device;
pub mod factory;
pub mod link;
pub mod metadata;
pub mod module;
pub mod node;
pub mod port;
pub mod profiler;
pub mod registry;

use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, Mutex},
};

use client::{ClientEvent, ClientProxy};
use client_node::ClientNodeEvent;
use core_proxy::CoreEvent;
use device::DeviceEvent;
use factory::FactoryEvent;
use link::LinkEvent;
use metadata::MetadataEvent;
use module::ModuleEvent;
use node::NodeEvent;
use port::PortEvent;
use profiler::ProfilerEvent;
use registry::RegistryEvent;
use spa::{
    deserialize::DeserializeError,
    opcode::DeserializeFromOpCode,
    serialize::{PodSerialize, PodSerializer},
};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes};

pub struct PipewireConnection {
    inner: Arc<Mutex<InnerConnection>>,
}
pub(crate) struct InnerConnection {
    stream: tokio::net::UnixStream,
    seq: u32,
    core_proxy: Option<std::sync::mpsc::Sender<CoreEvent>>,
    client_proxy: Option<std::sync::mpsc::Sender<ClientEvent>>,
    registry_proxies: HashMap<i32, std::sync::mpsc::Sender<RegistryEvent>>,
    registry_id_counter: i32, // Gets increment each time a new registry is allocated
    device_proxies: HashMap<i32, std::sync::mpsc::Sender<DeviceEvent>>,
    factory_proxies: HashMap<i32, std::sync::mpsc::Sender<FactoryEvent>>,
    link_proxies: HashMap<i32, std::sync::mpsc::Sender<LinkEvent>>,
    module_proxies: HashMap<i32, std::sync::mpsc::Sender<ModuleEvent>>,
    node_proxies: HashMap<i32, std::sync::mpsc::Sender<NodeEvent>>,
    port_proxies: HashMap<i32, std::sync::mpsc::Sender<PortEvent>>,
    client_node_proxies: HashMap<i32, std::sync::mpsc::Sender<ClientNodeEvent>>,
    metadata_proxies: HashMap<i32, std::sync::mpsc::Sender<MetadataEvent>>,
    profiler_proxies: HashMap<i32, std::sync::mpsc::Sender<ProfilerEvent>>,
}

impl PipewireConnection {
    pub async fn connect(stream: tokio::net::UnixStream) -> io::Result<Self> {
        let inner = InnerConnection {
            stream,
            seq: 1,
            core_proxy: None,
            client_proxy: None,
            registry_proxies: HashMap::new(),
            registry_id_counter: 0,
            device_proxies: HashMap::new(),
            factory_proxies: HashMap::new(),
            link_proxies: HashMap::new(),
            module_proxies: HashMap::new(),
            node_proxies: HashMap::new(),
            port_proxies: HashMap::new(),
            client_node_proxies: HashMap::new(),
            metadata_proxies: HashMap::new(),
            profiler_proxies: HashMap::new(),
        };
        Ok(PipewireConnection {
            inner: Arc::new(Mutex::new(inner)),
        })
    }
    pub async fn create_core_proxy(&mut self) -> io::Result<core_proxy::CoreProxy> {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.inner.lock().unwrap().core_proxy = Some(sender);
        core_proxy::CoreProxy::new(self.inner.clone(), receiver).await
    }

    pub fn create_client_proxy(&mut self) -> client::ClientProxy {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.inner.lock().unwrap().client_proxy = Some(sender);
        client::ClientProxy::new(self.inner.clone(), receiver)
    }
}
impl InnerConnection {
    async fn call_method(
        &mut self,
        id: i32,
        opcode: u32,
        payload: impl PodSerialize,
    ) -> io::Result<()> {
        let mut message = Message::new(id, opcode, self.seq, payload);
        self.stream.writable().await?;
        self.write(&mut message).await?;
        self.seq += 1;
        Ok(())
    }

    async fn read(&mut self) -> io::Result<Event> {
        let mut header = Header::new_zeroed();
        let bytes_read = self.stream.read_exact(header.as_mut_bytes()).await?; // Yes, Yes, unsafe
        let mut buffer = vec![0; header.size()];
        let size = self.stream.read(&mut buffer).await;
        let (remain, value) =
            self.deserialize_from_id_and_opcode(header.id, header.opcode(), &buffer).unwrap();
        Ok(value)
    }

    async fn write<T: PodSerialize>(&mut self, message: &mut Message<T>) -> io::Result<()> {
        let buffer: Vec<u8> = PodSerializer::serialize(Cursor::new(Vec::new()), &message.payload)
            .unwrap()
            .0
            .into_inner();
        message.header.opcode_size += buffer.len() as u32;
        self.stream.write_all(message.header.as_bytes()).await?;
        self.stream.write_all(&buffer).await
    }

    fn deserialize_from_id_and_opcode<'a>(
        &self,
        id: i32,
        opcode: u32,
        buffer: &'a [u8],
    ) -> Result<(&'a [u8], Event), spa::deserialize::DeserializeError<&'a [u8]>>
    where
        Self: Sized,
    {
        match id {
            core_proxy::CORE_ID => {
                let (remain, value) =
                    core_proxy::CoreEvent::deserialize_from_opcode(opcode, buffer)?;
                Ok((remain, Event::Core(value)))
            }
            client::ClientProxy::CLIENT_ID => {
                let (remain, value) = client::ClientEvent::deserialize_from_opcode(opcode, buffer)?;
                Ok((remain, Event::Client(value)))
            }
            id => {
                if self.registry_proxies.contains_key(&id) {
                    let (remain, value) = registry::RegistryEvent::deserialize_from_opcode(opcode, buffer)?;
                    Ok((remain, Event::Registry(value)))
                }
                else {
                    Err(DeserializeError::InvalidType)
                }
            },
        }
    }
}

#[derive(IntoBytes, FromBytes, Immutable, Debug)]
#[repr(C)]
struct Header {
    id: i32,
    opcode_size: u32,
    seq: u32,
    n_fds: u32,
}

impl Header {
    fn new(id: i32, opcode: u32, size: u32, seq: u32, n_fds: u32) -> Self {
        Self {
            id,
            opcode_size: size + (opcode << 24),
            seq,
            n_fds,
        }
    }
    fn incomplete(id: i32, opcode: u32, seq: u32) -> Self {
        Self::new(id, opcode, 0, seq, 0)
    }
    fn size(&self) -> usize {
        (self.opcode_size & 0xffffff) as usize
    }
    fn opcode(&self) -> u32 {
        self.opcode_size >> 24
    }
}

struct Message<T>
where
    T: PodSerialize,
{
    header: Header,
    payload: T,
}

impl<T> Message<T>
where
    T: PodSerialize,
{
    fn new(id: i32, opcode: u32, seq: u32, payload: T) -> Self {
        Self {
            header: Header::incomplete(id, opcode, seq),
            payload,
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Core(core_proxy::CoreEvent),
    Registry(registry::RegistryEvent),
    Client(client::ClientEvent),
}
