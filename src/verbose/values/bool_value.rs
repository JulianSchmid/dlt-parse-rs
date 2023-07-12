use arrayvec::{ArrayVec, CapacityError};

#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BoolValue<'a> {
    pub name: Option<&'a str>,
    pub value: bool,
}

impl<'a> BoolValue<'a> {
    /// Adds the verbose value to the given dlt mesage buffer.
    pub fn add_to_msg<const CAP: usize>(
        &self,
        buf: &mut ArrayVec<u8, CAP>,
        is_big_endian: bool,
    ) -> Result<(), CapacityError> {
        if self.name.is_some() {
            let name = unsafe { self.name.unwrap_unchecked() };
            let type_info: [u8; 4] = [0b0001_0001, 0b0000_1000, 0b0000_0000, 0b0000_0000];
            buf.try_extend_from_slice(&type_info)?;

            let name_len = if is_big_endian {
                (name.len() as u16 + 1).to_be_bytes()
            } else {
                (name.len() as u16 + 1).to_le_bytes()
            };
            buf.try_extend_from_slice(&[name_len[0], name_len[1]])?;
            buf.try_extend_from_slice(name.as_bytes())?;
            if buf.remaining_capacity() > 1 {
                // Safe as capacity is checked earlier
                unsafe { buf.push_unchecked(0) };
                unsafe { buf.push_unchecked(u8::from(self.value)) }
                Ok(())
            } else {
                Err(CapacityError::new(()))
            }
        } else {
            let type_info: [u8; 4] = [0b0001_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000];
            buf.try_extend_from_slice(&type_info)?;

            if buf.remaining_capacity() > 0 {
                // Safe as capacity is checked earlier
                unsafe { buf.push_unchecked(u8::from(self.value)) }
                Ok(())
            } else {
                Err(CapacityError::new(()))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::verbose::VerboseValue;
    use crate::verbose::VerboseValue::Bool;
    use alloc::vec::Vec;
    use proptest::arbitrary::any;
    use proptest::prelude::*;
    use std::format;

    proptest! {
        #[test]
        fn write_read(value in any::<bool>(), ref name in "\\pc{1,20}") {
            const MAX_SYMBOL_LENGTH: usize = 20;
            const BYTES_NEEDED: usize = 8;
            // The buffer needs to be sized the max len of the name * 4 + 8 bits. (8 Byte: 4 Byte TypeInfo + 2 Bytes Length of Name + 1 Byte Null Terminator of Name + 1 Byte Data)
            // As Proptest only generates chars by characters (which can be up to 4 bytes), the buffer needs to be 4 * len of name
            const BUFFER_SIZE: usize = MAX_SYMBOL_LENGTH * 4 + BYTES_NEEDED;

           // test big endian with name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

                let bool_val = BoolValue { name: Some(name), value };
                let slice_len = name.len() + BYTES_NEEDED;
                let mut content_buff = Vec::with_capacity(slice_len);
                let is_big_endian = true;
                let len_be = (name.len() as u16 + 1).to_be_bytes();
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));



                content_buff.extend_from_slice(&[0b0001_0001, 0b0000_1000, 0b0000_0000, 0b0000_0000, len_be[0], len_be[1]]);
                // for b in name.as_bytes() {
                //     content_buff.push(*b);
                // }
                content_buff.extend_from_slice(name.as_bytes());
                content_buff.push(0);
                content_buff.push(u8::from(value));

                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((Bool(bool_val),&[] as &[u8])));

            }

            // Test little endian with name
            {
                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let bool_val = BoolValue { name: Some(name), value };
                let slice_len = name.len() + BYTES_NEEDED;
                let mut content_buff = Vec::with_capacity(slice_len);
                let is_big_endian = false;
                let len_le = (name.len() as u16 + 1).to_le_bytes();

                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0001_0001, 0b0000_1000, 0b0000_0000, 0b0000_0000, len_le[0], len_le[1]]);
                for b in name.as_bytes() {
                    content_buff.push(*b);
                }
                content_buff.push(0);
                content_buff.push(u8::from(value));

                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((Bool(bool_val),&[] as &[u8])));

            }


            // Test big endian without name
            {
                let mut msg_buff: ArrayVec<u8, 5> = ArrayVec::new();

                let bool_val = BoolValue { name: None, value };
                let slice_len = 5;
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, true), Ok(()));

                let expected = &[0b0001_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000, u8::from(value)];
                prop_assert_eq!(&msg_buff[..slice_len], expected);

                // Now wrap back
                let parsed_back_be = VerboseValue::from_slice(&msg_buff, true);
                prop_assert_eq!(parsed_back_be, Ok((Bool(bool_val),&[] as &[u8])));

            }

            // Test little endian without name
            {
                let mut msg_buff: ArrayVec<u8, 5> = ArrayVec::new();

                let bool_val = BoolValue { name: None, value };
                let slice_len = 5;
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, false), Ok(()));

                let expected = &[0b0001_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000, u8::from(value)];
                prop_assert_eq!(&msg_buff[..slice_len], expected);

                // Now wrap back
                let parsed_back_le = VerboseValue::from_slice(&msg_buff, false);
                prop_assert_eq!(parsed_back_le, Ok((Bool(bool_val),&[] as &[u8])));

            }

             // Capacity error big endian with name
             {
                let bool_val = BoolValue { name: Some(name), value };
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, 7> = ArrayVec::new();
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 7> = ArrayVec::new();
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error little endian with name
            {
                let bool_val = BoolValue { name: Some(name), value };
                let is_big_endian = false;

                let mut msg_buff: ArrayVec<u8, 7> = ArrayVec::new();
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error big endian without name
            {
                let mut msg_buff: ArrayVec<u8, 4> = ArrayVec::new();

                let bool_val = BoolValue { name: None, value };
                let is_big_endian = true;
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }
            // Capacity error big endian without name
            {
                let mut msg_buff: ArrayVec<u8, 4> = ArrayVec::new();

                let bool_val = BoolValue { name: None, value };
                let is_big_endian = false;
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(bool_val.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }


        }
    }
}
