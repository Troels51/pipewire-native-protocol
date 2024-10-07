use core::panic;
use std::{
    collections::HashMap,
    io::{Cursor, IoSlice},
};

use spa::{
    deserialize::{PodDeserialize, PodDeserializer},
    serialize::{PodSerialize, PodSerializer},
    value::{Fd, Id, Value},
    CanonicalFixedSizedPod,
};
use spa_derive::{PodDeserialize, PodSerialize};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::unix::{OwnedReadHalf, OwnedWriteHalf},
};
use zerocopy::{AsBytes, FromBytes, FromZeroes};
pub struct PipewireClient {
    stream: tokio::net::UnixStream,
    seq: u32,
}

impl PipewireClient {
    const CORE_ID: u32 = 0;
    const CLIENT_ID: u32 = 1;

    pub async fn connect(stream: tokio::net::UnixStream) -> io::Result<Self> {
        let mut client = Self { stream, seq: 2 };

        client.hello(3).await?;
        let _ = client.update_properties().await;
        // let _ = client.get_registry().await;
        let value: Info = client.read().await?;
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

    async fn read<T: for<'a> PodDeserialize<'a>>(&mut self) -> io::Result<T> {
        let mut header = Header::new_zeroed();
        let bytes_read = self.stream.read_exact(header.as_bytes_mut()).await?; // Yes, Yes, unsafe
        let mut buffer = vec![0; header.size()];
        let size = self.stream.read(&mut buffer).await;
        let value: T = match PodDeserializer::deserialize_from(&buffer) {
            Ok((remain, value)) => value,
            Err(e) => {
                panic!(
                    "Could not parse value {:?} of type {}",
                    e,
                    std::any::type_name::<T>()
                );
            }
        };
        Ok(value)
    }

    async fn hello(&mut self, version: i32) -> io::Result<()> {
        self.call_method(Self::CORE_ID, 1, Hello { version: version })
            .await
    }

    async fn update_properties(&mut self) -> io::Result<()> {
        self.call_method(
            Self::CLIENT_ID,
            2,
            UpdateProperties {
                props: HashMap::from([("application.name".into(), "pipewirers".into())]),
            },
        )
        .await
    }

    async fn get_permissions(&mut self) -> io::Result<()> {
        let payload = GetPermissions { index: 0, num: 3 }; // TODO: Real values
        let mut message = Message::new(Self::CLIENT_ID, 3, self.seq, payload);
        let _ = self.stream.writable().await;
        message.write(&mut self.stream).await?;
        Ok(())
    }

    async fn get_registry(&mut self) -> io::Result<()> {
        let payload = GetRegistry {
            version: 3,
            new_id: 500,
        };
        let mut message = Message::new(Self::CORE_ID, 5, self.seq, payload);
        let _ = self.stream.writable().await;
        message.write(&mut self.stream).await?;
        Ok(())
    }
}

#[derive(AsBytes, FromBytes, FromZeroes, Debug)]
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

enum Event {
    //Core
    Info(Info),
    Done(Done),
}
// Methods and event structs
// ==== Core ====
// ==== Core::Methods ====
#[derive(PodSerialize, PodDeserialize, Debug)]
struct Hello {
    version: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct Sync {
    id: i32,
    seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct Pong {
    id: i32,
    seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct GetRegistry {
    version: i32,
    new_id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct CreateObject {
    factory_name: String,
    type_: String,
    version: i32,
    props: HashMap<String, String>,
    new_id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct Destroy {
    id: i32,
}
// ==== Core::Events ====
#[derive(PodSerialize, PodDeserialize, Debug)]
struct Info {
    id: i32,
    cookie: i32,
    user_name: String,
    host_name: String,
    version: String,
    name: String,
    change_mask: i64,
    props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct Done {
    id: i32,
    seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct Ping {
    id: i32,
    seq: i32,
}

// Both event and method
#[derive(PodSerialize, PodDeserialize, Debug)]
struct Error {
    id: i32,
    seq: i32,
    res: i32,
    message: String,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct RemoveId {
    id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct BoundId {
    id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct AddMem {
    id: i32,
    type_: Id,
    fd: Fd,
    flags: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct RemoveMem {
    id: i32,
}
#[derive(PodSerialize, PodDeserialize, Debug)]
struct BoundProps {
    id: i32,
    global_id: i32,
    props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
struct GetPermissions {
    index: i32,
    num: i32,
}

// ==== Registry ====

// ==== Client ====
#[derive(PodSerialize, PodDeserialize, Debug)]
struct UpdateProperties {
    props: HashMap<String, String>,
}
