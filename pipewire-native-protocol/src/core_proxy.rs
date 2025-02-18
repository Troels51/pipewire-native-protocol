use std::{
    collections::HashMap, io::Error, ops::{Deref, DerefMut}, sync::Arc
};

use spa::{
    deserialize::{DeserializeError, PodDeserializer},
    opcode::MessageOpCode,
    serialize::PodSerializer,
    value::{Fd, Id},
};
use spa_derive::{PodDeserialize, PodSerialize};
use tokio::sync::Mutex;

use crate::{
    registry::{self, RegistryProxy},
    PipewireWriter, Proxies,
};

pub const CORE_ID: i32 = 0;

// Proxy
pub struct CoreProxy {
    connection: Arc<Mutex<PipewireWriter>>,
    event_channel: tokio::sync::mpsc::Receiver<CoreEvent>,
    proxies: Arc<Mutex<Proxies>>,
}

impl CoreProxy {
    pub(crate) async fn new(
        connection: Arc<Mutex<PipewireWriter>>,
        event_channel: tokio::sync::mpsc::Receiver<CoreEvent>,
        proxies: Arc<Mutex<Proxies>>,
    ) -> std::io::Result<CoreProxy> {
        let mut this = CoreProxy {
            connection,
            event_channel,
            proxies,
        };
        this.hello().await?;
        Ok(this)
    }
    async fn hello(&mut self) -> Result<(), std::io::Error> {
        self.connection
            .lock()
            .await
            .call_method(CORE_ID, Hello::OP_CODE, Hello { version: 3 })
            .await
    }

    pub async fn sync(&mut self, id: i32) ->Result<(), std::io::Error>
    {
        let mut connection = self.connection.lock().await;
        let seq = connection.seq + 1;
        connection.call_method(CORE_ID, Sync::OP_CODE, Sync {
            id: id,
            seq: seq,
        }).await
    }

    pub async fn get_registry(&mut self) -> std::io::Result<RegistryProxy> {
        let mut id = 0;
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        {
            let mut connection = self.connection.lock().await;
            let mut proxies = self.proxies.lock().await;
            proxies.id_counter = proxies.id_counter + 1;
            id = proxies.id_counter;
            proxies.registry_proxies.insert(id, sender);
            connection
                .call_method(
                    CORE_ID,
                    GetRegistry::OP_CODE,
                    GetRegistry {
                        version: RegistryProxy::VERSION,
                        new_id: id,
                    },
                )
                .await?;
        }

        Ok(registry::RegistryProxy::new(
            id,
            self.connection.clone(),
            receiver,
        ))
    }
}
impl Deref for CoreProxy {
    type Target = tokio::sync::mpsc::Receiver<CoreEvent>;

    fn deref(&self) -> &Self::Target {
        &self.event_channel
    }
}
impl DerefMut for CoreProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event_channel
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
#[derive(Debug, Clone)]
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

#[derive(PodSerialize, PodDeserialize, Debug, Clone)]
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

#[derive(PodSerialize, PodDeserialize, Debug, Clone)]
#[spa_derive::opcode(1)]
pub struct Done {
    pub id: i32,
    pub seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug, Clone)]
#[spa_derive::opcode(2)]
pub struct Ping {
    pub id: i32,
    pub seq: i32,
}

// There is both an event and a error method, they are the same except the opcode
#[derive(PodSerialize, PodDeserialize, Debug, Clone)]
#[spa_derive::opcode(3)]
pub struct ErrorEvent {
    pub id: i32,
    pub seq: i32,
    pub res: i32,
    pub message: String,
}

#[derive(PodSerialize, PodDeserialize, Debug, Clone)]
#[spa_derive::opcode(4)]
pub struct RemoveId {
    pub id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug, Clone)]
#[spa_derive::opcode(5)]
pub struct BoundId {
    pub id: i32,
    pub global_id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug, Clone)]
#[spa_derive::opcode(6)]
pub struct AddMem {
    pub id: i32,
    pub type_: Id,
    pub fd: Fd,
    pub flags: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug, Clone)]
#[spa_derive::opcode(7)]
pub struct RemoveMem {
    pub id: i32,
}
#[derive(PodSerialize, PodDeserialize, Debug, Clone)]
#[spa_derive::opcode(8)]
pub struct BoundProps {
    pub id: i32,
    pub global_id: i32,
    pub props: HashMap<String, String>,
}
