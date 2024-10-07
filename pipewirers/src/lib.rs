use core::panic;
use std::{
    collections::HashMap,
    io::{Cursor, IoSlice},
};

use spa::{
    deserialize::{PodDeserialize, PodDeserializer},
    serialize::{PodSerialize, PodSerializer},
    value::Value,
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
        client.seq += 1;
        let _ = client.update_properties().await;
        dbg!("Update properties");
        client.seq += 1;
        // let _ = client.get_registry().await;
        dbg!("Get registry");
        client.seq += 1;

        let mut header = Header::new_zeroed();
        let mut header = [0u8; 16];
        let _ = client.stream.readable().await;
        println!("Readable");
        let bytes_read = client.stream.read_exact(header.as_bytes_mut()).await?; // Yes, Yes, unsafe
        println!("bytes: {}", bytes_read);
        // let mut buffer = Vec::<u8>::with_capacity(header.size());
        // let _ = client.stream.read(&mut buffer).await;
        // let value: Value = match PodDeserializer::deserialize_from(&buffer) {
        //     Ok((remain, value)) => value,
        //     Err(e) => {
        //         println!("Could not parse message");
        //         panic!("Not handled yet");
        //     }
        // };
        // println!("Got value {:?}", value);
        Ok(client)
    }

    async fn call_method(
        &mut self,
        id: u32,
        opcode: u8,
        payload: impl PodSerialize,
    ) -> io::Result<()> {
        let mut message = Message::new(id, opcode, self.seq, payload);
        self.stream.writable().await?;
        let size = message.write(&mut self.stream).await?;
        dbg!(self.stream.take_error());
        Ok(())
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
                props: HashMap::from([("application.name".into(), "pipewirersrrsrs".into())]),
            },
        )
        .await
    }

    async fn get_permissions(&mut self) -> io::Result<()> {
        let payload = GetPermissions { index: 0, num: 3 };
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

#[derive(AsBytes, FromBytes, FromZeroes)]
#[repr(C)]
struct Header {
    id: u32,
    opcode_size: u32,
    seq: u32,
    n_fds: u32,
}

impl Header {
    fn new(id: u32, opcode: u8, size: u32, seq: u32, n_fds: u32) -> Self {
        Self {
            id,
            opcode_size: (size << 8) + opcode as u32,
            seq,
            n_fds,
        }
    }
    fn incomplete(id: u32, opcode: u8, seq: u32) -> Self {
        Self::new(id, opcode, 0, seq, 0)
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
    fn new(id: u32, opcode: u8, seq: u32, payload: T) -> Self {
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
        self.header.opcode_size = (buffer.len() << 8) as u32 + self.header.opcode_size;
        out.write(self.header.as_bytes()).await?;
        out.write(&buffer).await
    }
}
#[derive(PodSerialize, PodDeserialize)]
struct Hello {
    version: i32,
}

#[derive(PodSerialize, PodDeserialize)]
struct UpdateProperties {
    props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize)]
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

#[derive(PodSerialize, PodDeserialize)]
struct GetRegistry {
    version: i32,
    new_id: i32,
}

#[derive(PodSerialize, PodDeserialize)]
struct GetPermissions {
    index: i32,
    num: i32,
}
