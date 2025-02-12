use std::{ collections::HashMap, ops::{Deref, DerefMut}, sync::Arc};

use spa::{
    deserialize::{DeserializeError, PodDeserializer}, opcode::{self, MessageOpCode}, serialize::PodSerializer
};
use spa_derive::{ PodDeserialize, PodSerialize};
use tokio::{io, sync::Mutex};

use crate::{PipewireWriter};


// Proxy
pub struct ClientProxy {
    connection: Arc<Mutex<PipewireWriter>>,
    event_receiver: tokio::sync::mpsc::Receiver<ClientEvent>
}

impl ClientProxy {
    pub const CLIENT_ID: i32 = 1;

    pub(crate) fn new(connection: Arc<Mutex<PipewireWriter>>, event_receiver: tokio::sync::mpsc::Receiver<ClientEvent>) -> ClientProxy{
        ClientProxy {connection, event_receiver }
    }

    pub async fn update_properties(&self) -> io::Result<()> {
        self.connection.lock().await.call_method(
            ClientProxy::CLIENT_ID,
            2,
            UpdateProperties {
                props: HashMap::from([("application.name".into(), "pipewirersssss".into())]),
            },
        )
        .await
    }
}

impl Deref for ClientProxy {
    type Target = tokio::sync::mpsc::Receiver<ClientEvent>;

    fn deref(&self) -> &Self::Target {
        &self.event_receiver
    }
}
impl DerefMut for ClientProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event_receiver
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
