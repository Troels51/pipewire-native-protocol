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
use core_proxy::{CoreEvent, Ping};
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

use tokio_stream::StreamExt;
use tokio_util::{
    bytes::BytesMut,
    codec::{Framed, FramedRead, LengthDelimitedCodec},
};
pub struct PipewireConnection {
    writer: Arc<Mutex<PipewireWriter>>,
    reader: PipewireReaderHandle,
    proxies: Arc<Mutex<Proxies>>,
}
pub(crate) struct PipewireWriter {
    stream: tokio::net::unix::OwnedWriteHalf,
    seq: u32,
}

enum PipewireReaderMessage {
    SetCoreProxy(tokio::sync::mpsc::Sender<CoreEvent>),
    SetClientProxy(tokio::sync::mpsc::Sender<ClientEvent>),
}

#[derive(Debug)]
struct PipewireReader {
    stream: FramedRead<tokio::net::unix::OwnedReadHalf, LengthDelimitedCodec>,
    control: tokio::sync::mpsc::Receiver<PipewireReaderMessage>,
    proxies: Arc<Mutex<Proxies>>,
}

// The collection of proxies currently active on a connection
#[derive(Debug)]
struct Proxies {
    id_counter: i32, // Gets increment each time a new proxy is allocated
    core_proxy: Option<tokio::sync::mpsc::Sender<CoreEvent>>,
    client_proxy: Option<tokio::sync::mpsc::Sender<ClientEvent>>,
    registry_proxies: HashMap<i32, tokio::sync::mpsc::Sender<RegistryEvent>>,
    device_proxies: HashMap<i32, tokio::sync::mpsc::Sender<DeviceEvent>>,
    factory_proxies: HashMap<i32, tokio::sync::mpsc::Sender<FactoryEvent>>,
    link_proxies: HashMap<i32, tokio::sync::mpsc::Sender<LinkEvent>>,
    module_proxies: HashMap<i32, tokio::sync::mpsc::Sender<ModuleEvent>>,
    node_proxies: HashMap<i32, tokio::sync::mpsc::Sender<NodeEvent>>,
    port_proxies: HashMap<i32, tokio::sync::mpsc::Sender<PortEvent>>,
    client_node_proxies: HashMap<i32, tokio::sync::mpsc::Sender<ClientNodeEvent>>,
    metadata_proxies: HashMap<i32, tokio::sync::mpsc::Sender<MetadataEvent>>,
    profiler_proxies: HashMap<i32, tokio::sync::mpsc::Sender<ProfilerEvent>>,
}

impl Default for Proxies {
    fn default() -> Self {
        Self {
            id_counter: 1, // Core and Client Proxies already have id 0 and 1
            core_proxy: Default::default(),
            client_proxy: Default::default(),
            registry_proxies: Default::default(),
            device_proxies: Default::default(),
            factory_proxies: Default::default(),
            link_proxies: Default::default(),
            module_proxies: Default::default(),
            node_proxies: Default::default(),
            port_proxies: Default::default(),
            client_node_proxies: Default::default(),
            metadata_proxies: Default::default(),
            profiler_proxies: Default::default(),
        }
    }
}

pub(crate) struct PipewireReaderHandle {
    sender: tokio::sync::mpsc::Sender<PipewireReaderMessage>,
}

impl PipewireReaderHandle {
    pub fn new(stream: tokio::net::unix::OwnedReadHalf, proxies: Arc<Mutex<Proxies>>) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        let reader = PipewireReader::new(stream, receiver, proxies);
        tokio::spawn(run_reader(reader));
        Self { sender }
    }
}

async fn run_reader(mut reader: PipewireReader) {
    while let Some(msg) = reader.stream.next().await {
        match msg {
            Ok(msg) => {
                match reader.get_event_from_bytes(msg) {
                    Ok(event) => match event {
                        Event::Core(core_event) => {dbg!(core_event);},
                        Event::Registry(registry_event) => {dbg!(registry_event);},
                        Event::Client(client_event) => {dbg!(client_event);},
                    },
                    Err(e) => {dbg!(e);},
                }
            },
            Err(e) => {
                println!("Error {}", e);
            },
        }
    }
}

