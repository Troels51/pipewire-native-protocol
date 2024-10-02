pub struct PipewireClient {
    stream: tokio::net::UnixStream,
    seq: u32,
}

impl PipewireClient {
    pub async fn connect(stream: tokio::net::UnixStream) -> Self {
        let mut client = Self { stream, seq: 0 };

        client.hello(3).await;
        client.seq += 1;
        client.update_properties().await;
        client
    }

    async fn hello(&self, version: i32) {
        // let message = HelloMessage::new(version, self.seq);
        // self.stream.try_write(message.into());
    }

    async fn update_properties(&self) {
        todo!()
    }
}

// Messages
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
            opcode_size: size << 24 + opcode,
            seq,
            n_fds,
        }
    }
}

struct Message {
    header: Header,
    payload: spa::value::Value,
}

// #[repr(C)]
// #[derive(AsBytes)]
// struct HelloMessage {
//     header: Header,
//     payload_pod: spa_pods::Int, // No, It is struct of int
// }

// impl HelloMessage {
//     fn new(version: i32, seq: u32) -> Self {
//         let payload = spa_pods::Int::new(version);
//         let header = Header::new(
//             0,
//             1,
//             core::mem::size_of_val::<spa_pods::Int>(&payload)
//                 .try_into()
//                 .unwrap(),
//             seq,
//             0,
//         );
//         Self {
//             header: header,
//             payload_pod: payload,
//         }
//     }
// }

// #[repr(C)]
// #[derive(AsBytes)]
// struct UpdatePropertiesMessage {
//     header: Header,
//     payload_pod: spa_pods::Int,
// }
// trait SPAPod {

// }
