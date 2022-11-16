mod field_slicer;
use field_slicer::*;

mod verbose_value;
pub use verbose_value::*;

use super::*;
use core::str;

#[derive(Debug, PartialEq)]
pub struct Scaling<T: Sized> {
    quantization: f32,
    offset: T,
}

#[derive(Debug, Eq, PartialEq)]
pub struct BoolValue<'a> {
    pub name: Option<&'a str>,
    pub value: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum StringValue<'a> {
    Ascii(&'a str),
    Utf8(&'a str),
}

#[derive(Debug, Eq, PartialEq)]
pub struct TraceInfoValue<'a> {
    // TODO
    // temp until actually implemented
    pub dummy: core::marker::PhantomData<&'a u8>,
}

/// Verbose 8 bit signed integer.
#[derive(Debug, PartialEq)]
pub struct I8Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub scaling: Option<Scaling<i32>>,
    pub value: i8,
}

/// Verbose 16 bit signed integer.
#[derive(Debug, PartialEq)]
pub struct I16Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub scaling: Option<Scaling<i32>>,
    pub value: i16,
}

/// Verbose 32 bit signed integer.
#[derive(Debug, PartialEq)]
pub struct I32Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub scaling: Option<Scaling<i32>>,
    pub value: i32,
}

/// Verbose 32 bit signed integer.
#[derive(Debug, PartialEq)]
pub struct I64Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub scaling: Option<Scaling<i64>>,
    pub value: i64,
}

/// Verbose 32 bit signed integer.
#[derive(Debug, PartialEq)]
pub struct I128Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub scaling: Option<Scaling<i128>>,
    pub value: i128,
}

/// Verbose 8 bit unsigned integer.
#[derive(Debug, PartialEq)]
pub struct U8Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub scaling: Option<Scaling<u32>>,
    pub value: u8,
}

/// Verbose 16 bit unsigned integer.
#[derive(Debug, PartialEq)]
pub struct U16Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub scaling: Option<Scaling<u32>>,
    pub value: u16,
}

/// Verbose 32 bit unsigned integer.
#[derive(Debug, PartialEq)]
pub struct U32Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub scaling: Option<Scaling<u32>>,
    pub value: u32,
}

/// Verbose 32 bit unsigned integer.
#[derive(Debug, PartialEq)]
pub struct U64Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub scaling: Option<Scaling<u64>>,
    pub value: u64,
}

/// Verbose 32 bit unsigned integer.
#[derive(Debug, PartialEq)]
pub struct U128Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub scaling: Option<Scaling<u64>>,
    pub value: u64,
}

/// Verbose 16 bit float number.
#[derive(Debug, PartialEq)]
pub struct F16Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub value: [u8;2],
}

/// Verbose 32 bit float number.
#[derive(Debug, PartialEq)]
pub struct F32Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub value: f32,
}

/// Verbose 64 bit float number.
#[derive(Debug, PartialEq)]
pub struct F64Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub value: f64,
}

/// Verbose 128 bit float number.
#[derive(Debug, PartialEq)]
pub struct F128Value<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub value: [u8;16],
}

#[derive(Debug, PartialEq)]
pub struct ArrayDimensions<'a> {
    /// Pointer to the raw dimensions data.
    dimensions: &'a [u8],
}

// TODO implement custom debug for "ArrayDimensions"

// Q: Should I just add array types for all int & bool types?
//
//   pro:
//      - explicit types, easy
//
//   negative:
//      - creates more types
//
//   open questions:
//      - does this conflict with dlt v2? (e.g. does v2 support more 
//        array types)
//
//   alternative:
//      - some kind of generic?
//         -> this only

#[derive(Debug, PartialEq)]
pub struct ArrayValue<'a> {
    // TODO
    // temp until actually implemented
    pub dummy: core::marker::PhantomData<&'a u8>,
    
    pub dimensions: ArrayDimensions<'a>,

}

#[derive(Debug, PartialEq)]
pub struct ArrayBool<'a> {
    pub dimensions: ArrayDimensions<'a>,
    pub data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ArrayU8<'a> {
    pub dimensions: ArrayDimensions<'a>,
    pub data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ArrayU16<'a> {
    dimensions: ArrayDimensions<'a>,
    data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ArrayU32<'a> {
    dimensions: ArrayDimensions<'a>,
    data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ArrayU64<'a> {
    dimensions: ArrayDimensions<'a>,
    data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ArrayU128<'a> {
    dimensions: ArrayDimensions<'a>,
    data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ArrayI8<'a> {
    dimensions: ArrayDimensions<'a>,
    data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ArrayI16<'a> {
    dimensions: ArrayDimensions<'a>,
    data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ArrayI32<'a> {
    dimensions: ArrayDimensions<'a>,
    data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ArrayI64<'a> {
    dimensions: ArrayDimensions<'a>,
    data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ArrayI128<'a> {
    dimensions: ArrayDimensions<'a>,
    data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct StructValue<'a> {
    // TODO
    // temp until actually implemented
    pub dummy: core::marker::PhantomData<&'a u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RawValue<'a> {
    pub name: Option<&'a str>,
    pub data: &'a[u8],
}
