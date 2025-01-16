use crate::deserialize;


pub trait DeserializeFromOpCode {
    // Todo: Create an error code that covers both deserialize and unknow opcode
    fn deserialize_from_opcode(
        opcode: u32,
        buffer: &[u8],
    ) -> Result<(&[u8], Self), deserialize::DeserializeError<&[u8]>>
    where
        Self: Sized;
}
// Implemented by messages to specify their method/event opcode
// Used when deserializing messages into events
// Use #[spa_derive::opcode(<opcode>)] to define the opcode on structs
// TODO(maybe): It could potentially be useful to split this into EventOpcode/MethodOpcode, there are some method/events that are the same struct by the opcode differs depending on wether it is an event or method
pub trait MessageOpCode {
    const OP_CODE: u32;
}