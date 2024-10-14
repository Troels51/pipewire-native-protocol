use std::collections::HashMap;

use spa::{
    serialize::{PodSerializer},
    value::{Fd, Id},
};
use spa_derive::{PodDeserialize, PodSerialize};

// Methods and event structs
// ==== Core ====
// ==== pub Corepub ::Methods ====
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Hello {
    pub version: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Sync {
    pub id: i32,
    pub seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Pong {
    pub id: i32,
    pub seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct GetRegistry {
    pub version: i32,
    pub new_id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct CreateObject {
    pub factory_name: String,
    pub type_: String,
    pub version: i32,
    pub props: HashMap<String, String>,
    pub new_id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Destroy {
    pub id: i32,
}
// ==== pub Corepub ::Events ====
#[derive(PodSerialize, PodDeserialize, Debug)]
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

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Done {
    pub id: i32,
    pub seq: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Ping {
    pub id: i32,
    pub seq: i32,
}

// Both event and method
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Error {
    pub id: i32,
    pub seq: i32,
    pub res: i32,
    pub message: String,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct RemoveId {
    pub id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct BoundId {
    pub id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct AddMem {
    pub id: i32,
    pub type_: Id,
    pub fd: Fd,
    pub flags: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct RemoveMem {
    pub id: i32,
}
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct BoundProps {
    pub id: i32,
    pub global_id: i32,
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct GetPermissions {
    pub index: i32,
    pub num: i32,
}
