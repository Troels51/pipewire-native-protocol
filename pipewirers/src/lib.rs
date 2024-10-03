use std::io::{Cursor, IoSlice};

use spa::{
    serialize::{PodSerialize, PodSerializer},
    value::Value,
};
use tokio::{
    io::{self, AsyncWriteExt},
    net::unix::{OwnedReadHalf, OwnedWriteHalf},
};
use zerocopy::AsBytes;
pub struct PipewireClient {
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
    seq: u32,
}

impl PipewireClient {
    pub async fn connect(stream: tokio::net::UnixStream) -> Self {
        let (reader, writer) = stream.into_split();
        let mut client = Self {
            reader,
            writer,
            seq: 0,
        };

        let _ = client.hello(3).await;
        client.seq += 1;
        let _ = client.update_properties().await;
        client.seq += 1;
        client
    }

    async fn hello(&mut self, version: i32) -> io::Result<()> {
        let payload = Value::Struct(vec![Value::Int(version)]);
        let mut message = Message::new(0, 1, self.seq, payload);
        message.write(&mut self.writer).await?;
        Ok(())
    }

    async fn update_properties(&mut self) -> io::Result<()> {
        let payload = Value::Struct(vec![Value::Struct(vec![Value::Int(0)])]);
        let mut message = Message::new(0, 1, self.seq, payload);
        message.write(&mut self.writer).await?;
        Ok(())
    }
}

#[derive(AsBytes)]
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

struct Message {
    header: Header,
    payload: Value,
}

impl Message {
    fn new(id: u32, opcode: u8, seq: u32, payload: Value) -> Self {
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
        out.write_vectored(&[IoSlice::new(self.header.as_bytes()), IoSlice::new(&buffer)])
            .await
    }
}
