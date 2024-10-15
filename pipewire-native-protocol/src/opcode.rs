use spa::deserialize::PodDeserialize;

pub(crate) trait OpCode {
    fn opcode(&self) -> u32;
    // Todo: Create an error code that covers both deserialize and unknow opcode
    fn deserialize_from_opcode<'de>(
        opcode: u32,
        buffer: &'de [u8],
    ) -> Result<(&'de [u8], Self), spa::deserialize::DeserializeError<&'de [u8]>>
    where
        Self: Sized;
}
