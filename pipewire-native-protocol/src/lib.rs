pub mod client;
pub mod client_node;
pub mod core_proxy;
pub mod device;
pub mod factory;
pub mod link;
pub mod metadata;
pub mod module;
pub mod node;
mod opcode;
pub mod port;
pub mod profiler;
pub mod registry;

use std::{collections::HashMap, io::Cursor};

use opcode::OpCode;
use spa::{
    deserialize::{self, DeserializeError, PodDeserialize, PodDeserializer},
    serialize::{PodSerialize, PodSerializer},
};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes};

pub struct PipewireClient {
    stream: tokio::net::UnixStream,
    seq: u32,
}

const CORE_ID: u32 = 0;
const CLIENT_ID: u32 = 1;

impl PipewireClient {
    pub async fn connect(stream: tokio::net::UnixStream) -> io::Result<Self> {
        let mut client = Self { stream, seq: 2 };

        client.hello(3).await?;
        let _ = client.update_properties().await;
        // let _ = client.get_registry().await;
        let value: Event = client.read().await?;
        println!("{:?}", value);
        Ok(client)
    }

    async fn call_method(
        &mut self,
        id: u32,
        opcode: u32,
        payload: impl PodSerialize,
    ) -> io::Result<()> {
        let mut message = Message::new(id, opcode, self.seq, payload);
        self.stream.writable().await?;
        let _ = message.write(&mut self.stream).await?;
        self.seq += 1;
        Ok(())
    }

    async fn read(&mut self) -> io::Result<Event> {
        let mut header = Header::new_zeroed();
        let bytes_read = self.stream.read_exact(header.as_mut_bytes()).await?; // Yes, Yes, unsafe
        let mut buffer = vec![0; header.size()];
        let size = self.stream.read(&mut buffer).await;
        let (remain, value) =
            Event::deserialize_from_id_and_opcode(header.id, header.opcode(), &buffer).unwrap();
        Ok(value)
    }

    async fn hello(&mut self, version: i32) -> io::Result<()> {
        self.call_method(CORE_ID, 1, core_proxy::Hello { version: version })
            .await
    }

    async fn update_properties(&mut self) -> io::Result<()> {
        self.call_method(
            CLIENT_ID,
            2,
            client::UpdateProperties {
                props: HashMap::from([("application.name".into(), "pipewirers".into())]),
            },
        )
        .await
    }

    async fn get_permissions(&mut self) -> io::Result<()> {
        let payload = client::GetPermissions { index: 0, num: 3 }; // TODO: Real values
        let mut message = Message::new(CLIENT_ID, 3, self.seq, payload);
        let _ = self.stream.writable().await;
        message.write(&mut self.stream).await?;
        Ok(())
    }

    async fn get_registry(&mut self) -> io::Result<()> {
        let payload = core_proxy::GetRegistry {
            version: 3,
            new_id: 500,
        };
        let mut message = Message::new(CORE_ID, 5, self.seq, payload);
        let _ = self.stream.writable().await;
        message.write(&mut self.stream).await?;
        Ok(())
    }
}

#[derive(IntoBytes, FromBytes, Immutable, Debug)]
#[repr(C)]
struct Header {
    id: u32,
    opcode_size: u32,
    seq: u32,
    n_fds: u32,
}

impl Header {
    fn new(id: u32, opcode: u32, size: u32, seq: u32, n_fds: u32) -> Self {
        Self {
            id,
            opcode_size: size + (opcode << 24),
            seq,
            n_fds,
        }
    }
    fn incomplete(id: u32, opcode: u32, seq: u32) -> Self {
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
    fn new(id: u32, opcode: u32, seq: u32, payload: T) -> Self {
        Self {
            header: Header::incomplete(id, opcode, seq),
            payload: payload,
        }
    }
    async fn write<'a, O: tokio::io::AsyncWrite + Unpin + 'a>(
        &mut self,
        mut out: O,
    ) -> io::Result<usize> {
        let buffer: Vec<u8> = PodSerializer::serialize(Cursor::new(Vec::new()), &self.payload)
            .unwrap()
            .0
            .into_inner();
        self.header.opcode_size = buffer.len() as u32 + self.header.opcode_size;
        out.write(self.header.as_bytes()).await?;
        out.write(&buffer).await
    }
}

#[derive(Debug)]
pub enum Event {
    //Core
    Core(core_proxy::CoreEvent),
    Registry(registry::RegistryEvent),
    Client(client::ClientEvent),
}

impl Event {
    fn deserialize_from_id_and_opcode<'de>(
        id: u32,
        opcode: u32,
        buffer: &'de [u8],
    ) -> Result<(&'de [u8], Self), spa::deserialize::DeserializeError<&'de [u8]>>
    where
        Self: Sized,
    {
        match id {
            CORE_ID => {
                let (remain, value) =
                    core_proxy::CoreEvent::deserialize_from_opcode(opcode, buffer)?;
                Ok((remain, Event::Core(value)))
            }
            CLIENT_ID => {
                let (remain, value) = client::ClientEvent::deserialize_from_opcode(opcode, buffer)?;
                Ok((remain, Event::Client(value)))
            }
            _ => Err(DeserializeError::InvalidType),
        }
    }
}
