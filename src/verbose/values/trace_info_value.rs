use arrayvec::{ArrayVec, CapacityError};

#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TraceInfoValue<'a> {
    pub value: &'a str,
}

impl<'a> TraceInfoValue<'a> {
    /// Adds the verbose value to the given dlt mesage buffer.
    pub fn add_to_msg<const CAP: usize>(
        &self,
        buf: &mut ArrayVec<u8, CAP>,
        is_big_endian: bool,
    ) -> Result<(), CapacityError> {
        let type_info = [0b0000_0000, 0b0010_0000, 0b0000_0000, 0b0000_0000];
        let value_len = match is_big_endian {
            true => (self.value.len() as u16 + 1).to_be_bytes(),
            false => (self.value.len() as u16 + 1).to_le_bytes(),
        };
        buf.try_extend_from_slice(&type_info)?;
        buf.try_extend_from_slice(&[value_len[0], value_len[1]])?;

        buf.try_extend_from_slice(self.value.as_bytes())?;
        if buf.remaining_capacity() > 0 {
            // Safe as capacity is checked earlier
            unsafe { buf.push_unchecked(0) };
        } else {
            return Err(CapacityError::new(()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::verbose::VerboseValue;
    use crate::verbose::VerboseValue::TraceInfo;
    use alloc::vec::Vec;
    use proptest::prelude::*;
    use std::format;

    proptest! {
        #[test]
        fn write_read(ref value in "\\pc{0,80}") {
            const MAX_SYMBOL_LENGTH_VALUE: usize = 80;
            const BYTES_NEEDED: usize = 7;

            const BUFFER_SIZE: usize = MAX_SYMBOL_LENGTH_VALUE * 4 + BYTES_NEEDED;


            // test big endian
            {

                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let slice_len = value.len() + BYTES_NEEDED;
                let is_big_endian = true;

                let trace_value = TraceInfoValue {value};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_value_be = (value.len() as u16 + 1).to_be_bytes();

                prop_assert_eq!(trace_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0000_0000, 0b0010_0000, 0b0000_0000, 0b0000_0000, len_value_be[0], len_value_be[1]]);
                content_buff.extend_from_slice(&value.as_bytes());
                content_buff.push(0);
                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((TraceInfo (trace_value),&[] as &[u8])));
            }

            // test little endian
            {

                let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let slice_len = value.len() + BYTES_NEEDED;
                let is_big_endian = false;

                let trace_value = TraceInfoValue {value};
                let mut content_buff = Vec::with_capacity(slice_len);

                let len_value_le = (value.len() as u16 + 1).to_le_bytes();

                prop_assert_eq!(trace_value.add_to_msg(&mut msg_buff, is_big_endian), Ok(()));

                content_buff.extend_from_slice(&[0b0000_0000, 0b0010_0000, 0b0000_0000, 0b0000_0000, len_value_le[0], len_value_le[1]]);
                content_buff.extend_from_slice(&value.as_bytes());
                content_buff.push(0);
                prop_assert_eq!(&msg_buff[..slice_len], &content_buff[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&msg_buff, is_big_endian);
                prop_assert_eq!(parsed_back, Ok((TraceInfo(trace_value),&[] as &[u8])));
            }


            // Capacity error big endian
            {
                const SLICE_LEN: usize = BYTES_NEEDED - 1;

                let trace_value = TraceInfoValue {value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(trace_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(trace_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }

            // Capacity error little endian
            {
                const SLICE_LEN: usize = BYTES_NEEDED - 1;

                let trace_value = TraceInfoValue {value};
                let is_big_endian = true;

                let mut msg_buff: ArrayVec<u8, SLICE_LEN> = ArrayVec::new();
                prop_assert_eq!(trace_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

                let mut msg_buff: ArrayVec<u8, 0> = ArrayVec::new();
                prop_assert_eq!(trace_value.add_to_msg(&mut msg_buff, is_big_endian), Err(CapacityError::new(())));

            }


        }
    }
}
