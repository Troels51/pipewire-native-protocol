use std::collections::HashMap;

use spa::serialize::PodSerializer;
use spa_derive::{PodDeserialize, PodSerialize};
// === Methods ===
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct SubscribeParams {
    pub ids: Vec<spa::value::Id>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct EnumParams {
    pub seq: i32,
    pub id: spa::value::Id,
    pub index: i32,
    pub num: i32,
    pub filter: spa::value::Value,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct SetParam {
    pub id: spa::value::Id,
    pub flags: i32,
    pub param: spa::value::Value,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct SendCommand {
    pub command: spa::value::Value,
}

// === Events ===
pub enum NodeEvent {
    Info(Info),
    Param(Param)
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Info {
    pub id: i32,
    pub max_input_ports: i32,
    pub max_output_ports: i32,
    pub change_mask: i64,
    pub n_input_ports: i32,
    pub n_output_ports: i32,
    pub state: spa::value::Id, // Can this really be true?
    pub error: String,
    pub props: HashMap<String, String>,
    pub param_info: HashMap<i32, i32>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Param {
    pub seq: i32,
    pub id: spa::value::Id,
    pub index: i32,
    pub next: i32,
    pub param: spa::value::Value,
}
