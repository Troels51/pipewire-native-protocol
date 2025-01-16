use std::{ collections::HashMap, sync::{Arc, Mutex}};

use spa::{
    deserialize::{DeserializeError, PodDeserializer}, opcode::{self, MessageOpCode}, serialize::PodSerializer
};
use spa_derive::{ PodDeserialize, PodSerialize};
use tokio::io;

use crate::{InnerConnection};

pub const CLIENT_ID: u32 = 1;

// Proxy
pub struct ClientProxy {
    connection: Arc<Mutex<InnerConnection>>,
}

impl ClientProxy {
    pub(crate) fn new(connection: Arc<Mutex<InnerConnection>>) -> ClientProxy{
        ClientProxy {connection}
    }

    pub async fn update_properties(&self) -> io::Result<()> {
        self.connection.lock().unwrap().call_method(
            CLIENT_ID,
            2,
            UpdateProperties {
                props: HashMap::from([("application.name".into(), "pipewirersssss".into())]),
            },
        )
        .await
    }
    pub async fn get_registry(&self) -> io::Result<()> {
        // self.connection.lock().unwrap().call_method(CLIENT_ID, opcode, payload)
        Ok(())
    }
}

// === Methods ===
#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(1)]
pub struct Error {
    pub id: i32,
    pub res: i32,
    pub error: String,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(2)]
pub struct UpdateProperties {
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(3)]
pub struct GetPermissions {
    pub index: i32,
    pub num: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(4)]
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
#[spa_derive::opcode(0)]
pub struct Info {
    pub id: i32,
    pub change_mask: i64,
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[spa_derive::opcode(1)]
pub struct Permissions {
    pub index: i32,
    pub permissions: HashMap<i32, i32>,
}

impl opcode::DeserializeFromOpCode for ClientEvent {

    fn deserialize_from_opcode<'de>(
        opcode: u32,
        buffer: &'de [u8],
    ) -> Result<(&'de [u8], Self), spa::deserialize::DeserializeError<&'de [u8]>>
    where
        Self: Sized,
    {
        match opcode {
            Info::OP_CODE => {
                let (remain, value): (&[u8], Info) = PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, ClientEvent::Info(value)))
            }
            Permissions::OP_CODE => {
                let (remain, value): (&[u8], Permissions) =
                    PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, ClientEvent::Permissions(value)))
            }
            _ => Err(DeserializeError::InvalidType),
        }
    }
}
