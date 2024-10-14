
use spa::{
    serialize::{PodSerializer},
};
use spa_derive::{PodDeserialize, PodSerialize};
// === Methods ===

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct SetProperty {
    pub subject: i32,
    pub key: String,
    pub type_: String,
    pub value: String,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Clear {
    none: spa::value::Value, // Should always be spa::value::Value::None
}
impl Default for Clear {
    fn default() -> Self {
        Self {
            none: spa::value::Value::None,
        }
    }
}

// === Events ===
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Property {
    pub subject: i32,
    pub key: String,
    pub type_: String,
    pub value: String,
}
