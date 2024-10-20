use std::collections::HashMap;

use spa::serialize::PodSerializer;
use spa_derive::{PodDeserialize, PodSerialize};
// === Methods ===

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct GetNode {
    pub version: i32,
    pub new_id: i32,
}

#[derive(Debug)]
pub struct Update {
    pub change_mask: i32,
    pub params: Vec<spa::value::Value>, // This is implemented as a n_params, followed by a n spa values, why the hell is it that?
    pub info: crate::node::Info,
}

#[derive(Debug)]
pub struct PortUpdate {
    pub direction: i32,
    pub port_id: i32,
    pub change_mask: i32,
    pub params: Vec<spa::value::Value>, // This is implemented as a n_params, followed by a n spa values, why the hell is it that?
    pub info: crate::port::Info,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct SetActive {
    pub active: bool,
}
// Both a method and event
#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Event {
    pub event: spa::value::Value,
}
// DataPlane is the inner "struct" inside a buffer, but not a spa struct according to the spec, and also the Vec in Buffer and PortBuffer are not spa arrays according to spec
#[derive(Debug)]
pub struct DataPlane {
    pub type_: spa::value::Id,
    pub memfd: spa::value::Fd,
    pub flags: i32,
    pub mapoffset: i32,
    pub maxsize: i32,
}
#[derive(Debug)]
pub struct Buffer {
    pub data_planes: Vec<DataPlane>,
}

#[derive(Debug)]
pub struct PortBuffers {
    pub direction: i32,
    pub port_id: i32,
    pub mix_id: i32,
    pub buffers: Vec<Buffer>,
}

