use std::{collections::HashMap, sync::{Arc, Mutex}};

use spa::{deserialize::{DeserializeError, PodDeserializer}, opcode::{self, MessageOpCode}, serialize::PodSerializer};
use spa_derive::{opcode, PodDeserialize, PodSerialize};

use crate::{PipewireWriter};

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