use std::{collections::HashMap, sync::{Arc, Mutex}};

use spa::serialize::PodSerializer;
use spa_derive::{PodDeserialize, PodSerialize};

use crate::PipewireWriter;

pub struct FactoryProxy {
    connection: Arc<Mutex<PipewireWriter>>,
    event_receiver: std::sync::mpsc::Receiver<FactoryEvent>
}

impl FactoryProxy {
    pub(crate) fn new(connection: Arc<Mutex<PipewireWriter>>, event_receiver: std::sync::mpsc::Receiver<FactoryEvent>) -> FactoryProxy{
        FactoryProxy {connection, event_receiver }
    }
}

// === Methods ===

// Factory has no methods

// === Events ===
pub enum FactoryEvent {
    FactoryEvent(Info)
}
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Info {
    pub id: i32,
    pub name: String,
    pub type_: String,
    pub version: i32,
    pub change_mask: i64,
    pub props: HashMap<String, String>,
}
