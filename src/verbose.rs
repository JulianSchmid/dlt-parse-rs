use super::*;

#[derive(Debug, PartialEq)]
pub enum VerboseValue<'a> {
    Bool(BoolValue<'a>),
    Str(StringValue<'a>),
    TraceInfo(TraceInfoValue<'a>),
    I8(I8Value<'a>),
    I16(I16Value<'a>),
    I32(I32Value<'a>),
    I64(I64Value<'a>),
    I128(I128Value<'a>),
    U8(U8Value<'a>),
    U16(U16Value<'a>),
    U32(U32Value<'a>),
    U64(U64Value<'a>),
    U128(U128Value<'a>),
    F16(F16Value<'a>),
    F32(F32Value<'a>),
    F64(F64Value<'a>),
    F128(F128Value<'a>),
    Array(ArrayValue<'a>),
    Struct(StructValue<'a>),
    Raw(RawValue<'a>),
}

impl<'a> VerboseValue<'a> {
    pub fn from_slice(slice: &'a [u8], is_big_endian: bool) -> Result<VerboseValue<'a>, error::VerboseDecodeError> {

        use error::{UnexpectedEndOfSliceError, VerboseDecodeError::*};
        use VerboseValue::*;

        // check that enough data for the type info is present
        if slice.len() < 4 {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: error::Layer::VerboseTypeInfo,
                    minimum_size: 4,
                    actual_size: slice.len(),
                }
            ));
        }

        // SAFETY: Length of at least 4 verified in the previous if.
        let type_info: [u8;4] = unsafe {
            [
                *slice.get_unchecked(0),
                *slice.get_unchecked(1),
                *slice.get_unchecked(2),
                *slice.get_unchecked(3),
            ]
        };

        // determine the type
        const TYPE_LEN_MASK_0: u8 = 0b0000_1111;
        const BOOL_FLAG_0: u8 = 0b0001_0000;
        const SIGNED_FLAG_0: u8 = 0b0010_0000;
        const UNSIGNED_FLAG_0: u8 = 0b0100_0000;
        const FLOAT_FLAG_0: u8 = 0b1000_0000;

        const ARRAY_FLAG_1: u8 = 0b0000_0001;
        const STRING_FLAG_1: u8 = 0b0000_0010;
        const RAW_FLAG_1: u8 = 0b0000_0100;
        const VARINFO_FLAG_1: u8 = 0b0000_1000;
        const FIXED_POINT_FLAG_1: u8 = 0b0001_0000;
        const TRACE_INFO_FLAG_1: u8 = 0b0010_0000;
        const STRUCT_FLAG_1: u8 = 0b0100_0000;

        let mut slicer = FieldSlicer{
            // SAFETY: Length of at least 4 verified in the if at the beginning.
            rest: unsafe {
                std::slice::from_raw_parts(slice.as_ptr().add(4), slice.len() - 4)
            },
            offset: 4,
        };

        if 0 != type_info[0] & BOOL_FLAG_0 {
            
            const CONTRADICTING_MASK_0: u8 = 0b1110_0000;
            const CONTRADICTING_MASK_1: u8 = 0b0111_0111;
            if
                // check type length (must be 1 for bool)
                (1 != type_info[0] & TYPE_LEN_MASK_0) ||
                // check none of the other type flags other then varinfo
                // flag is set
                (0 != type_info[0] & CONTRADICTING_MASK_0) ||
                (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(
                    InvalidTypeInfo(type_info)
                );
            }

            // check for varinfo
            let name = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name(is_big_endian)?)
            } else {
                None
            };

            // verify no conflicting information is present
            let value_u8 = slicer.read_u8()?;

            let value = match value_u8 {
                0 => false,
                1 => true,
                value => return Err(
                    InvalidBoolValue(value)
                ),
            };
            Ok(Bool(BoolValue{
                name,
                value,
            }))
        } else if 0 != type_info[0] & SIGNED_FLAG_0 {
            // verify no conflicting information is present
            // TODO implement
            Err(Unsupported)
        } else if 0 != type_info[0] & UNSIGNED_FLAG_0 {
            // verify no conflicting information is present
            // TODO implement
            Err(Unsupported)
        } else if 0 != type_info[0] & FLOAT_FLAG_0 {
            // verify no conflicting information is present
            // TODO implement
            Err(Unsupported)
        } else if 0 != type_info[1] & ARRAY_FLAG_1 {
            // verify no conflicting information is present
            // TODO implement
            Err(Unsupported)
        } else if 0 != type_info[1] & STRING_FLAG_1 {
            // verify no conflicting information is present
            // TODO implement
            Err(Unsupported)
        } else if 0 != type_info[1] & RAW_FLAG_1 {
            // verify no conflicting information is present+
            const CONTRADICTING_MASK_0: u8 = 0b1111_0000;
            const CONTRADICTING_MASK_1: u8 = 0b0111_0011;
            if
                // check none of the other type flags other then varinfo
                // flag is set
                (0 != type_info[0] & CONTRADICTING_MASK_0) ||
                (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(
                    InvalidTypeInfo(type_info)
                );
            }

            // read len of raw data
            let len = usize::from(slicer.read_u16(is_big_endian)?);

            // check for varinfo
            let name = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name(is_big_endian)?)
            } else {
                None
            };

            Ok(Raw(RawValue{
                name,
                data: slicer.read_raw(len)?,
            }))
        } else if 0 != type_info[1] & TRACE_INFO_FLAG_1 {
            // verify no conflicting information is present
            // TODO implement
            Err(Unsupported)
        } else if 0 != type_info[1] & STRUCT_FLAG_1 {
            // verify no conflicting information is present
            // TODO implement
            Err(Unsupported)
        } else {
            // nothing matches type info uninterpretable
            Err(
                InvalidTypeInfo(type_info)
            )
        }
    }
}

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
pub struct StringValue<'a> {
    // TODO
    // temp until actually implemented
    pub dummy: std::marker::PhantomData<&'a u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TraceInfoValue<'a> {
    // TODO
    // temp until actually implemented
    pub dummy: std::marker::PhantomData<&'a u8>,
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
pub struct ArrayValue<'a> {
    // TODO
    // temp until actually implemented
    pub dummy: std::marker::PhantomData<&'a u8>,
}

#[derive(Debug, PartialEq)]
pub struct StructValue<'a> {
    // TODO
    // temp until actually implemented
    pub dummy: std::marker::PhantomData<&'a u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RawValue<'a> {
    pub name: Option<&'a str>,
    pub data: &'a[u8],
}

/// Helper for parsing verbose messages.
struct FieldSlicer<'a> {
    rest: &'a [u8],
    /// Offset since the parsing has started.
    offset: usize,
}

impl<'a> FieldSlicer<'a> {
    fn read_u8(&mut self) -> Result<u8, error::VerboseDecodeError> {
        use error::{UnexpectedEndOfSliceError, VerboseDecodeError::*};

        // check length
        if self.rest.len() < 1 {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: error::Layer::VerboseValue,
                    minimum_size: self.offset + 1,
                    actual_size: self.offset + self.rest.len(),
                }
            ));
        }

        // SAFETY: Length of at least 1 verified in the previous if.
        let result = unsafe {
            *self.rest.get_unchecked(0)
        };

        // move slice
        // SAFETY: Length of at least 1 verified in the previous if.
        self.rest = unsafe {
            std::slice::from_raw_parts(
                self.rest.as_ptr().add(1),
                self.rest.len() - 1
            )
        };
        self.offset += 1;

        Ok(result)
    }

    fn read_2bytes(&mut self) -> Result<[u8;2], error::VerboseDecodeError> {
        use error::{UnexpectedEndOfSliceError, VerboseDecodeError::*};

        // check length
        if self.rest.len() < 2 {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: error::Layer::VerboseValue,
                    minimum_size: self.offset + 2,
                    actual_size: self.offset + self.rest.len(),
                }
            ));
        }

        // read value
        // SAFETY: Length of at least 2 verified in the previous if.
        let result = unsafe {[
            *self.rest.get_unchecked(0),
            *self.rest.get_unchecked(1)
        ]};

        // move slice
        // SAFETY: Length of at least 2 verified in the previous if.
        self.rest = unsafe {
            std::slice::from_raw_parts(
                self.rest.as_ptr().add(2),
                self.rest.len() - 2
            )
        };
        self.offset += 2;

        Ok(result)
    }

    fn read_u16(&mut self, is_big_endian: bool) -> Result<u16, error::VerboseDecodeError> {
        self.read_2bytes().map(
            |bytes| if is_big_endian {
                u16::from_be_bytes(bytes)
            } else {
                u16::from_le_bytes(bytes)
            }
        )
    }

    fn read_var_name(&mut self, is_big_endian: bool) -> Result<&'a str, error::VerboseDecodeError> {
        // read len
        let len = usize::from(self.read_u16(is_big_endian)?);

        // try decoding the variable name
        Ok(std::str::from_utf8(self.read_raw(len)?)?)
    }

    fn read_raw(&mut self, len: usize) -> Result<&'a [u8], error::VerboseDecodeError> {
        use error::{UnexpectedEndOfSliceError, VerboseDecodeError::*};

        // check that the string length is present
        if self.rest.len() < len {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: error::Layer::VerboseValue,
                    minimum_size: self.offset + len,
                    actual_size: self.offset + self.rest.len(),
                }
            ));
        }

        // SAFETY: Slice length checked above to be at least len
        let result = unsafe {
            std::slice::from_raw_parts(
                self.rest.as_ptr(),
                len
            )
        };

        // move rest & offset
        self.rest = unsafe {
            std::slice::from_raw_parts(
                self.rest.as_ptr().add(len),
                self.rest.len() - len
            )
        };
        self.offset += len;

        Ok(result)
    }
}
