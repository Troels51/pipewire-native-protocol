use std::collections::HashMap;

use spa::{
    serialize::{PodSerializer},
};
use spa_derive::{PodDeserialize, PodSerialize};
// === Methods ===

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Bind {
    pub id: i32,
    pub type_: String,
    pub version: i32,
    pub new_id: i32,
}
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Destroy {
    pub id: i32,
}

// === Events ===
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Global {
    pub id: i32,
    pub permissions: i32,
    pub type_: String,
    pub version: i32,
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct GlobalRemove {
    pub id: i32,
}
