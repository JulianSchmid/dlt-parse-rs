use crate::error::VerboseDecodeError;

use super::*;

use core::slice;

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
    ArrBool(ArrayBool<'a>),
    ArrI8(ArrayI8<'a>),
    ArrI16(ArrayI16<'a>),
    ArrI32(ArrayI32<'a>),
    ArrI64(ArrayI64<'a>),
    ArrI128(ArrayI128<'a>),
    ArrU8(ArrayU8<'a>),
    ArrU16(ArrayU16<'a>),
    ArrU32(ArrayU32<'a>),
    ArrU64(ArrayU64<'a>),
    ArrU128(ArrayU128<'a>),
    ArrF16(ArrayF16<'a>),
    ArrF32(ArrayF32<'a>),
    ArrF64(ArrayF64<'a>),
    ArrF128(ArrayF128<'a>),
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
        const TRACE_INFO_FLAG_1: u8 = 0b0010_0000;
        const STRUCT_FLAG_1: u8 = 0b0100_0000;

        let mut slicer = FieldSlicer::new(
            // SAFETY: Length of at least 4 verified in the if at the beginning.
            unsafe { slice::from_raw_parts(slice.as_ptr().add(4), slice.len() - 4) },
            4,
        );

        if 0 != type_info[1] & ARRAY_FLAG_1 {
            let type_len: usize = usize::from(type_info[0] & TYPE_LEN_MASK_0);

            // read array dimensions
            let dimensions = slicer.read_array_dimesions(is_big_endian)?;

            // check for varinfo
            let name_and_unit = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name_and_unit(is_big_endian)?)
            } else {
                None
            };

            let variable_info = name_and_unit.map(|(name, unit)| VariableInfoUnit { name, unit });

            if 0 != type_info[0] & BOOL_FLAG_0 {
                const CONTRADICTING_MASK_0: u8 = 0b1110_0000;
                const CONTRADICTING_MASK_1: u8 = 0b1111_0110;
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

                // determine data size of array
                let mut data_len = 0;
                for dim in &dimensions {
                    if let Some(sum) = (usize::from(dim)).checked_add(data_len) {
                        data_len = sum;
                    } else {
                        return Err(VerboseDecodeError::ArrayDimensionsOverflow);
                    }
                }

                // take the data area of the bool array
                Ok((
                    ArrBool(ArrayBool {
                        dimensions,
                        variable_info,
                        data: slicer.read_raw(data_len)?,
                    }),
                    slicer.rest(),
                ))
            } else if 0 != type_info[0] & SIGNED_FLAG_0 {
                const CONTRADICTING_MASK_0: u8 = 0b1101_0000;
                const CONTRADICTING_MASK_1: u8 = 0b1110_0110;

                // check that no contradicting type info is present
                if (0 != type_info[0] & CONTRADICTING_MASK_0)
                    || (0 != type_info[1] & CONTRADICTING_MASK_1)
                {
                    return Err(InvalidTypeInfo(type_info));
                }

                match type_len {
                    1..=5 => {}
                    _ => return Err(InvalidTypeInfo(type_info)), //Look
                }

                let real_type_len = 0b0000_0001 << (type_len - 1);

                // determine data size of array
                let mut data_len = 0;
                for dim in &dimensions {
                    if let Some(sum) = (usize::from(dim) * real_type_len).checked_add(data_len) {
                        data_len = sum;
                    } else {
                        return Err(VerboseDecodeError::ArrayDimensionsOverflow);
                    }
                }

                match type_len {
                    1 => Ok((
                        ArrI8(ArrayI8 {
                            dimensions,
                            variable_info,
                            scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    2 => Ok((
                        ArrI16(ArrayI16 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    3 => Ok((
                        ArrI32(ArrayI32 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    4 => Ok((
                        ArrI64(ArrayI64 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            scaling: slicer.read_i64_scaling(is_big_endian, type_info)?,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    5 => Ok((
                        ArrI128(ArrayI128 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            scaling: slicer.read_i128_scaling(is_big_endian, type_info)?,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    _ => unreachable!(),
                }
            } else if 0 != type_info[0] & UNSIGNED_FLAG_0 {
                const CONTRADICTING_MASK_0: u8 = 0b1011_0000;
                const CONTRADICTING_MASK_1: u8 = 0b1110_0110;

                // check that no contradicting type info is present
                if (0 != type_info[0] & CONTRADICTING_MASK_0)
                    || (0 != type_info[1] & CONTRADICTING_MASK_1)
                {
                    return Err(InvalidTypeInfo(type_info));
                }

                let type_len = type_info[0] & TYPE_LEN_MASK_0;
                match type_len {
                    1..=5 => {}
                    _ => return Err(InvalidTypeInfo(type_info)),
                }

                let real_type_len = 0b0000_0001 << (type_len - 1);

                // determine data size of array
                let mut data_len = 0;
                for dim in &dimensions {
                    if let Some(sum) = (usize::from(dim) * real_type_len).checked_add(data_len) {
                        data_len = sum;
                    } else {
                        return Err(VerboseDecodeError::ArrayDimensionsOverflow);
                    }
                }

                match type_len {
                    1 => Ok((
                        ArrU8(ArrayU8 {
                            dimensions,
                            variable_info,
                            scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    2 => Ok((
                        ArrU16(ArrayU16 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    3 => Ok((
                        ArrU32(ArrayU32 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    4 => Ok((
                        ArrU64(ArrayU64 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            scaling: slicer.read_i64_scaling(is_big_endian, type_info)?,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    5 => Ok((
                        ArrU128(ArrayU128 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            scaling: slicer.read_i128_scaling(is_big_endian, type_info)?,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    _ => unreachable!(),
                }
            } else if 0 != type_info[0] & FLOAT_FLAG_0 {
                const CONTRADICTING_MASK_0: u8 = 0b0111_0000;
                const CONTRADICTING_MASK_1: u8 = 0b1111_0110;

                // check that no contradicting type info is present
                if (0 != type_info[0] & CONTRADICTING_MASK_0)
                    || (0 != type_info[1] & CONTRADICTING_MASK_1)
                {
                    return Err(InvalidTypeInfo(type_info));
                }

                let type_len = type_info[0] & TYPE_LEN_MASK_0;
                match type_len {
                    2..=5 => {}
                    _ => return Err(InvalidTypeInfo(type_info)),
                }

                let real_type_len = 0b0000_0001 << (type_len - 1);

                // determine data size of array
                let mut data_len = 0;
                for dim in &dimensions {
                    if let Some(sum) = (usize::from(dim) * real_type_len).checked_add(data_len) {
                        data_len = sum;
                    } else {
                        return Err(VerboseDecodeError::ArrayDimensionsOverflow);
                    }
                }

                match type_len {
                    2 => Ok((
                        ArrF16(ArrayF16 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    3 => Ok((
                        ArrF32(ArrayF32 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    4 => Ok((
                        ArrF64(ArrayF64 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    5 => Ok((
                        ArrF128(ArrayF128 {
                            is_big_endian,
                            dimensions,
                            variable_info,
                            data: slicer.read_raw(data_len)?,
                        }),
                        slicer.rest(),
                    )),
                    _ => unreachable!(),
                }
            } else {
                Err(VerboseDecodeError::InvalidTypeInfo(type_info))
            }
        } else if 0 != type_info[0] & BOOL_FLAG_0 {
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
            const CONTRADICTING_MASK_0: u8 = 0b1101_0000;
            const CONTRADICTING_MASK_1: u8 = 0b1110_0111;

            // check that no contradicting type info is present
            if (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            let type_len = type_info[0] & TYPE_LEN_MASK_0;
            match type_len {
                1..=5 => {}
                _ => return Err(InvalidTypeInfo(type_info)),
            }

            // check for varinfo
            let name_and_unit = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name_and_unit(is_big_endian)?)
            } else {
                None
            };

            let var_info = name_and_unit.map(|(name, unit)| VariableInfoUnit { name, unit });

            match type_len {
                1 => Ok((
                    I8(I8Value {
                        variable_info: var_info,
                        scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                        value: slicer.read_i8()?,
                    }),
                    slicer.rest(),
                )),
                2 => Ok((
                    I16(I16Value {
                        variable_info: var_info,
                        scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                        value: slicer.read_i16(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                3 => Ok((
                    I32(I32Value {
                        variable_info: var_info,
                        scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                        value: slicer.read_i32(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                4 => Ok((
                    I64(I64Value {
                        variable_info: var_info,
                        scaling: slicer.read_i64_scaling(is_big_endian, type_info)?,
                        value: slicer.read_i64(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                5 => Ok((
                    I128(I128Value {
                        variable_info: var_info,
                        scaling: slicer.read_i128_scaling(is_big_endian, type_info)?,
                        value: slicer.read_i128(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                _ => unreachable!(),
            }
        } else if 0 != type_info[0] & UNSIGNED_FLAG_0 {
            // verify no conflicting information is present
            const CONTRADICTING_MASK_0: u8 = 0b1011_0000;
            const CONTRADICTING_MASK_1: u8 = 0b1110_0111;

            // check that no contradicting type info is present
            if (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            let type_len = type_info[0] & TYPE_LEN_MASK_0;
            match type_len {
                1..=5 => {}
                _ => return Err(InvalidTypeInfo(type_info)),
            }

            // check for varinfo
            let name_and_unit = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name_and_unit(is_big_endian)?)
            } else {
                None
            };

            let var_info = name_and_unit.map(|(name, unit)| VariableInfoUnit { name, unit });

            match type_len {
                1 => Ok((
                    U8(U8Value {
                        variable_info: var_info,
                        scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                        value: slicer.read_u8()?,
                    }),
                    slicer.rest(),
                )),
                2 => Ok((
                    U16(U16Value {
                        variable_info: var_info,
                        scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                        value: slicer.read_u16(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                3 => Ok((
                    U32(U32Value {
                        variable_info: var_info,
                        scaling: slicer.read_i32_scaling(is_big_endian, type_info)?,
                        value: slicer.read_u32(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                4 => Ok((
                    U64(U64Value {
                        variable_info: var_info,
                        scaling: slicer.read_i64_scaling(is_big_endian, type_info)?,
                        value: slicer.read_u64(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                5 => Ok((
                    U128(U128Value {
                        variable_info: var_info,
                        scaling: slicer.read_i128_scaling(is_big_endian, type_info)?,
                        value: slicer.read_u128(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                _ => unreachable!(),
            }
        } else if 0 != type_info[0] & FLOAT_FLAG_0 {
            // verify no conflicting information is present

            const CONTRADICTING_MASK_0: u8 = 0b0111_0000;
            const CONTRADICTING_MASK_1: u8 = 0b1111_0111;

            // check that no contradicting type info is present
            if (0 != type_info[0] & CONTRADICTING_MASK_0)
                || (0 != type_info[1] & CONTRADICTING_MASK_1)
            {
                return Err(InvalidTypeInfo(type_info));
            }

            let type_len = type_info[0] & TYPE_LEN_MASK_0;
            match type_len {
                2..=5 => {}
                _ => return Err(InvalidTypeInfo(type_info)),
            }

            // check for varinfo
            let name_and_unit = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name_and_unit(is_big_endian)?)
            } else {
                None
            };

            let variable_info = name_and_unit.map(|(name, unit)| VariableInfoUnit { name, unit });

            match type_len {
                2 => Ok((
                    F16(F16Value {
                        variable_info,
                        value: slicer.read_f16(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                3 => Ok((
                    F32(F32Value {
                        variable_info,
                        value: slicer.read_f32(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                4 => Ok((
                    F64(F64Value {
                        variable_info,
                        value: slicer.read_f64(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                5 => Ok((
                    F128(F128Value {
                        variable_info,
                        value: slicer.read_f128(is_big_endian)?,
                    }),
                    slicer.rest(),
                )),
                _ => unreachable!(),
            }
        } else if 0 != type_info[1] & STRING_FLAG_1 {
            const CONTRADICTING_MASK_0: u8 = 0b1111_1111;
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
            let value = match slicer.read_raw(len) {
                Ok(valid_parse) => {
                    if len > 0 {
                        core::str::from_utf8(&valid_parse[..valid_parse.len() - 1])?
                    } else {
                        ""
                    }
                }
                Err(_) => {
                    return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                        layer: error::Layer::VerboseValue,
                        minimum_size: len,
                        actual_size: slicer.rest().len(),
                    }))
                }
            };

            Ok((Str(StringValue { name, value }), slicer.rest()))
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
                Err(_) => {
                    return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                        layer: error::Layer::VerboseValue,
                        minimum_size: len,
                        actual_size: slicer.rest().len(),
                    }))
                }
            };
            Ok((TraceInfo(TraceInfoValue { value: parse? }), slicer.rest()))
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
            let number_of_entries = slicer.read_u16(is_big_endian)?;

            let name = if 0 != type_info[1] & VARINFO_FLAG_1 {
                Some(slicer.read_var_name(is_big_endian)?)
            } else {
                None
            };

            let mut rest = slicer.rest();

            // While this reduces the amount of duplicated code to a minimum, I am not quite sure if this safe as too nested structs could possibly lead to "infinite" recursion
            for _ in 0..number_of_entries {
                (_, rest) = VerboseValue::from_slice(rest, is_big_endian)?;
            }
            let slice_begin = slicer.rest().as_ptr();
            // Rust allocations are ensured to always be smaller than isize::MAX, hence the distance can't result overflow
            // This operation is therefore safe
            let data_len = unsafe { rest.as_ptr().offset_from(slice_begin) as usize };

            Ok((
                Struct(StructValue {
                    is_big_endian,
                    number_of_entries,
                    name,
                    entries_data: slicer.read_raw(data_len)?,
                }),
                slicer.rest(),
            ))
        } else {
            // nothing matches type info uninterpretable
            Err(InvalidTypeInfo(type_info))
        }
    }

    /// Returns the name of the value (if it has one).
    pub fn name(&self) -> Option<&'a str> {
        use VerboseValue::*;

        match self {
            Bool(v) => v.name,
            Str(v) => v.name,
            TraceInfo(_) => None,
            I8(v) => v.variable_info.as_ref().map(|v| v.name),
            I16(v) => v.variable_info.as_ref().map(|v| v.name),
            I32(v) => v.variable_info.as_ref().map(|v| v.name),
            I64(v) => v.variable_info.as_ref().map(|v| v.name),
            I128(v) => v.variable_info.as_ref().map(|v| v.name),
            U8(v) => v.variable_info.as_ref().map(|v| v.name),
            U16(v) => v.variable_info.as_ref().map(|v| v.name),
            U32(v) => v.variable_info.as_ref().map(|v| v.name),
            U64(v) => v.variable_info.as_ref().map(|v| v.name),
            U128(v) => v.variable_info.as_ref().map(|v| v.name),
            F16(v) => v.variable_info.as_ref().map(|v| v.name),
            F32(v) => v.variable_info.as_ref().map(|v| v.name),
            F64(v) => v.variable_info.as_ref().map(|v| v.name),
            F128(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrBool(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrI8(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrI16(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrI32(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrI64(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrI128(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrU8(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrU16(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrU32(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrU64(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrU128(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrF16(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrF32(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrF64(v) => v.variable_info.as_ref().map(|v| v.name),
            ArrF128(v) => v.variable_info.as_ref().map(|v| v.name),
            Struct(v) => v.name,
            Raw(v) => v.name,
        }
    }

    /// Returns the unit of the value (if it has one).
    pub fn unit(&self) -> Option<&'a str> {
        use VerboseValue::*;

        match self {
            Bool(_) => None,
            Str(_) => None,
            TraceInfo(_) => None,
            I8(v) => v.variable_info.as_ref().map(|v| v.unit),
            I16(v) => v.variable_info.as_ref().map(|v| v.unit),
            I32(v) => v.variable_info.as_ref().map(|v| v.unit),
            I64(v) => v.variable_info.as_ref().map(|v| v.unit),
            I128(v) => v.variable_info.as_ref().map(|v| v.unit),
            U8(v) => v.variable_info.as_ref().map(|v| v.unit),
            U16(v) => v.variable_info.as_ref().map(|v| v.unit),
            U32(v) => v.variable_info.as_ref().map(|v| v.unit),
            U64(v) => v.variable_info.as_ref().map(|v| v.unit),
            U128(v) => v.variable_info.as_ref().map(|v| v.unit),
            F16(v) => v.variable_info.as_ref().map(|v| v.unit),
            F32(v) => v.variable_info.as_ref().map(|v| v.unit),
            F64(v) => v.variable_info.as_ref().map(|v| v.unit),
            F128(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrBool(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrI8(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrI16(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrI32(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrI64(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrI128(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrU8(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrU16(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrU32(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrU64(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrU128(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrF16(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrF32(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrF64(v) => v.variable_info.as_ref().map(|v| v.unit),
            ArrF128(v) => v.variable_info.as_ref().map(|v| v.unit),
            Struct(_) => None,
            Raw(_) => None,
        }
    }
}
