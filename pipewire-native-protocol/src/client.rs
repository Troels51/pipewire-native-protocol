use std::collections::HashMap;

use spa::{
    deserialize::{DeserializeError, PodDeserializer},
    serialize::PodSerializer,
};
use spa_derive::{PodDeserialize, PodSerialize};

use crate::opcode;

// === Methods ===
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Error {
    pub id: i32,
    pub res: i32,
    pub error: String,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct UpdateProperties {
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct GetPermissions {
    pub index: i32,
    pub num: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct UpdatePermissions {
    pub index: i32,
    pub num: i32,
}

// === Events ===
#[derive(Debug)]
pub enum ClientEvent {
    Info(Info),
    Permissions(Permissions),
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Info {
    pub id: i32,
    pub change_mask: i64,
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Permissions {
    pub index: i32,
    pub permissions: HashMap<i32, i32>,
}

impl opcode::OpCode for ClientEvent {
    fn opcode(&self) -> u32 {
        match self {
            ClientEvent::Info(_) => 0,
            ClientEvent::Permissions(_) => 1,
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
                Ok((remain, ClientEvent::Info(value)))
            }
            1 => {
                let (remain, value): (&[u8], Permissions) =
                    PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, ClientEvent::Permissions(value)))
            }
            _ => Err(DeserializeError::InvalidType),
        }
    }
}
