extern crate alloc;

use crate::error::VerboseDecodeError;

use super::*;

use core::slice;
use core::str;
use std::println;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
            println!("No type info present!");
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
            const CONTRADICTING_MASK_1: u8 = 0b1111_0111;
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
            // todo: handle arrays (currently set as contradicting)
            const CONTRADICTING_MASK_0: u8 = 0b1101_0000;
            const CONTRADICTING_MASK_1: u8 = 0b1110_0111;

            // check that no contradicting type info is present
            if (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            let type_len = type_info[0] & 0b1111;
            match type_len {
                1 | 2 | 3 | 4 | 5 => {}
                _ => return Err(InvalidTypeInfo(type_info)),
            }

            // check for varinfo
            let name_and_unit = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name_and_unit(is_big_endian)?)
            } else {
                None
            };

            let read_i32_scaling =
                |slicer: &mut FieldSlicer| -> Result<Option<Scaling<i32>>, VerboseDecodeError> {
                    if 0 != type_info[1] & FIXED_POINT_FLAG_1 {
                        Ok(Some(Scaling {
                            quantization: slicer.read_f32(is_big_endian)?,
                            offset: slicer.read_i32(is_big_endian)?,
                        }))
                    } else {
                        Ok(None)
                    }
                };

            let read_i64_scaling =
                |slicer: &mut FieldSlicer| -> Result<Option<Scaling<i64>>, VerboseDecodeError> {
                    if 0 != type_info[1] & FIXED_POINT_FLAG_1 {
                        Ok(Some(Scaling {
                            quantization: slicer.read_f32(is_big_endian)?,
                            offset: slicer.read_i64(is_big_endian)?,
                        }))
                    } else {
                        Ok(None)
                    }
                };

            let read_i128_scaling =
                |slicer: &mut FieldSlicer| -> Result<Option<Scaling<i128>>, VerboseDecodeError> {
                    if 0 != type_info[1] & FIXED_POINT_FLAG_1 {
                        Ok(Some(Scaling {
                            quantization: slicer.read_f32(is_big_endian)?,
                            offset: slicer.read_i128(is_big_endian)?,
                        }))
                    } else {
                        Ok(None)
                    }
                };

            match type_len {
                1 => Ok((
                    I8(I8Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        scaling: read_i32_scaling(&mut slicer)?,
                        value: slicer.read_i8()?,
                    }),
                    slicer.rest(),
                )),
                2 => Ok((
                    I16(I16Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        scaling: read_i32_scaling(&mut slicer)?,
                        value: slicer.read_i16(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                3 => Ok((
                    I32(I32Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        scaling: read_i32_scaling(&mut slicer)?,
                        value: slicer.read_i32(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                4 => Ok((
                    I64(I64Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        scaling: read_i64_scaling(&mut slicer)?,
                        value: slicer.read_i64(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                5 => Ok((
                    I128(I128Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        scaling: read_i128_scaling(&mut slicer)?,
                        value: slicer.read_i128(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                _ => unreachable!(),
            }
        } else if 0 != type_info[0] & UNSIGNED_FLAG_0 {
            // verify no conflicting information is present
            // todo: handle arrays (currently set as contradicting)
            const CONTRADICTING_MASK_0: u8 = 0b1011_0000;
            const CONTRADICTING_MASK_1: u8 = 0b1110_0111;

            // check that no contradicting type info is present
            if (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            let type_len = type_info[0] & 0b1111;
            match type_len {
                1 | 2 | 3 | 4 | 5 => {}
                _ => return Err(InvalidTypeInfo(type_info)),
            }

            // check for varinfo
            let name_and_unit = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name_and_unit(is_big_endian)?)
            } else {
                None
            };

            let read_u32_scaling =
                |slicer: &mut FieldSlicer| -> Result<Option<Scaling<u32>>, VerboseDecodeError> {
                    if 0 != type_info[1] & FIXED_POINT_FLAG_1 {
                        Ok(Some(Scaling {
                            quantization: slicer.read_f32(is_big_endian)?,
                            offset: slicer.read_u32(is_big_endian)?,
                        }))
                    } else {
                        Ok(None)
                    }
                };

            let read_u64_scaling =
                |slicer: &mut FieldSlicer| -> Result<Option<Scaling<u64>>, VerboseDecodeError> {
                    if 0 != type_info[1] & FIXED_POINT_FLAG_1 {
                        Ok(Some(Scaling {
                            quantization: slicer.read_f32(is_big_endian)?,
                            offset: slicer.read_u64(is_big_endian)?,
                        }))
                    } else {
                        Ok(None)
                    }
                };

            let read_u128_scaling =
                |slicer: &mut FieldSlicer| -> Result<Option<Scaling<u128>>, VerboseDecodeError> {
                    if 0 != type_info[1] & FIXED_POINT_FLAG_1 {
                        Ok(Some(Scaling {
                            quantization: slicer.read_f32(is_big_endian)?,
                            offset: slicer.read_u128(is_big_endian)?,
                        }))
                    } else {
                        Ok(None)
                    }
                };

            match type_len {
                1 => Ok((
                    U8(U8Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        scaling: read_u32_scaling(&mut slicer)?,
                        value: slicer.read_u8()?,
                    }),
                    slicer.rest(),
                )),
                2 => Ok((
                    U16(U16Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        scaling: read_u32_scaling(&mut slicer)?,
                        value: slicer.read_u16(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                3 => Ok((
                    U32(U32Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        scaling: read_u32_scaling(&mut slicer)?,
                        value: slicer.read_u32(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                4 => Ok((
                    U64(U64Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        scaling: read_u64_scaling(&mut slicer)?,
                        value: slicer.read_u64(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                5 => Ok((
                    U128(U128Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        scaling: read_u128_scaling(&mut slicer)?,
                        value: slicer.read_u128(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                _ => unreachable!(),
            }

            // TODO implement
        } else if 0 != type_info[0] & FLOAT_FLAG_0 {
            // verify no conflicting information is present
            // todo: handle arrays (currently set as contradicting)

            const CONTRADICTING_MASK_0: u8 = 0b0111_0000;
            const CONTRADICTING_MASK_1: u8 = 0b1111_0111;

            // check that no contradicting type info is present
            if (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            let type_len = type_info[0] & 0b1111;
            match type_len {
                2 | 3 | 4 | 5 => {}
                _ => return Err(InvalidTypeInfo(type_info)),
            }

            // check for varinfo
            let name_and_unit = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name_and_unit(is_big_endian)?)
            } else {
                None
            };

            match type_len {
                2 => Ok((
                    F16(F16Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        value: slicer.read_f16(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                3 => Ok((
                    F32(F32Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        value: slicer.read_f32(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                4 => Ok((
                    F64(F64Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        value: slicer.read_f64(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                5 => Ok((
                    F128(F128Value {
                        name: name_and_unit.map(|v| v.0),
                        unit: name_and_unit.map(|v| v.1),
                        value: slicer.read_f128(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                _ => unreachable!(),
            }

            // TODO implement
            //Err(Unsupported(type_info[0], type_info[1]))
        } else if 0 != type_info[1] & ARRAY_FLAG_1 {
            // verify no conflicting information is present
            // TODO implement
            const CONTRADICTING_MASK_0: u8 = 0b1111_0000;
            const CONTRADICTING_MASK_1: u8 = 0b1111_0110;

            if
            // check none of the other type flags other then varinfo
            // flag is set
            (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            Err(Unsupported(type_info[0], type_info[1]))
        } else if 0 != type_info[1] & STRING_FLAG_1 {
            const CONTRADICTING_MASK_0: u8 = 0b1111_0000;
            const CONTRADICTING_MASK_1: u8 = 0b0111_0101;

            if
            // check none of the other type flags other then varinfo
            // flag is set
            (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            let len = usize::from(slicer.read_u16(is_big_endian)?);

            let name = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name(is_big_endian)?)
            } else {
                None
            };
            let parse: Result<&str, str::Utf8Error> = match slicer.read_raw(len) {
                Ok(valid_parse) => {
                    if len > 0 {
                        str::from_utf8(&valid_parse[..valid_parse.len() - 1])
                    } else {
                        Ok("")
                    }
                }
                Err(_) => return Err(Unsupported(type_info[0], type_info[1])),
            };

            match parse {
                Ok(value) => Ok((Str(StringValue { name, value }), slicer.rest())),
                Err(_) => Err(Unsupported(type_info[0], type_info[1])),
            }

            // println!("Result is {str_to_prt}");
            // verify no conflicting information is present
            // TODO implement
            //Err(Unsupported(type_info[0], type_info[1]))
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

            const CONTRADICTING_MASK_0: u8 = 0b1111_1111;
            const CONTRADICTING_MASK_1: u8 = 0b0101_1111;

            // check that no contradicting type info is present
            if (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            // read len of trace data string
            let len = usize::from(slicer.read_u16(is_big_endian)?);

            let parse: Result<&str, str::Utf8Error> = match slicer.read_raw(len) {
                Ok(valid_parse) => {
                    if len > 0 {
                        str::from_utf8(&valid_parse[..valid_parse.len() - 1])
                    } else {
                        Ok("")
                    }
                }
                Err(_) => return Err(Unsupported(type_info[0], type_info[1])),
            };

            match parse {
                Ok(value) => Ok((TraceInfo(TraceInfoValue { value }), slicer.rest())),
                Err(_) => Err(Unsupported(type_info[0], type_info[1])),
            }
        } else if 0 != type_info[1] & STRUCT_FLAG_1 {
            // verify no conflicting information is present
            const CONTRADICTING_MASK_0: u8 = 0b1111_1111;
            const CONTRADICTING_MASK_1: u8 = 0b1011_0111;

            // check that no contradicting type info is present
            if (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            // read number of struct entries
            let len = usize::from(slicer.read_u16(is_big_endian)?);

            let name = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name(is_big_endian)?)
            } else {
                None
            };

            // TODO implement
            Err(Unsupported(type_info[0], type_info[1]))
        } else {
            // nothing matches type info uninterpretable
            Err(InvalidTypeInfo(type_info))
        }
    }
}