// === Events ===

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Transport {
    pub readfd: spa::value::Fd,
    pub write: spa::value::Fd,
    pub memfd: i32,
    pub offset: i32,
    pub size: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct SetParam {
    pub id: i32,
    pub flags: i32,
    pub param: spa::value::Value,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct SetIO {
    pub id: spa::value::Id,
    pub memid: i32,
    pub offset: i32,
    pub size: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct Command {
    pub command: spa::value::Value,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct AddPort {
    pub direction: i32,
    pub port_id: i32,
    pub props: HashMap<String, String>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct RemovePort {
    pub direction: i32,
    pub port_id: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct PortSetParam {
    pub direction: i32,
    pub port_id: i32,
    pub id: spa::value::Id,
    pub flags: i32,
    pub param: spa::value::Value,
}

// Not a real spa struct
pub struct Meta {
    pub type_: spa::value::Id,
    pub size: i32,
}
// Not a real spa struct
pub struct DataBlock {
    pub type_: spa::value::Id,
    pub data: i32,
    pub flags: i32,
    pub mapoffset: i32,
    pub maxsize: i32,
}
// Not a real spa struct
pub struct MixerBuffer {
    pub memid: i32,
    pub offset: i32,
    pub size: i32,
    pub flags: i32,
    pub metas: Vec<Meta>,            // Not a real spa array
    pub data_blocks: Vec<DataBlock>, // Not a real spa array
}
//TODO: Implement PodSerDe
pub struct UseBuffers {
    pub direction: i32,
    pub port_id: i32,
    pub mix_id: i32,
    pub flags: i32,
    pub buffers: Vec<MixerBuffer>,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct PortIO {
    pub direction: i32,
    pub port_id: i32,
    pub mix_id: i32,
    pub id: spa::value::Id,
    pub memid: i32,
    pub offset: i32,
    pub size: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct SetActivation {
    pub nodeid: i32,
    pub signalfd: spa::value::Fd,
    pub memid: i32,
    pub offset: i32,
    pub size: i32,
}

#[derive(PodSerialize, PodDeserialize, Debug)]
pub struct PortSetMixInfo {
    pub direction: i32,
    pub port_id: i32,
    pub mix_id: i32,
    pub peer_id: i32,
    pub probs: HashMap<String, String>,
}

// PodSerialize/PodDeserialize
// We have to do this manuel implementation of PodSerialize and PodDeserialize because params is stored as n_params followed by n spa pods instead of nice array
impl spa::serialize::PodSerialize for Update {
    fn serialize<O: std::io::Write + std::io::Seek>(
        &self,
        serializer: PodSerializer<O>,
    ) -> Result<spa::serialize::SerializeSuccess<O>, cookie_factory::GenError> {
        let mut struct_serializer = serializer.serialize_struct()?;
        struct_serializer.serialize_field(&self.change_mask)?;
        struct_serializer.serialize_field(&(self.params.len() as i32))?;
        for param in self.params.iter() {
            struct_serializer.serialize_field(param)?;
        }
        struct_serializer.serialize_field(&self.info)?;
        struct_serializer.end()
    }
}

impl<'de> spa::deserialize::PodDeserialize<'de> for Update {
    fn deserialize(
        deserializer: spa::deserialize::PodDeserializer<'de>,
    ) -> Result<
        (Self, spa::deserialize::DeserializeSuccess<'de>),
        spa::deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        struct UpdateVisitor;
        impl<'de> spa::deserialize::Visitor<'de> for UpdateVisitor {
            type Value = Update;
            type ArrayElem = std::convert::Infallible;
            fn visit_struct(
                &self,
                struct_deserializer: &mut spa::deserialize::StructPodDeserializer<'de>,
            ) -> Result<Self::Value, spa::deserialize::DeserializeError<&'de [u8]>> {
                let change_mask: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no change_mask field");
                let n_params: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no n_params field");
                let mut params: Vec<spa::value::Value> = Vec::new();
                for _ in 0..n_params {
                    let param: spa::value::Value = struct_deserializer
                        .deserialize_field()?
                        .expect("Input missing param");
                    params.push(param);
                }
                let info: crate::node::Info = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no info field");
                Ok(Update {
                    change_mask,
                    params,
                    info,
                })
            }
        }
        deserializer.deserialize_struct(UpdateVisitor)
    }
}

impl spa::serialize::PodSerialize for PortUpdate {
    fn serialize<O: std::io::Write + std::io::Seek>(
        &self,
        serializer: PodSerializer<O>,
    ) -> Result<spa::serialize::SerializeSuccess<O>, cookie_factory::GenError> {
        let mut struct_serializer = serializer.serialize_struct()?;
        struct_serializer.serialize_field(&self.direction)?;
        struct_serializer.serialize_field(&self.port_id)?;
        struct_serializer.serialize_field(&self.change_mask)?;
        struct_serializer.serialize_field(&(self.params.len() as i32))?;
        for param in self.params.iter() {
            struct_serializer.serialize_field(param)?;
        }
        struct_serializer.serialize_field(&self.info)?;
        struct_serializer.end()
    }
}

impl<'de> spa::deserialize::PodDeserialize<'de> for PortUpdate {
    fn deserialize(
        deserializer: spa::deserialize::PodDeserializer<'de>,
    ) -> Result<
        (Self, spa::deserialize::DeserializeSuccess<'de>),
        spa::deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        struct PortUpdateVisitor;
        impl<'de> spa::deserialize::Visitor<'de> for PortUpdateVisitor {
            type Value = PortUpdate;
            type ArrayElem = std::convert::Infallible;
            fn visit_struct(
                &self,
                struct_deserializer: &mut spa::deserialize::StructPodDeserializer<'de>,
            ) -> Result<Self::Value, spa::deserialize::DeserializeError<&'de [u8]>> {
                let direction: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no change_mask field");
                let port_id: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no change_mask field");
                let change_mask: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no change_mask field");
                let n_params: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no n_params field");
                let mut params: Vec<spa::value::Value> = Vec::new();
                for _ in 0..n_params {
                    let param: spa::value::Value = struct_deserializer
                        .deserialize_field()?
                        .expect("Input missing param");
                    params.push(param);
                }
                let info: crate::port::Info = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no info field");
                Ok(PortUpdate {
                    direction,
                    port_id,
                    change_mask,
                    params,
                    info,
                })
            }
        }
        deserializer.deserialize_struct(PortUpdateVisitor)
    }
}

impl spa::serialize::PodSerialize for PortBuffers {
    fn serialize<O: std::io::Write + std::io::Seek>(
        &self,
        serializer: PodSerializer<O>,
    ) -> Result<spa::serialize::SerializeSuccess<O>, cookie_factory::GenError> {
        let mut struct_serializer = serializer.serialize_struct()?;
        struct_serializer.serialize_field(&self.direction)?;
        struct_serializer.serialize_field(&self.port_id)?;
        struct_serializer.serialize_field(&self.mix_id)?;
        struct_serializer.serialize_field(&(self.buffers.len() as i32))?;
        for buffer in self.buffers.iter() {
            struct_serializer.serialize_field(&(buffer.data_planes.len() as i32))?;
            for data_plane in buffer.data_planes.iter() {
                struct_serializer.serialize_field(&data_plane.type_)?;
                struct_serializer.serialize_field(&data_plane.memfd)?;
                struct_serializer.serialize_field(&data_plane.flags)?;
                struct_serializer.serialize_field(&data_plane.mapoffset)?;
                struct_serializer.serialize_field(&data_plane.maxsize)?;
            }
        }
        struct_serializer.end()
    }
}

impl<'de> spa::deserialize::PodDeserialize<'de> for PortBuffers {
    fn deserialize(
        deserializer: spa::deserialize::PodDeserializer<'de>,
    ) -> Result<
        (Self, spa::deserialize::DeserializeSuccess<'de>),
        spa::deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        struct PortBuffersVisitor;
        impl<'de> spa::deserialize::Visitor<'de> for PortBuffersVisitor {
            type Value = PortBuffers;
            type ArrayElem = std::convert::Infallible;
            fn visit_struct(
                &self,
                struct_deserializer: &mut spa::deserialize::StructPodDeserializer<'de>,
            ) -> Result<Self::Value, spa::deserialize::DeserializeError<&'de [u8]>> {
                let direction: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no change_mask field");
                let port_id: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no change_mask field");
                let mix_id: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no change_mask field");
                let n_buffers: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no n_buffers field");
                let mut buffers: Vec<Buffer> = Vec::new();
                for _ in 0..n_buffers {
                    let n_datas: i32 = struct_deserializer
                        .deserialize_field()?
                        .expect("Input has no n_datas field");
                    let mut data_planes: Vec<DataPlane> = Vec::with_capacity(n_datas as usize);
                    for _ in 0..n_datas {
                        let type_: spa::value::Id = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no type field");
                        let memfd: spa::value::Fd = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no type field");
                        let flags: i32 = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no type field");
                        let mapoffset: i32 = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no type field");
                        let maxsize: i32 = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no type field");
                        data_planes.push(DataPlane {
                            type_,
                            memfd,
                            flags,
                            mapoffset,
                            maxsize,
                        });
                    }
                    buffers.push(Buffer { data_planes });
                }

                Ok(PortBuffers {
                    direction,
                    port_id,
                    mix_id,
                    buffers,
                })
            }
        }
        deserializer.deserialize_struct(PortBuffersVisitor)
    }
}

impl spa::serialize::PodSerialize for UseBuffers {
    fn serialize<O: std::io::Write + std::io::Seek>(
        &self,
        serializer: PodSerializer<O>,
    ) -> Result<spa::serialize::SerializeSuccess<O>, cookie_factory::GenError> {
        let mut struct_serializer = serializer.serialize_struct()?;
        struct_serializer.serialize_field(&self.direction)?;
        struct_serializer.serialize_field(&self.port_id)?;
        struct_serializer.serialize_field(&self.mix_id)?;
        struct_serializer.serialize_field(&(self.buffers.len() as i32))?;
        for buffer in self.buffers.iter() {
            struct_serializer.serialize_field(&buffer.memid)?;
            struct_serializer.serialize_field(&buffer.offset)?;
            struct_serializer.serialize_field(&buffer.size)?;
            struct_serializer.serialize_field(&(buffer.metas.len() as i32))?;
            for meta in buffer.metas.iter() {
                struct_serializer.serialize_field(&meta.type_)?;
                struct_serializer.serialize_field(&meta.size)?;
            }
            struct_serializer.serialize_field(&(buffer.data_blocks.len() as i32))?;
            for data_block in buffer.data_blocks.iter() {
                struct_serializer.serialize_field(&data_block.type_)?;
                struct_serializer.serialize_field(&data_block.data)?;
                struct_serializer.serialize_field(&data_block.flags)?;
                struct_serializer.serialize_field(&data_block.mapoffset)?;
                struct_serializer.serialize_field(&data_block.maxsize)?;
            }
        }
        struct_serializer.end()
    }
}

impl<'de> spa::deserialize::PodDeserialize<'de> for UseBuffers {
    fn deserialize(
        deserializer: spa::deserialize::PodDeserializer<'de>,
    ) -> Result<
        (Self, spa::deserialize::DeserializeSuccess<'de>),
        spa::deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        struct UseBuffersVisitor;
        impl<'de> spa::deserialize::Visitor<'de> for UseBuffersVisitor {
            type Value = UseBuffers;
            type ArrayElem = std::convert::Infallible;
            fn visit_struct(
                &self,
                struct_deserializer: &mut spa::deserialize::StructPodDeserializer<'de>,
            ) -> Result<Self::Value, spa::deserialize::DeserializeError<&'de [u8]>> {
                let direction: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no direction field");
                let port_id: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no port_id field");
                let mix_id: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no mix_id field");
                let flags: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no flags field");
                let n_buffers: i32 = struct_deserializer
                    .deserialize_field()?
                    .expect("Input has no n_buffers field");
                let mut buffers: Vec<MixerBuffer> = Vec::new();
                for _ in 0..n_buffers {
                    let memid: i32 = struct_deserializer
                        .deserialize_field()?
                        .expect("Input has no memid field");
                    let offset: i32 = struct_deserializer
                        .deserialize_field()?
                        .expect("Input has no offset field");
                    let size: i32 = struct_deserializer
                        .deserialize_field()?
                        .expect("Input has no size field");
                    let n_metas: i32 = struct_deserializer
                        .deserialize_field()?
                        .expect("Input has no n_metas field");
                    let mut metas: Vec<Meta> = Vec::with_capacity(n_metas as usize);
                    for _ in 0..n_metas {
                        let type_: spa::value::Id = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no type field");
                        let size: i32 = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no type field");
                        metas.push(Meta { type_, size });
                    }
                    let n_datas: i32 = struct_deserializer
                        .deserialize_field()?
                        .expect("Input has no n_datas field");
                    let mut data_blocks: Vec<DataBlock> = Vec::with_capacity(n_datas as usize);
                    for _ in 0..n_datas {
                        let type_: spa::value::Id = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no type field");
                        let data: i32 = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no data field");
                        let flags: i32 = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no flags field");
                        let mapoffset: i32 = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no mapoffset field");
                        let maxsize: i32 = struct_deserializer
                            .deserialize_field()?
                            .expect("Input has no maxsize field");
                        data_blocks.push(DataBlock {
                            type_,
                            data,
                            flags,
                            mapoffset,
                            maxsize,
                        });
                    }
                    buffers.push(MixerBuffer {
                        memid,
                        offset,
                        size,
                        flags,
                        metas,
                        data_blocks,
                    });
                }

                Ok(UseBuffers {
                    direction,
                    port_id,
                    mix_id,
                    flags,
                    buffers,
                })
            }
        }
        deserializer.deserialize_struct(UseBuffersVisitor)
    }
}
