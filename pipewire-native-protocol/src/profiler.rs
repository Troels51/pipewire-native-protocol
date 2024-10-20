use spa::serialize::PodSerializer;
use spa_derive::{PodDeserialize, PodSerialize};
// === Methods ===

// No methods for profiler

// === Events ===
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Profile {
    pub object: spa::value::Value,
}
