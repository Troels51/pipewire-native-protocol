use std::{collections::HashMap, ops::{Deref, DerefMut}, sync::Arc};

use spa::{deserialize::{DeserializeError, PodDeserializer}, opcode::{self, MessageOpCode}, serialize::PodSerializer};
use spa_derive::{opcode, PodDeserialize, PodSerialize};
use tokio::sync::Mutex;

use crate::{ core_proxy, proxy::Proxy, PipewireWriter};

pub struct RegistryProxy {
    id: i32,
    connection: Arc<Mutex<PipewireWriter>>,
    event_receiver: tokio::sync::mpsc::Receiver<RegistryEvent>,
}

impl RegistryProxy {
    pub(crate) const VERSION: i32 = 3; // Version of the registry interface used
    pub(crate) fn new(id: i32, connection: Arc<Mutex<PipewireWriter>>, event_receiver: tokio::sync::mpsc::Receiver<RegistryEvent>) -> RegistryProxy{
        RegistryProxy {id, connection, event_receiver }
    }
}

impl Proxy for RegistryProxy {
    type Event = RegistryEvent;

    fn id(&self) -> i32 {
        self.id
    }

    fn get_channel(&mut self) ->  &mut tokio::sync::mpsc::Receiver<Self::Event> {
        &mut self.event_receiver
    }

    fn get_connection(&self) -> std::sync::Arc<tokio::sync::Mutex<crate::PipewireWriter>> {
        self.connection.clone()
    }
}

impl Deref for RegistryProxy {
    type Target = tokio::sync::mpsc::Receiver<RegistryEvent>;

    fn deref(&self) -> &Self::Target {
        &self.event_receiver
    }
}

impl DerefMut for RegistryProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event_receiver
    }
}

// === Methods ===

#[derive(PodSerialize, PodDeserialize, Debug)]
#[opcode(1)]
pub struct Bind {
    pub id: i32,
    pub type_: String,
    pub version: i32,
    pub new_id: i32,
}
#[derive(PodSerialize, PodDeserialize, Debug)]
#[opcode(2)]
pub struct Destroy {
    pub id: i32,
}

// === Events ===
#[derive(Debug)]
pub enum RegistryEvent {
    Global(Global),
    GlobalRemove(GlobalRemove),
    // Added to allow for receiving Done events on all proxies
    Done(core_proxy::Done)
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[opcode(0)]
pub struct Global {
    pub id: i32,
    pub permissions: i32,
    pub type_: String,
    pub version: i32,
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
#[opcode(1)]
pub struct GlobalRemove {
    pub id: i32,
}


impl opcode::DeserializeFromOpCode for RegistryEvent {

    fn deserialize_from_opcode<'de>(
        opcode: u32,
        buffer: &'de [u8],
    ) -> Result<(&'de [u8], Self), spa::deserialize::DeserializeError<&'de [u8]>>
    where
        Self: Sized,
    {
        match opcode {
            Global::OP_CODE => {
                let (remain, value): (&[u8], Global) = PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, RegistryEvent::Global(value)))
            }
            GlobalRemove::OP_CODE => {
                let (remain, value): (&[u8], GlobalRemove) =
                    PodDeserializer::deserialize_from(&buffer)?;
                Ok((remain, RegistryEvent::GlobalRemove(value)))
            }
            _ => Err(DeserializeError::InvalidType),
        }
    }
}

impl core::fmt::Display for RegistryEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryEvent::Global(global) => {
                write!(f, "Id {}, type {}\n", global.id, global.type_)?;
                for (key, value) in global.props.iter() {
                    write!(f, "\t{} {} \n", key, value)?;
                }
                write!(f, "")
            },
            RegistryEvent::GlobalRemove(global_remove) => Ok(()),
            RegistryEvent::Done(done) => Ok(()),
        }
    }
}