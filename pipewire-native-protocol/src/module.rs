use std::collections::HashMap;

use spa::serialize::PodSerializer;
use spa_derive::{PodDeserialize, PodSerialize};
// === Methods ===

// Module has no methods

// === Events ===
pub enum ModuleEvent{
    Info(Info)
}
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Info {
    pub id: i32,
    pub name: String,
    pub file_name: String,
    pub args: String,
    pub change_mask: i64,
    pub props: HashMap<String, String>,
}
