use std::collections::HashMap;

use spa::{
    deserialize::{DeserializeError, PodDeserialize, PodDeserializer},
    serialize::{PodSerialize, PodSerializer},
    value::{Fd, Id, Value},
    CanonicalFixedSizedPod,
};
use spa_derive::{PodDeserialize, PodSerialize};

use crate::opcode;

// Methods and event structs
// ==== Core ====
// ==== pub Corepub ::Methods ====
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Hello {
    pub version: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Sync {
    pub id: i32,
    pub seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Pong {
    pub id: i32,
    pub seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct GetRegistry {
    pub version: i32,
    pub new_id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct CreateObject {
    pub factory_name: String,
    pub type_: String,
    pub version: i32,
    pub props: HashMap<String, String>,
    pub new_id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Destroy {
    pub id: i32,
}
// ==== pub Corepub ::Events ====
#[derive(Debug)]
pub enum CoreEvent {
    Info(Info),
    Done(Done),
    Ping(Ping),
    Error(Error),
    RemoveId(RemoveId),
    BoundId(BoundId),
    AddMem(AddMem),
    RemoveMem(RemoveMem),
    BoundProps(BoundProps),
}
impl opcode::OpCode for CoreEvent {
    fn opcode(&self) -> u32 {
        match self {
            CoreEvent::Info(_) => 0,
            CoreEvent::Done(_) => 1,
            CoreEvent::Ping(_) => 2,
            CoreEvent::Error(_) => 3,
            CoreEvent::RemoveId(_) => 4,
            CoreEvent::BoundId(_) => 5,
            CoreEvent::AddMem(_) => 6,
            CoreEvent::RemoveMem(_) => 7,
            CoreEvent::BoundProps(_) => 8,
        }
    }

    fn deserialize_from_opcode<'de>(
        opcode: u32,
        buffer: &'de [u8],
    ) -> Result<(&'de [u8], Self), spa::deserialize::DeserializeError<&'de [u8]>>
    where
        Self: Sized,
    {
        match opcode {
            0 => {
                let (remain, value): (&[u8], Info) = PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, CoreEvent::Info(value)))
            }
            1 => {
                let (remain, value): (&[u8], Done) = PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, CoreEvent::Done(value)))
            }
            2 => {
                let (remain, value): (&[u8], Ping) = PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, CoreEvent::Ping(value)))
            }
            3 => {
                let (remain, value): (&[u8], Error) = PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, CoreEvent::Error(value)))
            }
            4 => {
                let (remain, value): (&[u8], RemoveId) =
                    PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, CoreEvent::RemoveId(value)))
            }
            5 => {
                let (remain, value): (&[u8], BoundId) = PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, CoreEvent::BoundId(value)))
            }
            6 => {
                let (remain, value): (&[u8], AddMem) = PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, CoreEvent::AddMem(value)))
            }
            7 => {
                let (remain, value): (&[u8], RemoveMem) =
                    PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, CoreEvent::RemoveMem(value)))
            }
            8 => {
                let (remain, value): (&[u8], BoundProps) =
                    PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, CoreEvent::BoundProps(value)))
            }
            _ => Err(DeserializeError::InvalidType),
        }
    }
}

#[derive(PodSerialize, PodDeserialize, Debug)]
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
pub struct Done {
    pub id: i32,
    pub seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Ping {
    pub id: i32,
    pub seq: i32,
}

// Both event and method
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Error {
    pub id: i32,
    pub seq: i32,
    pub res: i32,
    pub message: String,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct RemoveId {
    pub id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct BoundId {
    pub id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct AddMem {
    pub id: i32,
    pub type_: Id,
    pub fd: Fd,
    pub flags: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct RemoveMem {
    pub id: i32,
}
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct BoundProps {
    pub id: i32,
    pub global_id: i32,
    pub props: HashMap<String, String>,
}
