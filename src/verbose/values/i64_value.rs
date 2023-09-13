use arrayvec::{ArrayVec, CapacityError};

use crate::verbose::{Scaling, VariableInfoUnit};

/// Verbose 64 bit signed integer.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct I64Value<'a> {
    pub variable_info: Option<VariableInfoUnit<'a>>,
    pub scaling: Option<Scaling<i64>>,
    pub value: i64,
}

impl<'a> I64Value<'a> {
    /// Adds the verbose value to the given dlt mesage buffer.
    pub fn add_to_msg<const CAP: usize>(
        &self,
        buf: &mut ArrayVec<u8, CAP>,
        is_big_endian: bool,
    ) -> Result<(), CapacityError> {
        if let Some(var_info) = &self.variable_info {
            let name_len;
            let unit_len;
            if is_big_endian {
                name_len = (var_info.name.len() as u16 + 1).to_be_bytes();
                unit_len = (var_info.unit.len() as u16 + 1).to_be_bytes();
            } else {
                name_len = (var_info.name.len() as u16 + 1).to_le_bytes();
                unit_len = (var_info.unit.len() as u16 + 1).to_le_bytes();
            };

            if let Some(scaler) = &self.scaling {
                let type_info = [0b0010_0100, 0b0001_1000, 0b0000_0000, 0b0000_0000];
                let quantization;
                let offset: [u8; 8];

                if is_big_endian {
                    quantization = scaler.quantization.to_be_bytes();
                    offset = scaler.offset.to_be_bytes();
                } else {
                    quantization = scaler.quantization.to_le_bytes();
                    offset = scaler.offset.to_le_bytes();
                }
                buf.try_extend_from_slice(&type_info)?;
                buf.try_extend_from_slice(&[name_len[0], name_len[1], unit_len[0], unit_len[1]])?;
                buf.try_extend_from_slice(var_info.name.as_bytes())?;
                if buf.remaining_capacity() > var_info.unit.len() + 2 {
                    // Safe as capacity is checked earlier
                    unsafe { buf.push_unchecked(0) };
                    let _ = buf.try_extend_from_slice(var_info.unit.as_bytes());
                    unsafe { buf.push_unchecked(0) };
                } else {
                    return Err(CapacityError::new(()));
                }

                buf.try_extend_from_slice(&quantization)?;
                buf.try_extend_from_slice(&offset)?;
            } else {
                let type_info = [0b0010_0100, 0b0000_1000, 0b0000_0000, 0b0000_0000];

                buf.try_extend_from_slice(&type_info)?;
                buf.try_extend_from_slice(&[name_len[0], name_len[1], unit_len[0], unit_len[1]])?;
                buf.try_extend_from_slice(var_info.name.as_bytes())?;
                if buf.remaining_capacity() > var_info.unit.len() + 2 {
                    // Safe as capacity is checked earlier
                    unsafe { buf.push_unchecked(0) };
                    let _ = buf.try_extend_from_slice(var_info.unit.as_bytes());
                    unsafe { buf.push_unchecked(0) };
                } else {
                    return Err(CapacityError::new(()));
                }
            }
        }
        // No name & unit
        else if let Some(scaler) = &self.scaling {
            let type_info: [u8; 4] = [0b0010_0100, 0b0001_0000, 0b0000_0000, 0b0000_0000];

            let quantization;
            let offset: [u8; 8];
            if is_big_endian {
                quantization = scaler.quantization.to_be_bytes();
                offset = scaler.offset.to_be_bytes();
            } else {
                quantization = scaler.quantization.to_le_bytes();
                offset = scaler.offset.to_le_bytes();
            }
            buf.try_extend_from_slice(&type_info)?;
            buf.try_extend_from_slice(&quantization)?;
            buf.try_extend_from_slice(&offset)?;
        } else {
            let type_info: [u8; 4] = [0b0010_0100, 0b0000_0000, 0b0000_0000, 0b0000_0000];
            buf.try_extend_from_slice(&type_info)?;
        }

        // value
        if is_big_endian {
            buf.try_extend_from_slice(&self.value.to_be_bytes())
        } else {
            buf.try_extend_from_slice(&self.value.to_le_bytes())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::verbose::VerboseValue;
    use crate::verbose::VerboseValue::I64;
    use alloc::vec::Vec;
    use proptest::arbitrary::any;
    use proptest::prelude::*;
    use std::format;

    proptest! {
        #[test]
        fn write_read(value in any::<i64>(), ref name in "\\pc{0,20}", ref unit in "\\pc{0,20}", quantization in any::<f32>(), offset in any::<i64>()) {
            const MAX_SYMBOL_LENGTH_NAME: usize = 20;
            const MAX_SYMBOL_LENGTH_UNIT: usize = 20;
            const FIXED_POINT_LENGTH: usize = 12;
            const BYTES_NEEDED: usize = 12;
            const BYTES_NEEDED_WITH_NAME: usize = 6 + BYTES_NEEDED;

            // The buffer needs to be sized the (max len of the name + max len unit) * 4 + 11 bits. (11 Byte: 4 Byte TypeInfo + 2 Bytes Length of Name + 1 Byte Null Terminator of Name + 2 Byte unit Length + 1 Byte Null Terminator Unit + 1 Byte Data)
            // As Proptest only generates chars by characters (which can be up to 4 bytes), the buffer needs to be 4 * len of name
            const BUFFER_SIZE_NO_FIXED_POINT: usize = MAX_SYMBOL_LENGTH_NAME * 4 + MAX_SYMBOL_LENGTH_UNIT * 4 + BYTES_NEEDED_WITH_NAME;
            const BUFFER_SIZE_FIXED_POINT: usize = BUFFER_SIZE_NO_FIXED_POINT + FIXED_POINT_LENGTH;

            // test big endian with name and fixed point
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE_FIXED_POINT> = ArrayVec::new();
                let slice_len = name.len() + unit.len() + FIXED_POINT_LENGTH + BYTES_NEEDED_WITH_NAME;
                let is_big_endian = true;

                let variable_info = Some(VariableInfoUnit { name , unit });
                let scaling = Some(Scaling { quantization, offset });

                let i64_value = I64Value {variable_info, scaling, value};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_name_be = (name.len() as u16 + 1).to_be_bytes();
                let len_unit_be = (unit.len() as u16 + 1).to_be_bytes();

                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0010_0100, 0b0001_1000, 0b0000_0000, 0b0000_0000, len_name_be[0], len_name_be[1], len_unit_be[0], len_unit_be[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(unit.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(&quantization.to_be_bytes());
                content_buff.extend_from_slice(&offset.to_be_bytes());
                content_buff.extend_from_slice(&value.to_be_bytes());

                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((I64(i64_value),&[] as &[u8])));

            }

                        // test big endian with name and fixed point
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE_FIXED_POINT> = ArrayVec::new();
                let slice_len = name.len() + unit.len() + FIXED_POINT_LENGTH + BYTES_NEEDED_WITH_NAME;
                let is_big_endian = true;

                let variable_info = Some(VariableInfoUnit { name , unit });
                let scaling = Some(Scaling { quantization, offset });

                let i64_value = I64Value {variable_info, scaling, value};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_name_be = (name.len() as u16 + 1).to_be_bytes();
                let len_unit_be = (unit.len() as u16 + 1).to_be_bytes();

                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0010_0100, 0b0001_1000, 0b0000_0000, 0b0000_0000, len_name_be[0], len_name_be[1], len_unit_be[0], len_unit_be[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(unit.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(&quantization.to_be_bytes());
                content_buff.extend_from_slice(&offset.to_be_bytes());
                content_buff.extend_from_slice(&value.to_be_bytes());

                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((I64(i64_value),&[] as &[u8])));

            }
            // test little endian with name and fixed point
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE_FIXED_POINT> = ArrayVec::new();
                let slice_len = name.len() + unit.len() + FIXED_POINT_LENGTH + BYTES_NEEDED_WITH_NAME;
                let is_big_endian = false;

                let variable_info = Some(VariableInfoUnit { name , unit });
                let scaling = Some(Scaling { quantization, offset });

                let i64_value = I64Value {variable_info, scaling, value};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_name_le = (name.len() as u16 + 1).to_le_bytes();
                let len_unit_le = (unit.len() as u16 + 1).to_le_bytes();

                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0010_0100, 0b0001_1000, 0b0000_0000, 0b0000_0000, len_name_le[0], len_name_le[1], len_unit_le[0], len_unit_le[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(unit.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(&quantization.to_le_bytes());
                content_buff.extend_from_slice(&offset.to_le_bytes());
                content_buff.extend_from_slice(&value.to_le_bytes());

                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((I64(i64_value),&[] as &[u8])));

            }

            // test big endian with name without fixed point
          {
            let mut msg_buff: ArrayVec<u8, BUFFER_SIZE_NO_FIXED_POINT> = ArrayVec::new();
            let slice_len = name.len() + unit.len() + BYTES_NEEDED_WITH_NAME;
            let is_big_endian = true;

            let variable_info = Some(VariableInfoUnit { name , unit });
            let scaling = None;

            let i64_value = I64Value {variable_info, scaling, value};
            let mut content_buff = Vec::with_capacity(slice_len);

            let len_name_be = (name.len() as u16 + 1).to_be_bytes();
            let len_unit_be = (unit.len() as u16 + 1).to_be_bytes();

            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

            content_buff.extend_from_slice(&[0b0010_0100, 0b0000_1000, 0b0000_0000, 0b0000_0000, len_name_be[0], len_name_be[1], len_unit_be[0], len_unit_be[1]]);
            content_buff.extend_from_slice(name.as_bytes());
            content_buff.push(0);
            content_buff.extend_from_slice(unit.as_bytes());
            content_buff.push(0);
            content_buff.extend_from_slice(&value.to_be_bytes());

            prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

            // Now wrap back
            let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
            prop_assert_eq!(parsed_back, Ok((I64(i64_value),&[] as &[u8])));

         }

            // test little endian with name without fixed point
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE_NO_FIXED_POINT> = ArrayVec::new();
                let slice_len = name.len() + unit.len() + BYTES_NEEDED_WITH_NAME;
                let is_big_endian = false;

                let variable_info = Some(VariableInfoUnit { name , unit });
                let scaling = None;

                let i64_value = I64Value {variable_info, scaling, value};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_name_le = (name.len() as u16 + 1).to_le_bytes();
                let len_unit_le = (unit.len() as u16 + 1).to_le_bytes();

                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0010_0100, 0b0000_1000, 0b0000_0000, 0b0000_0000, len_name_le[0], len_name_le[1], len_unit_le[0], len_unit_le[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(unit.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(&value.to_le_bytes());

                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((I64(i64_value),&[] as &[u8])));

             }


        // test big endian without name, unit and without fixed point
        {
            let mut msg_buff: ArrayVec<u8, BUFFER_SIZE_NO_FIXED_POINT> = ArrayVec::new();
            let slice_len = BYTES_NEEDED;
            let is_big_endian = true;

            let variable_info = None;
            let scaling = None;

            let i64_value = I64Value {variable_info, scaling, value};
            let mut content_buff = Vec::with_capacity(slice_len);

            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

            content_buff.extend_from_slice(&[0b0010_0100, 0b0000_0000, 0b0000_0000, 0b0000_0000]);
            content_buff.extend_from_slice(&value.to_be_bytes());

            prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

            // Now wrap back
            let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
            prop_assert_eq!(parsed_back, Ok((I64(i64_value),&[] as &[u8])));

            }

        // test little endian without name, unit and without fixed point
        {
            let mut msg_buff: ArrayVec<u8, BUFFER_SIZE_NO_FIXED_POINT> = ArrayVec::new();
            let slice_len = BYTES_NEEDED;
            let is_big_endian = false;

            let variable_info = None;
            let scaling = None;

            let i64_value = I64Value {variable_info, scaling, value};
            let mut content_buff = Vec::with_capacity(slice_len);

            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

            content_buff.extend_from_slice(&[0b0010_0100, 0b0000_0000, 0b0000_0000, 0b0000_0000]);
            content_buff.extend_from_slice(&value.to_le_bytes());

            prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

            // Now wrap back
            let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
            prop_assert_eq!(parsed_back, Ok((I64(i64_value),&[] as &[u8])));

            }


        // test big endian with fixed point and without name
        {
            let mut msg_buff: ArrayVec<u8, BUFFER_SIZE_FIXED_POINT> = ArrayVec::new();
            let slice_len = FIXED_POINT_LENGTH + BYTES_NEEDED;
            let is_big_endian = true;

            let variable_info = None;
            let scaling = Some(Scaling { quantization, offset });

            let i64_value = I64Value {variable_info, scaling, value};
            let mut content_buff = Vec::with_capacity(slice_len);

            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

            content_buff.extend_from_slice(&[0b0010_0100, 0b0001_0000, 0b0000_0000, 0b0000_0000]);
            content_buff.extend_from_slice(&quantization.to_be_bytes());
            content_buff.extend_from_slice(&offset.to_be_bytes());
            content_buff.extend_from_slice(&value.to_be_bytes());

            prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

            // Now wrap back
            let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
            prop_assert_eq!(parsed_back, Ok((I64(i64_value),&[] as &[u8])));

        }

        // test little endian with fixed point and without name
        {
            let mut msg_buff: ArrayVec<u8, BUFFER_SIZE_FIXED_POINT> = ArrayVec::new();
            let slice_len = FIXED_POINT_LENGTH + BYTES_NEEDED;
            let is_big_endian = false;

            let variable_info = None;
            let scaling = Some(Scaling { quantization, offset });

            let i64_value = I64Value {variable_info, scaling, value};
            let mut content_buff = Vec::with_capacity(slice_len);

            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

            content_buff.extend_from_slice(&[0b0010_0100, 0b0001_0000, 0b0000_0000, 0b0000_0000]);
            content_buff.extend_from_slice(&quantization.to_le_bytes());
            content_buff.extend_from_slice(&offset.to_le_bytes());
            content_buff.extend_from_slice(&value.to_le_bytes());

            prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

            // Now wrap back
            let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
            prop_assert_eq!(parsed_back, Ok((I64(i64_value),&[] as &[u8])));

        }

         // Capacity error big endian with name and scaling
         {
            const SLICE_LEN: usize = FIXED_POINT_LENGTH + BYTES_NEEDED_WITH_NAME - 1;
            let variable_info = Some(VariableInfoUnit { name , unit });
            let scaling = Some(Scaling { quantization, offset });

            let i64_value = I64Value {variable_info, scaling, value};
            let is_big_endian = true;

            let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

        }

        // Capacity error little endian with name and scaling
        {
            const SLICE_LEN: usize = FIXED_POINT_LENGTH + BYTES_NEEDED_WITH_NAME - 1;
            let variable_info = Some(VariableInfoUnit { name , unit });
            let scaling = Some(Scaling { quantization, offset });

            let i64_value = I64Value {variable_info, scaling, value};
            let is_big_endian = false;

            let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

        }

        // Capacity error big endian with name, without scaling
        {
            const SLICE_LEN: usize = BYTES_NEEDED_WITH_NAME - 1;
            let variable_info = Some(VariableInfoUnit { name , unit });
            let scaling = None;

            let i64_value = I64Value {variable_info, scaling, value};
            let is_big_endian = true;

            let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

        }

        // Capacity error big endian with name, without scaling
        {
            const SLICE_LEN: usize = BYTES_NEEDED_WITH_NAME - 1;
            let variable_info = Some(VariableInfoUnit { name , unit });
            let scaling = None;

            let i64_value = I64Value {variable_info, scaling, value};
            let is_big_endian = false;

            let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

        }

        // Capacity error big endian without name, without scaling
        {
            const SLICE_LEN: usize = BYTES_NEEDED - 1;
            let variable_info = None;
            let scaling = None;

            let i64_value = I64Value {variable_info, scaling, value};
            let is_big_endian = true;

            let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

        }

        // Capacity error little endian without name, without scaling
        {
            const SLICE_LEN: usize = BYTES_NEEDED - 1;
            let variable_info = None;
            let scaling = None;

            let i64_value = I64Value {variable_info, scaling, value};
            let is_big_endian = false;

            let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

        }

        // Capacity error big endian with scaling and without name
        {
            const SLICE_LEN: usize = FIXED_POINT_LENGTH + BYTES_NEEDED - 1;
            let variable_info = None;
            let scaling = Some(Scaling { quantization, offset });

            let i64_value = I64Value {variable_info, scaling, value};
            let is_big_endian = true;

            let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

        }

        // Capacity error little endian with scaling and without name
        {
            const SLICE_LEN: usize = FIXED_POINT_LENGTH + BYTES_NEEDED - 1;
            let variable_info = None;
            let scaling = Some(Scaling { quantization, offset });

            let i64_value = I64Value {variable_info, scaling, value};
            let is_big_endian = false;

            let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
            prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

        }


            // Capacity error 2 with name & scaling: Type info only
            {
                const SLICE_LEN: usize = 4; // Only type info fits in
                let variable_info = Some(VariableInfoUnit { name , unit });
                let scaling = Some(Scaling { quantization, offset });

                let i64_value = I64Value {variable_info, scaling, value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error 2 without name & scaling: Type info only
            {
                const SLICE_LEN: usize = 4; // Only type info fits in
                let variable_info = Some(VariableInfoUnit { name , unit });
                let scaling = None;

                let i64_value = I64Value {variable_info, scaling, value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error 3: With name & unit: Offset does not fit in
            {
                const SLICE_LEN: usize = BYTES_NEEDED + FIXED_POINT_LENGTH + 5;
                let variable_info = Some(VariableInfoUnit { name:"Abc" , unit:"defg" });
                let scaling = Some(Scaling { quantization, offset });

                let i64_value = I64Value {variable_info, scaling, value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error 3: Without name & unit: Offset does not fit in
            {
                const SLICE_LEN: usize = BYTES_NEEDED + FIXED_POINT_LENGTH - 9;
                let variable_info = None;
                let scaling = Some(Scaling { quantization, offset });

                let i64_value = I64Value {variable_info, scaling, value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error 4: With name & unit: Quantization does not fit in
            {
                const SLICE_LEN: usize = 17;
                let variable_info = Some(VariableInfoUnit { name:"Abc" , unit:"defg" });
                let scaling = Some(Scaling { quantization, offset });

                let i64_value = I64Value {variable_info, scaling, value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error 4: Without name & unit: Quantization does not fit in
            {
                const SLICE_LEN: usize = 7;
                let variable_info = None;
                let scaling = Some(Scaling { quantization, offset });

                let i64_value = I64Value {variable_info, scaling, value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(i64_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

        }
    }

    proptest! {
        #[test]
        fn debug(value in any::<i64>(), ref name in "\\pc{0,20}", ref unit in "\\pc{0,20}", quantization in any::<f32>(), offset in any::<i64>()) {
            { // Test with name, unit & scaling
            let value_struct: I64Value = I64Value {variable_info: Some(VariableInfoUnit { name, unit }), scaling: Some(Scaling { quantization, offset }), value};
            assert_eq!(
                    format!(
                        "I64Value {{ variable_info: {:?}, scaling: {:?}, value: {:?} }}",
                        value_struct.variable_info,
                        value_struct.scaling,
                        value_struct.value,
                    ),
                    format!("{:?}", value_struct)
                );
            }

            { // Test with name, unit & without scaling
                let value_struct: I64Value = I64Value {variable_info: Some(VariableInfoUnit { name, unit }), scaling: None, value};
                assert_eq!(
                        format!(
                            "I64Value {{ variable_info: {:?}, scaling: {:?}, value: {:?} }}",
                            value_struct.variable_info,
                            value_struct.scaling,
                            value_struct.value,
                        ),
                        format!("{:?}", value_struct)
                    );
                }

            { // Test without name, unit & without scaling
                let value_struct: I64Value = I64Value {variable_info: None, scaling: None, value};
                assert_eq!(
                        format!(
                            "I64Value {{ variable_info: {:?}, scaling: {:?}, value: {:?} }}",
                            value_struct.variable_info,
                            value_struct.scaling,
                            value_struct.value,
                        ),
                        format!("{:?}", value_struct)
                    );
                }

           { // Test with scaling, but without name & unit
            let value_struct: I64Value = I64Value {variable_info: None, scaling: Some(Scaling { quantization, offset }), value};
            assert_eq!(
                    format!(
                        "I64Value {{ variable_info: {:?}, scaling: {:?}, value: {:?} }}",
                        value_struct.variable_info,
                        value_struct.scaling,
                        value_struct.value,
                    ),
                    format!("{:?}", value_struct)
                );
            }
        }
    }
}
