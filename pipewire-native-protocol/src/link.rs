use std::collections::HashMap;

use spa::{
    serialize::{PodSerializer},
};
use spa_derive::{PodDeserialize, PodSerialize};
// === Methods ===

// Link has no methods

// === Events ===
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Info {
    pub id: i32,
    pub output_node_id: i32,
    pub output_port_id: i32,
    pub input_node_id: i32,
    pub input_port_id: i32,
    pub change_mask: i64,
    pub state: i32,
    pub error: String,
    pub format: spa::value::Value,
    pub props: HashMap<String, String>,
}
