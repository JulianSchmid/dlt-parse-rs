use arrayvec::{ArrayVec, CapacityError};

#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RawValue<'a> {
    pub name: Option<&'a str>,
    pub(crate) data: &'a [u8],
}

impl<'a> RawValue<'a> {
    /// Adds the verbose value to the given dlt mesage buffer.
    pub fn add_to_msg<const CAP: usize>(
        &self,
        buf: &mut ArrayVec<u8, CAP>,
        is_big_endian: bool,
    ) -> Result<(), CapacityError> {
        if let Some(name) = self.name {
            let type_info = [0b0000_0000, 0b0000_1100, 0b0000_0000, 0b0000_0000];
            let (data_len, name_len) = match is_big_endian {
                true => (
                    (self.data.len() as u16).to_be_bytes(),
                    (name.len() as u16 + 1).to_be_bytes(),
                ),
                false => (
                    (self.data.len() as u16).to_le_bytes(),
                    (name.len() as u16 + 1).to_le_bytes(),
                ),
            };
            buf.try_extend_from_slice(&type_info)?;
            buf.try_extend_from_slice(&[data_len[0], data_len[1], name_len[0], name_len[1]])?;
            buf.try_extend_from_slice(name.as_bytes())?;
            if buf.remaining_capacity() > 0 {
                // Safe as capacity is checked earlier
                unsafe { buf.push_unchecked(0) };
            } else {
                return Err(CapacityError::new(()));
            }
        } else {
            let type_info = [0b0000_0000, 0b0000_0100, 0b0000_0000, 0b0000_0000];
            let data_len = match is_big_endian {
                true => (self.data.len() as u16).to_be_bytes(),
                false => (self.data.len() as u16).to_le_bytes(),
            };
            buf.try_extend_from_slice(&type_info)?;
            buf.try_extend_from_slice(&[data_len[0], data_len[1]])?;
        }

        buf.try_extend_from_slice(self.data)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::verbose::VerboseValue;
    use crate::verbose::VerboseValue::Raw;
    use alloc::vec::Vec;
    use proptest::prelude::*;
    use std::format;

    proptest! {
        #[test]
        fn write_read(ref data in "\\pc{0,80}", ref name in "\\pc{0,20}") {
            const MAX_SYMBOL_LENGTH_NAME: usize = 20;
            const MAX_SYMBOL_LENGTH_VALUE: usize = 80;
            const BYTES_NEEDED: usize = 6;
            const BYTES_NEEDED_WITH_NAME: usize = 3 + BYTES_NEEDED;

            const BUFFER_SIZE: usize = MAX_SYMBOL_LENGTH_NAME * 4 + MAX_SYMBOL_LENGTH_VALUE * 4 + BYTES_NEEDED_WITH_NAME;

            let data = data.as_bytes();
            // test big endian with name
            {

                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let slice_len = name.len() + data.len() + BYTES_NEEDED_WITH_NAME;
                let is_big_endian = true;

                let raw_value = RawValue {name: Some(name), data};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_name_be = (name.len() as u16 + 1).to_be_bytes();
                let len_value_be = (data.len() as u16).to_be_bytes();

                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0000_0000, 0b0000_1100, 0b0000_0000, 0b0000_0000, len_value_be[0], len_value_be[1], len_name_be[0], len_name_be[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(data);
                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((Raw(raw_value),&[] as &[u8])));
            }

            // test little endian with name
            {

                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let slice_len = name.len() + data.len() + BYTES_NEEDED_WITH_NAME;
                let is_big_endian = false;

                let raw_value = RawValue {name: Some(name), data};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_name_le = (name.len() as u16 + 1).to_le_bytes();
                let len_value_le = (data.len() as u16).to_le_bytes();

                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0000_0000, 0b0000_1100, 0b0000_0000, 0b0000_0000, len_value_le[0], len_value_le[1], len_name_le[0], len_name_le[1]]);
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.extend_from_slice(data);
                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((Raw(raw_value),&[] as &[u8])));
            }

            // test big endian without name
            {

                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let slice_len = data.len() + BYTES_NEEDED;
                let is_big_endian = true;

                let raw_value = RawValue {name: None, data};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_value_be = (data.len() as u16).to_be_bytes();

                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0000_0000, 0b0000_0100, 0b0000_0000, 0b0000_0000, len_value_be[0], len_value_be[1]]);
                content_buff.extend_from_slice(data);
                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((Raw(raw_value),&[] as &[u8])));
            }

            // test little endian without name
            {

                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let slice_len = data.len() + BYTES_NEEDED;
                let is_big_endian = false;

                let raw_value = RawValue {name: None, data};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_value_le = (data.len() as u16).to_le_bytes();

                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0000_0000, 0b0000_0100, 0b0000_0000, 0b0000_0000, len_value_le[0], len_value_le[1]]);
                content_buff.extend_from_slice(data);
                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((Raw(raw_value),&[] as &[u8])));
            }

             // Capacity error big endian with name
             {
                const SLICE_LEN: usize = BYTES_NEEDED_WITH_NAME-1;

                let raw_value = RawValue {name: Some(name), data};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error little endian with name
            {
                const SLICE_LEN: usize = BYTES_NEEDED_WITH_NAME-1;

                let raw_value = RawValue {name: Some(name), data};
                let is_big_endian = false;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error big endian without name
            {
                const SLICE_LEN: usize = BYTES_NEEDED - 1;

                let raw_value = RawValue {name: None, data};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error little endian without name
            {
                const SLICE_LEN: usize = BYTES_NEEDED - 1;

                let raw_value = RawValue {name: None, data};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(raw_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }


        }
    }
}