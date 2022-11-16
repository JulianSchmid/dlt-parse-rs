use super::*;

use core::slice;
use core::str;

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
    pub fn from_slice(
        slice: &'a [u8],
        is_big_endian: bool,
    ) -> Result<(VerboseValue<'a>, &'a [u8]), error::VerboseDecodeError> {
        use error::{UnexpectedEndOfSliceError, VerboseDecodeError::*};
        use VerboseValue::*;

        // check that enough data for the type info is present
        if slice.len() < 4 {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: error::Layer::VerboseTypeInfo,
                minimum_size: 4,
                actual_size: slice.len(),
            }));
        }

        // SAFETY: Length of at least 4 verified in the previous if.
        let type_info: [u8; 4] = unsafe {
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

        let mut slicer = FieldSlicer::new(
            // SAFETY: Length of at least 4 verified in the if at the beginning.
            unsafe { slice::from_raw_parts(slice.as_ptr().add(4), slice.len() - 4) },
            4,
        );

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
                return Err(InvalidTypeInfo(type_info));
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
                value => return Err(InvalidBoolValue(value)),
            };
            Ok((Bool(BoolValue { name, value }), slicer.rest()))
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
            (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            // read len of raw data
            let len = usize::from(slicer.read_u16(is_big_endian)?);

            // check for varinfo
            let name = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name(is_big_endian)?)
            } else {
                None
            };

            Ok((
                Raw(RawValue {
                    name,
                    data: slicer.read_raw(len)?,
                }),
                slicer.rest(),
            ))
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
            Err(InvalidTypeInfo(type_info))
        }
    }
}
