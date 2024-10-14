use std::collections::HashMap;

use spa::{
    serialize::{PodSerializer},
};
use spa_derive::{PodDeserialize, PodSerialize};
// === Methods ===

// Factory has no methods

// === Events ===
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Info {
    pub id: i32,
    pub name: String,
    pub type_: String,
    pub version: i32,
    pub change_mask: i64,
    pub props: HashMap<String, String>,
}
