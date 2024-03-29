use arrayvec::{ArrayVec, CapacityError};

use crate::verbose::VariableInfoUnit;

/// Verbose 32 bit float number.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct F32Value<'a> {
    pub variable_info: Option<VariableInfoUnit<'a>>,
    pub value: f32,
}

impl<'a> F32Value<'a> {
    /// Adds the verbose value to the given dlt mesage buffer.
    pub fn add_to_msg<const CAP: usize>(
        &self,
        buf: &mut ArrayVec<u8, CAP>,
        is_big_endian: bool,
    ) -> Result<(), CapacityError> {
        if let Some(var_info) = &self.variable_info {
            let type_info = [0b1000_0011, 0b0000_1000, 0b0000_0000, 0b0000_0000];
            let name_len;
            let unit_len;
            if is_big_endian {
                name_len = (var_info.name.len() as u16 + 1).to_be_bytes();
                unit_len = (var_info.unit.len() as u16 + 1).to_be_bytes();
            } else {
                name_len = (var_info.name.len() as u16 + 1).to_le_bytes();
                unit_len = (var_info.unit.len() as u16 + 1).to_le_bytes();
            };
            buf.try_extend_from_slice(&type_info)?;
            buf.try_extend_from_slice(&[name_len[0], name_len[1], unit_len[0], unit_len[1]])?;
            buf.try_extend_from_slice(var_info.name.as_bytes())?;
            if buf.remaining_capacity() > var_info.unit.len() + 2 {
                // Safe as capacity is checked earlier
                unsafe { buf.push_unchecked(0) };
                buf.try_extend_from_slice(var_info.unit.as_bytes())?;
                unsafe { buf.push_unchecked(0) };
            } else {
                return Err(CapacityError::new(()));
            }
        } else {
            let type_info = [0b1000_0011, 0b0000_0000, 0b0000_0000, 0b0000_0000];
            buf.try_extend_from_slice(&type_info)?;
        }

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
    use crate::verbose::VerboseValue::F32;
    use alloc::vec::Vec;
    use proptest::prelude::*;
    use std::format;

    proptest! {
        #[test]
        fn write_read(value in any::<f32>(), ref name in "\\pc{0,20}", ref unit in "\\pc{0,20}") {
            const MAX_SYMBOL_LENGTH_NAME: usize = 20;
            const MAX_SYMBOL_LENGTH_UNIT: usize = 20;
            const BYTES_NEEDED: usize = 8;
            const BYTES_NEEDED_WITH_NAME: usize = 6 + BYTES_NEEDED;

            // The buffer needs to be sized the (max len of the name + max len unit) * 4 + 14 Byte. (14 Byte: 4 Byte TypeInfo + 2 Bytes Length of Name + 1 Byte Null Terminator of Name + 2 Byte unit Length + 1 Byte Null Terminator Unit + 4 Byte Data)
            // As Proptest only generates chars by characters (which can be up to 4 bytes), the buffer needs to be 4 * len of name
            const BUFFER_SIZE: usize = MAX_SYMBOL_LENGTH_NAME * 4 + MAX_SYMBOL_LENGTH_UNIT * 4 + BYTES_NEEDED_WITH_NAME;

            // test big endian with name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let slice_len = name.len() + unit.len() + BYTES_NEEDED_WITH_NAME;
                let is_big_endian = true;

                let variable_info = Some(VariableInfoUnit { name , unit });

                let f32_value = F32Value {variable_info, value};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_name_be = (name.len() as u16 + 1).to_be_bytes();
                let len_unit_be = (unit.len() as u16 + 1).to_be_bytes();

                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b1000_0011, 0b0000_1000, 0b0000_0000, 0b0000_0000, len_name_be[0], len_name_be[1], len_unit_be[0], len_unit_be[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(unit.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(&value.to_be_bytes());

                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((F32(f32_value),&[] as &[u8])));
            }

            // test little endian with name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let slice_len = name.len() + unit.len() + BYTES_NEEDED_WITH_NAME;
                let is_big_endian = false;

                let variable_info = Some(VariableInfoUnit { name , unit });

                let f32_value = F32Value {variable_info, value};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_name_le = (name.len() as u16 + 1).to_le_bytes();
                let len_unit_le = (unit.len() as u16 + 1).to_le_bytes();

                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b1000_0011, 0b0000_1000, 0b0000_0000, 0b0000_0000, len_name_le[0], len_name_le[1], len_unit_le[0], len_unit_le[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(unit.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(&value.to_le_bytes());

                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((F32(f32_value),&[] as &[u8])));
            }

            // test big endian without name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let slice_len = BYTES_NEEDED;
                let is_big_endian = true;

                let variable_info = None;

                let f32_value = F32Value {variable_info, value};
                let mut content_buff = Vec::with_capacity(slice_len);

                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b1000_0011, 0b0000_0000, 0b0000_0000, 0b0000_0000]);
                content_buff.extend_from_slice(&value.to_be_bytes());

                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((F32(f32_value),&[] as &[u8])));
            }

            // test little endian without name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let slice_len = BYTES_NEEDED;
                let is_big_endian = false;

                let variable_info = None;

                let f32_value = F32Value {variable_info, value};
                let mut content_buff = Vec::with_capacity(slice_len);

                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b1000_0011, 0b0000_0000, 0b0000_0000, 0b0000_0000]);
                content_buff.extend_from_slice(&value.to_le_bytes());

                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((F32(f32_value),&[] as &[u8])));
            }

            // Capacity error big endian with name
            {
                const SLICE_LEN: usize = BYTES_NEEDED_WITH_NAME-1;
                let variable_info = Some(VariableInfoUnit { name , unit });

                let f32_value = F32Value {variable_info, value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error little endian with name
            {
                const SLICE_LEN: usize = BYTES_NEEDED_WITH_NAME-1;
                let variable_info = Some(VariableInfoUnit { name , unit });

                let f32_value = F32Value {variable_info, value};
                let is_big_endian = false;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error big endian without name
            {
                const SLICE_LEN: usize = BYTES_NEEDED - 1;
                let variable_info = None;

                let f32_value = F32Value {variable_info, value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error little endian without name
            {
                const SLICE_LEN: usize = BYTES_NEEDED - 1;
                let variable_info = None;

                let f32_value = F32Value {variable_info, value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(f32_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

        }
    }
}
