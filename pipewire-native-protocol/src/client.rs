use std::collections::HashMap;

use spa::serialize::PodSerializer;
use spa_derive::{PodDeserialize, PodSerialize};

// === Methods ===
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Error {
    pub id: i32,
    pub res: i32,
    pub error: String,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct UpdateProperties {
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct GetPermissions {
    pub index: i32,
    pub num: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
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
pub struct Info {
    pub id: i32,
    pub change_mask: i64,
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Permissions {
    pub index: i32,
    pub permissions: HashMap<i32, i32>,
}
