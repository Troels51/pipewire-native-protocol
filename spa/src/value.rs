use std::{
    io::{Seek, Write},
    os::raw::c_void,
};

use bitflags::bitflags;
use cookie_factory::{
    bytes::{ne_f32, ne_f64, ne_i32, ne_i64, ne_u32},
    gen_simple,
    sequence::pair,
    GenError,
};

use nom::{
    combinator::map,
    number::{
        complete::{f32, f64, i32, i64, u32},
        Endianness,
    },
    IResult,
};

use self::deserialize::{
    ChoiceBoolVisitor, ChoiceDoubleVisitor, ChoiceFdVisitor, ChoiceFloatVisitor,
    ChoiceFractionVisitor, ChoiceIdVisitor, ChoiceIntVisitor, ChoiceLongVisitor,
    ChoiceRectangleVisitor, DoubleVisitor, FdVisitor, FloatVisitor, FractionVisitor, IdVisitor,
    IntVisitor, LongVisitor, PointerVisitor, RectangleVisitor,
};
use crate::{
    deserialize::{self, BoolVisitor, NoneVisitor, PodDeserialize, PodDeserializer},
    serialize::{self, PodSerialize, PodSerializer},
    spa_pod_types, CanonicalFixedSizedPod, FixedSizedPod,
};

/// A typed pod value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// no value or a NULL pointer.
    None,
    /// a boolean value.
    Bool(bool),
    /// an enumerated value.
    Id(Id),
    /// a 32 bits integer.
    Int(i32),
    /// a 64 bits integer.
    Long(i64),
    /// a 32 bits floating.
    Float(f32),
    /// a 64 bits floating.
    Double(f64),
    /// a string.
    String(String),
    /// a byte array.
    Bytes(Vec<u8>),
    /// a rectangle with width and height.
    Rectangle(Rectangle),
    /// a fraction with numerator and denominator.
    Fraction(Fraction),
    /// a file descriptor.
    Fd(Fd),
    /// an array of same type objects.
    ValueArray(ValueArray),
    /// a collection of types and objects.
    Struct(Vec<Value>),
    /// an object.
    Object(Object),
    /// a choice.
    Choice(ChoiceValue),
    /// a pointer.
    Pointer(u32, *const c_void),
}

/// an array of same type objects.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueArray {
    /// an array of none.
    None(Vec<()>),
    /// an array of booleans.
    Bool(Vec<bool>),
    /// an array of Id.
    Id(Vec<Id>),
    /// an array of 32 bits integer.
    Int(Vec<i32>),
    /// an array of 64 bits integer.
    Long(Vec<i64>),
    /// an array of 32 bits floating.
    Float(Vec<f32>),
    /// an array of 64 bits floating.
    Double(Vec<f64>),
    /// an array of Rectangle.
    Rectangle(Vec<Rectangle>),
    /// an array of Fraction.
    Fraction(Vec<Fraction>),
    /// an array of Fd.
    Fd(Vec<Fd>),
}

/// A typed choice.
#[derive(Debug, Clone, PartialEq)]
pub enum ChoiceValue {
    /// Choice on boolean values.
    Bool(Choice<bool>),
    /// Choice on 32 bits integer values.
    Int(Choice<i32>),
    /// Choice on 64 bits integer values.
    Long(Choice<i64>),
    /// Choice on 32 bits floating values.
    Float(Choice<f32>),
    /// Choice on 64 bits floating values.
    Double(Choice<f64>),
    /// Choice on id values.
    Id(Choice<Id>),
    /// Choice on rectangle values.
    Rectangle(Choice<Rectangle>),
    /// Choice on fraction values.
    Fraction(Choice<Fraction>),
    /// Choice on fd values.
    Fd(Choice<Fd>),
}

/// An object from a pod.
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    /// the object type.
    pub type_: u32,
    /// the object id.
    pub id: u32,
    /// the object properties.
    pub properties: Vec<Property>,
}

/// An object property.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// key of the property, list of valid keys depends on the object type.
    pub key: u32,
    /// flags for the property.
    pub flags: PropertyFlags,
    /// value of the property.
    pub value: Value,
}

impl Property {
    pub fn new(key: u32, value: Value) -> Self {
        Self {
            key,
            value,
            flags: PropertyFlags::empty(),
        }
    }
}

