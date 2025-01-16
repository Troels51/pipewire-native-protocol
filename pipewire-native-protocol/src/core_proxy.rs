use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use spa::{
    deserialize::{DeserializeError, PodDeserializer}, opcode::MessageOpCode, serialize::PodSerializer, value::{Fd, Id}
};
use spa_derive::{PodDeserialize, PodSerialize};

use crate::InnerConnection;

pub const CORE_ID: u32 = 0;

// Proxy
pub struct CoreProxy {
    connection: Arc<Mutex<InnerConnection>>,
}

impl CoreProxy {
    pub(crate) fn new(connection: Arc<Mutex<InnerConnection>>) -> CoreProxy {
        CoreProxy { connection }
    }
    pub async fn hello(&mut self) -> Result<(), std::io::Error> {
        self.connection
            .lock()
            .unwrap()
            .call_method(CORE_ID, 1, Hello { version: 3 })
            .await
    }
}

// Methods and event structs
// ==== Core ====
// ==== pub Corepub ::Methods ====
#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(1)]
pub struct Hello {
    pub version: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(2)]
pub struct Sync {
    pub id: i32,
    pub seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(3)]
pub struct Pong {
    pub id: i32,
    pub seq: i32,
}

// There is both an event and a error method, they are the same except the opcode
#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(4)]
pub struct ErrorMethod {
    pub id: i32,
    pub seq: i32,
    pub res: i32,
    pub message: String,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(5)]
pub struct GetRegistry {
    pub version: i32,
    pub new_id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(6)]
pub struct CreateObject {
    pub factory_name: String,
    pub type_: String,
    pub version: i32,
    pub props: HashMap<String, String>,
    pub new_id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(7)]
pub struct Destroy {
    pub id: i32,
}
// ==== pub Corepub ::Events ====
#[derive(Debug)]
pub enum CoreEvent {
    Info(Info),
    Done(Done),
    Ping(Ping),
    Error(ErrorEvent),
    RemoveId(RemoveId),
    BoundId(BoundId),
    AddMem(AddMem),
    RemoveMem(RemoveMem),
    BoundProps(BoundProps),
}
impl spa::opcode::DeserializeFromOpCode for CoreEvent {
    fn deserialize_from_opcode(
        opcode: u32,
        buffer: &[u8],
    ) -> Result<(&[u8], Self), spa::deserialize::DeserializeError<&[u8]>>
    where
        Self: Sized,
    {
        match opcode {
            Info::OP_CODE => {
                let (remain, value): (&[u8], Info) = PodDeserializer::deserialize_from(buffer)?;
                Ok((remain, CoreEvent::Info(value)))
            }
            Done::OP_CODE => {
                let (remain, value): (&[u8], Done) = PodDeserializer::deserialize_from(buffer)?;
                Ok((remain, CoreEvent::Done(value)))
            }
            Ping::OP_CODE => {
                let (remain, value): (&[u8], Ping) = PodDeserializer::deserialize_from(buffer)?;
                Ok((remain, CoreEvent::Ping(value)))
            }
            ErrorEvent::OP_CODE => {
                let (remain, value): (&[u8], ErrorEvent) =
                    PodDeserializer::deserialize_from(buffer)?;
                Ok((remain, CoreEvent::Error(value)))
            }
            RemoveId::OP_CODE => {
                let (remain, value): (&[u8], RemoveId) = PodDeserializer::deserialize_from(buffer)?;
                Ok((remain, CoreEvent::RemoveId(value)))
            }
            BoundId::OP_CODE => {
                let (remain, value): (&[u8], BoundId) = PodDeserializer::deserialize_from(buffer)?;
                Ok((remain, CoreEvent::BoundId(value)))
            }
            AddMem::OP_CODE => {
                let (remain, value): (&[u8], AddMem) = PodDeserializer::deserialize_from(buffer)?;
                Ok((remain, CoreEvent::AddMem(value)))
            }
            RemoveMem::OP_CODE => {
                let (remain, value): (&[u8], RemoveMem) =
                    PodDeserializer::deserialize_from(buffer)?;
                Ok((remain, CoreEvent::RemoveMem(value)))
            }
            BoundProps::OP_CODE => {
                let (remain, value): (&[u8], BoundProps) =
                    PodDeserializer::deserialize_from(buffer)?;
                Ok((remain, CoreEvent::BoundProps(value)))
            }
            _ => Err(DeserializeError::InvalidType),
        }
    }
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(0)]
pub struct Info {
    pub id: i32,
    pub cookie: i32,
    pub user_name: String,
    pub host_name: String,
    pub version: String,
    pub name: String,
    pub change_mask: i64,
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(1)]
pub struct Done {
    pub id: i32,
    pub seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(2)]
pub struct Ping {
    pub id: i32,
    pub seq: i32,
}

// There is both an event and a error method, they are the same except the opcode
#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(3)]
pub struct ErrorEvent {
    pub id: i32,
    pub seq: i32,
    pub res: i32,
    pub message: String,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(4)]
pub struct RemoveId {
    pub id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(5)]
pub struct BoundId {
    pub id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(6)]
pub struct AddMem {
    pub id: i32,
    pub type_: Id,
    pub fd: Fd,
    pub flags: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(7)]
pub struct RemoveMem {
    pub id: i32,
}
#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(8)]
pub struct BoundProps {
    pub id: i32,
    pub global_id: i32,
    pub props: HashMap<String, String>,
}