impl PipewireConnection {
    pub async fn connect(stream: tokio::net::UnixStream) -> io::Result<Self> {
        let (input_stream, output_stream) = stream.into_split();
        let proxies = Arc::new(Mutex::new(Proxies::default()));
        let reader = PipewireReaderHandle::new(input_stream, proxies.clone());
        let writer = PipewireWriter::new(output_stream);

        let writer = Arc::new(Mutex::new(writer));
        Ok(PipewireConnection {
            writer: writer,
            reader: reader,
            proxies: proxies,
        })
    }
    pub async fn create_core_proxy(&mut self) -> io::Result<core_proxy::CoreProxy> {
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        let _ = self
            .reader
            .sender
            .send(PipewireReaderMessage::SetCoreProxy(sender))
            .await;
        core_proxy::CoreProxy::new(self.writer.clone(), receiver, self.proxies.clone()).await
    }

    pub async fn create_client_proxy(&mut self) -> client::ClientProxy {
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        let _ = self
            .reader
            .sender
            .send(PipewireReaderMessage::SetClientProxy(sender))
            .await;
        client::ClientProxy::new(self.writer.clone(), receiver)
    }
}
impl PipewireWriter {
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

    async fn write<T: PodSerialize>(&mut self, message: &mut Message<T>) -> io::Result<()> {
        let buffer: Vec<u8> = PodSerializer::serialize(Cursor::new(Vec::new()), &message.payload)
            .unwrap()
            .0
            .into_inner();
        message.header.opcode_size += buffer.len() as u32;
        self.stream.write_all(message.header.as_bytes()).await?;
        self.stream.write_all(&buffer).await
    }

    fn new(output_stream: tokio::net::unix::OwnedWriteHalf) -> Self {
        Self {
            stream: output_stream,
            seq: 0,
        }
    }
}

impl PipewireReader {
    fn new(
        input_stream: tokio::net::unix::OwnedReadHalf,
        control: tokio::sync::mpsc::Receiver<PipewireReaderMessage>,
        proxies: Arc<Mutex<Proxies>>,
    ) -> Self {
        let reader: FramedRead<tokio::net::unix::OwnedReadHalf, LengthDelimitedCodec> =
            LengthDelimitedCodec::builder()
                .length_field_offset(4)
                .length_field_length(3)
                .num_skip(0)
                .length_adjustment(16)
                .native_endian()
                .new_read(input_stream);
        PipewireReader {
            stream: reader,
            control,
            proxies,
        }
    }

    fn get_event_from_bytes(&self, bytes: BytesMut) -> Result<Event, DeserializeError<&[u8]>> {
        match bytes.split_first_chunk::<16>() {
            Some((header_bytes, message_bytes)) => {
                let header = Header::read_from_bytes(header_bytes)
                    .expect("Length of byte slice must be equal to header size");
                let message =
                    self.deserialize_from_id_and_opcode(header.id, header.opcode(), message_bytes);
                match message {
                    Ok((remain, event)) => Ok(event),
                    Err(e) => {
                        dbg!(e);
                        Err(DeserializeError::InvalidType)
                    }
                }
            }
            None => {
                Err(DeserializeError::UnsupportedType) //TODO: Not corret err
            }
        }
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
                if self
                    .proxies
                    .lock()
                    .unwrap()
                    .registry_proxies
                    .contains_key(&id)
                {
                    let (remain, value) =
                        registry::RegistryEvent::deserialize_from_opcode(opcode, buffer)?;
                    Ok((remain, Event::Registry(value)))
                } else {
                    println!("Could not find id {} ", id);
                    Err(DeserializeError::InvalidType)
                }
            }
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
            opcode_size: size | (opcode << 24),
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