bitflags! {
    /// Property flags
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct PropertyFlags: u32 {
        // These flags are redefinitions from
        // https://gitlab.freedesktop.org/pipewire/pipewire/-/blob/master/spa/include/spa/pod/pod.h
        /// Property is read-only.
        const READONLY = 1 << 0;
        /// Property is some sort of hardware parameter.
        const HARDWARE = 1 << 1;
        /// Property contains a dictionary struct.
        const HINT_DICT = 1 << 2;
        /// Property is mandatory.
        const MANDATORY = 1 << 3;
        /// Property choices need no fixation.
        const DONT_FIXATE = 1 << 4;
    }
}

/// An enumerated value in a pod
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Id(pub u32);

/// A file descriptor in a pod
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Fd(pub i64);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Rectangle {
    width: u32,
    height: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Fraction {
    num: u32,
    denom: u32,
}

#[derive(Debug, Eq, PartialEq, Clone)]
/// the flags and choice of a choice pod.
pub struct Choice<T: CanonicalFixedSizedPod>(pub ChoiceFlags, pub ChoiceEnum<T>);

bitflags! {
    /// [`Choice`] flags
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct ChoiceFlags: u32 {
        // no flags defined yet but we need at least one to keep bitflags! happy
        #[doc(hidden)]
        const _FAKE = 1;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// a choice in a pod.
pub enum ChoiceEnum<T: CanonicalFixedSizedPod> {
    /// no choice.
    None(T),
    /// range.
    Range {
        /// default value.
        default: T,
        /// minimum value.
        min: T,
        /// maximum value.
        max: T,
    },
    /// range with step.
    Step {
        /// default value.
        default: T,
        /// minimum value.
        min: T,
        /// maximum value.
        max: T,
        /// step.
        step: T,
    },
    /// list.
    Enum {
        /// default value.
        default: T,
        /// alternative values.
        alternatives: Vec<T>,
    },
    /// flags.
    Flags {
        /// default value.
        default: T,
        /// possible flags.
        flags: Vec<T>,
    },
}

impl<T: CanonicalFixedSizedPod + Copy> FixedSizedPod for T {
    type CanonicalType = Self;

    fn as_canonical_type(&self) -> Self::CanonicalType {
        *self
    }

    fn from_canonical_type(canonical: &Self::CanonicalType) -> Self {
        *canonical
    }
}

/// Serialize into a `None` type pod.
impl CanonicalFixedSizedPod for () {
    const TYPE: u32 = spa_pod_types::NONE;
    const SIZE: u32 = 0;

    fn serialize_body<O: Write>(&self, out: O) -> Result<O, GenError> {
        Ok(out)
    }

    fn deserialize_body(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized,
    {
        Ok((input, ()))
    }
}

/// Serialize into a `Bool` type pod.
impl CanonicalFixedSizedPod for bool {
    const TYPE: u32 = spa_pod_types::BOOL;
    const SIZE: u32 = 4;

    fn serialize_body<O: Write>(&self, out: O) -> Result<O, GenError> {
        gen_simple(ne_u32(u32::from(*self)), out)
    }

    fn deserialize_body(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized,
    {
        map(u32(Endianness::Native), |b| b != 0)(input)
    }
}

/// Serialize into a `Int` type pod.
impl CanonicalFixedSizedPod for i32 {
    const TYPE: u32 = spa_pod_types::INT;
    const SIZE: u32 = 4;

    fn serialize_body<O: Write>(&self, out: O) -> Result<O, GenError> {
        gen_simple(ne_i32(*self), out)
    }

    fn deserialize_body(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized,
    {
        i32(Endianness::Native)(input)
    }
}

/// Serialize into a `Long` type pod.
impl CanonicalFixedSizedPod for i64 {
    const TYPE: u32 = spa_pod_types::LONG;
    const SIZE: u32 = 8;

    fn serialize_body<O: Write>(&self, out: O) -> Result<O, GenError> {
        gen_simple(ne_i64(*self), out)
    }

    fn deserialize_body(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized,
    {
        i64(Endianness::Native)(input)
    }
}

/// Serialize into a `Float` type pod.
impl CanonicalFixedSizedPod for f32 {
    const TYPE: u32 = spa_pod_types::FLOAT;
    const SIZE: u32 = 4;

    fn serialize_body<O: Write>(&self, out: O) -> Result<O, GenError> {
        gen_simple(ne_f32(*self), out)
    }

    fn deserialize_body(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized,
    {
        f32(Endianness::Native)(input)
    }
}

/// Serialize into a `Double` type pod.
impl CanonicalFixedSizedPod for f64 {
    const TYPE: u32 = spa_pod_types::DOUBLE;
    const SIZE: u32 = 8;

    fn serialize_body<O: Write>(&self, out: O) -> Result<O, GenError> {
        gen_simple(ne_f64(*self), out)
    }

    fn deserialize_body(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized,
    {
        f64(Endianness::Native)(input)
    }
}

/// Serialize into a `Rectangle` type pod.
impl CanonicalFixedSizedPod for Rectangle {
    const TYPE: u32 = spa_pod_types::RECTANGLE;
    const SIZE: u32 = 8;

    fn serialize_body<O: Write>(&self, out: O) -> Result<O, GenError> {
        gen_simple(pair(ne_u32(self.width), ne_u32(self.height)), out)
    }

    fn deserialize_body(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized,
    {
        map(
            nom::sequence::pair(u32(Endianness::Native), u32(Endianness::Native)),
            |(width, height)| Rectangle { width, height },
        )(input)
    }
}

/// Serialize into a `Fraction` type pod.
impl CanonicalFixedSizedPod for Fraction {
    const TYPE: u32 = spa_pod_types::FRACTION;
    const SIZE: u32 = 8;

    fn serialize_body<O: Write>(&self, out: O) -> Result<O, GenError> {
        gen_simple(pair(ne_u32(self.num), ne_u32(self.denom)), out)
    }

    fn deserialize_body(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized,
    {
        map(
            nom::sequence::pair(u32(Endianness::Native), u32(Endianness::Native)),
            |(num, denom)| Fraction { num, denom },
        )(input)
    }
}

impl CanonicalFixedSizedPod for Id {
    const TYPE: u32 = spa_pod_types::ID;
    const SIZE: u32 = 4;

    fn serialize_body<O: Write>(&self, out: O) -> Result<O, GenError> {
        gen_simple(ne_u32(self.0), out)
    }

    fn deserialize_body(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized,
    {
        map(u32(Endianness::Native), Id)(input)
    }
}

impl CanonicalFixedSizedPod for Fd {
    const TYPE: u32 = spa_pod_types::FD;
    const SIZE: u32 = 8;

    fn serialize_body<O: Write>(&self, out: O) -> Result<O, GenError> {
        gen_simple(ne_i64(self.0), out)
    }

    fn deserialize_body(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized,
    {
        map(i64(Endianness::Native), Fd)(input)
    }
}

impl<T: FixedSizedPod> PodSerialize for T {
    fn serialize<O: Write + Seek>(
        &self,
        serializer: PodSerializer<O>,
    ) -> Result<serialize::SerializeSuccess<O>, GenError> {
        serializer.serialized_fixed_sized_pod(self)
    }
}

impl<'de> PodDeserialize<'de> for () {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_none(NoneVisitor)
    }
}

impl<'de> PodDeserialize<'de> for bool {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_bool(BoolVisitor)
    }
}

impl<'de> PodDeserialize<'de> for i32 {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_int(IntVisitor)
    }
}

impl<'de> PodDeserialize<'de> for i64 {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_long(LongVisitor)
    }
}

impl<'de> PodDeserialize<'de> for f32 {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_float(FloatVisitor)
    }
}

impl<'de> PodDeserialize<'de> for f64 {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_double(DoubleVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Rectangle {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_rectangle(RectangleVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Fraction {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_fraction(FractionVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Id {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_id(IdVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Fd {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_fd(FdVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Choice<bool> {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_choice(ChoiceBoolVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Choice<i32> {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_choice(ChoiceIntVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Choice<i64> {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_choice(ChoiceLongVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Choice<f32> {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_choice(ChoiceFloatVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Choice<f64> {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_choice(ChoiceDoubleVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Choice<Id> {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_choice(ChoiceIdVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Choice<Rectangle> {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_choice(ChoiceRectangleVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Choice<Fraction> {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_choice(ChoiceFractionVisitor)
    }
}

impl<'de> PodDeserialize<'de> for Choice<Fd> {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_choice(ChoiceFdVisitor)
    }
}

impl<'de, T> PodDeserialize<'de> for (u32, *const T) {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_pointer(PointerVisitor::<T>::default())
    }
}

impl<'de> PodDeserialize<'de> for Value {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<
        (Self, deserialize::DeserializeSuccess<'de>),
        deserialize::DeserializeError<&'de [u8]>,
    >
    where
        Self: Sized,
    {
        deserializer.deserialize_any()
    }
}
