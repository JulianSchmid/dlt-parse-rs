use arrayvec::{ArrayVec, CapacityError};

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructValue<'a> {
    pub number_of_entries: u16,
    pub name: Option<&'a str>,
    pub(crate) data: &'a [u8],
}

impl<'a> StructValue<'a> {
    /// Adds the verbose value to the given dlt mesage buffer.
    pub fn add_to_msg<const CAP: usize>(
        &self,
        buf: &mut ArrayVec<u8, CAP>,
        is_big_endian: bool,
    ) -> Result<(), CapacityError> {
        if let Some(name) = self.name {
            let type_info = [0b0000_0000, 0b0100_1000, 0b0000_0000, 0b0000_0000];
            let (number_of_entries, name_len) = match is_big_endian {
                true => (
                    self.number_of_entries.to_be_bytes(),
                    (name.len() as u16 + 1).to_be_bytes(),
                ),
                false => (
                    self.number_of_entries.to_le_bytes(),
                    (name.len() as u16 + 1).to_le_bytes(),
                ),
            };
            buf.try_extend_from_slice(&type_info)?;
            buf.try_extend_from_slice(&number_of_entries)?;
            buf.try_extend_from_slice(&[name_len[0], name_len[1]])?;
            buf.try_extend_from_slice(name.as_bytes())?;
            if buf.remaining_capacity() > 0 {
                // Safe as capacity is checked earlier
                unsafe { buf.push_unchecked(0) };
            } else {
                return Err(CapacityError::new(()));
            }
        } else {
            let type_info = [0b0000_0000, 0b0100_0000, 0b0000_0000, 0b0000_0000];
            let number_of_entries = match is_big_endian {
                true => self.number_of_entries.to_be_bytes(),
                false => self.number_of_entries.to_le_bytes(),
            };
            buf.try_extend_from_slice(&type_info)?;
            buf.try_extend_from_slice(&[number_of_entries[0], number_of_entries[1]])?;
        }

        buf.try_extend_from_slice(self.data)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::verbose::VerboseValue::*;
    use crate::verbose::*;
    use alloc::vec::Vec;
    use proptest::prelude::*;
    use std::format;

    proptest! {
            #[test]
            fn write_read(ref data_str in "\\pc{0,80}", ref name in "\\pc{0,20}") {
                const STRUCT_INIT_LEN_WITHOUT_NAME: usize = 6;
                const STRUCT_INIT_LEN_WITH_NAME: usize = STRUCT_INIT_LEN_WITHOUT_NAME + 3;

                let len_name_be = (name.len() as u16 + 1).to_be_bytes();
                let len_name_le = (name.len() as u16 + 1).to_le_bytes();
                // test big endian with name and fields of type i8, i16, i32
                {
                    const MAX_CONTENT_LEN: usize = 182;
                    const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN;
                    let is_big_endian = true;
                    let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

                    let number_of_entries: u16 = 3;
                    let number_of_entries_be = number_of_entries.to_be_bytes();

                    let first_entry = I8Value { variable_info: None, scaling: None, value: 1 };
                    first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let second_entry = I16Value { variable_info: None, scaling: None, value: 2 };
                    second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let third_entry = I32Value { variable_info: None, scaling: None, value: 3 };
                    third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let struct_value = StructValue { number_of_entries, name: Some(&name[..]), data: &deparsed_stuff[..] };
                    let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                    let mut content_msg = Vec::new();


                    content_msg.extend_from_slice(&[0b0000_0000, 0b0100_1000, 0b0000_0000, 0b0000_0000, number_of_entries_be[0], number_of_entries_be[1], len_name_be[0], len_name_be[1]]);
                    content_msg.extend_from_slice(name.as_bytes());
                    content_msg.push(0);
                    content_msg.extend_from_slice(&deparsed_stuff);
                    struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

                    prop_assert_eq!(&deparsed_struct[..], &content_msg[..]);

                    // Now wrap back
                    let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);

                    prop_assert_eq!(&parsed_back, &Ok((Struct(struct_value),&[] as &[u8])));

                    let parsed_struct = match parsed_back.unwrap().0 {
                        Struct(x) => x,
                        _ => return Err(TestCaseError::Fail("Expected struct on parsing".into())),
                    };

                        let (first_read_entry, rest) = VerboseValue::from_slice(parsed_struct.data, is_big_endian).unwrap();
                        let entry = match first_read_entry {
                            I8(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, first_entry);

                        let (second_read_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match second_read_entry {
                            I16(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, second_entry);

                        let (third_read_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match third_read_entry {
                            I32(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I32 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, third_entry);



                }

                // test little endian with name and fields of type bool
                {
                    const MAX_CONTENT_LEN: usize = 182;
                    const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN;
                    let is_big_endian = false;
                    let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

                    let number_of_entries: u16 = 3;
                    let number_of_entries_le = number_of_entries.to_le_bytes();

                    let first_entry = I8Value { variable_info: None, scaling: None, value: 1 };
                    first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let second_entry = I16Value { variable_info: None, scaling: None, value: 2 };
                    second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let third_entry = I32Value { variable_info: None, scaling: None, value: 3 };
                    third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let struct_value = StructValue { number_of_entries, name: Some(&name[..]), data: &deparsed_stuff[..] };
                    let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                    let mut content_msg = Vec::new();


                    content_msg.extend_from_slice(&[0b0000_0000, 0b0100_1000, 0b0000_0000, 0b0000_0000, number_of_entries_le[0], number_of_entries_le[1], len_name_le[0], len_name_le[1]]);
                    content_msg.extend_from_slice(name.as_bytes());
                    content_msg.push(0);
                    content_msg.extend_from_slice(&deparsed_stuff);
                    struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

                    prop_assert_eq!(&deparsed_struct[..], &content_msg[..]);

                    // Now wrap back
                    let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);

                    prop_assert_eq!(&parsed_back, &Ok((Struct(struct_value),&[] as &[u8])));

                    let parsed_struct = match parsed_back.unwrap().0 {
                        Struct(x) => x,
                        _ => return Err(TestCaseError::Fail("Expected struct on parsing".into())),
                    };

                        let (first_read_entry, rest) = VerboseValue::from_slice(parsed_struct.data, is_big_endian).unwrap();
                        let entry = match first_read_entry {
                            I8(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, first_entry);

                        let (second_read_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match second_read_entry {
                            I16(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, second_entry);

                        let (third_read_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match third_read_entry {
                            I32(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I32 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, third_entry);
                }

                // test big endian without name and fields of type i8, i16, i32
                {
                    const MAX_CONTENT_LEN: usize = 182;
                    const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN;
                    let is_big_endian = true;
                    let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

                    let number_of_entries: u16 = 3;
                    let number_of_entries_be = number_of_entries.to_be_bytes();

                    let first_entry = I8Value { variable_info: None, scaling: None, value: 1 };
                    first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let second_entry = I16Value { variable_info: None, scaling: None, value: 2 };
                    second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let third_entry = I32Value { variable_info: None, scaling: None, value: 3 };
                    third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let struct_value = StructValue { number_of_entries, name: None, data: &deparsed_stuff[..] };
                    let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                    let mut content_msg = Vec::new();


                    content_msg.extend_from_slice(&[0b0000_0000, 0b0100_0000, 0b0000_0000, 0b0000_0000, number_of_entries_be[0], number_of_entries_be[1]]);
                    content_msg.extend_from_slice(&deparsed_stuff);
                    struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

                    prop_assert_eq!(&deparsed_struct[..], &content_msg[..]);

                    // Now wrap back
                    let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);

                    prop_assert_eq!(&parsed_back, &Ok((Struct(struct_value),&[] as &[u8])));

                    let parsed_struct = match parsed_back.unwrap().0 {
                        Struct(x) => x,
                        _ => return Err(TestCaseError::Fail("Expected struct on parsing".into())),
                    };

                        let (first_read_entry, rest) = VerboseValue::from_slice(parsed_struct.data, is_big_endian).unwrap();
                        let entry = match first_read_entry {
                            I8(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, first_entry);

                        let (second_read_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match second_read_entry {
                            I16(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, second_entry);

                        let (third_read_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match third_read_entry {
                            I32(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I32 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, third_entry);



                }

                 // test big endian without name, but varnames and fields of type i8, i16, i32
                 {
                    const MAX_CONTENT_LEN: usize = 182;
                    const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN;
                    let name_1 = "Abc";
                    let name_2 = "Epsilon";
                    let name_3 = "Delta";

                    let unit_1 = "Einheit";
                    let unit_2 = "Fließ";
                    let unit_3 = "komma";

                    let var_info_1 = VariableInfoUnit { name: name_1, unit: unit_1};
                    let var_info_2 = VariableInfoUnit { name: name_2, unit: unit_2};
                    let var_info_3 = VariableInfoUnit { name: name_3, unit: unit_3};

                    let scaling_1 = Scaling { quantization: 1., offset: 1 };
                    let is_big_endian = true;
                    let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

                    let number_of_entries: u16 = 3;
                    let number_of_entries_be = number_of_entries.to_be_bytes();

                    let first_entry = I8Value { variable_info: Some(var_info_1), scaling: Some(scaling_1), value: 1 };
                    first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let second_entry = I16Value { variable_info: Some(var_info_2), scaling: None, value: 2 };
                    second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let third_entry = I32Value { variable_info: Some(var_info_3), scaling: None, value: 3 };
                    third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let struct_value = StructValue { number_of_entries, name: None, data: &deparsed_stuff[..] };
                    let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                    let mut content_msg = Vec::new();


                    content_msg.extend_from_slice(&[0b0000_0000, 0b0100_0000, 0b0000_0000, 0b0000_0000, number_of_entries_be[0], number_of_entries_be[1]]);
                    content_msg.extend_from_slice(&deparsed_stuff);
                    struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

                    prop_assert_eq!(&deparsed_struct[..], &content_msg[..]);

                    // Now wrap back
                    let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);

                    prop_assert_eq!(&parsed_back, &Ok((Struct(struct_value),&[] as &[u8])));

                    let parsed_struct = match parsed_back.unwrap().0 {
                        Struct(x) => x,
                        _ => return Err(TestCaseError::Fail("Expected struct on parsing".into())),
                    };

                        let (first_read_entry, rest) = VerboseValue::from_slice(parsed_struct.data, is_big_endian).unwrap();
                        let entry = match first_read_entry {
                            I8(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, first_entry);

                        let (second_read_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match second_read_entry {
                            I16(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, second_entry);

                        let (third_read_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match third_read_entry {
                            I32(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I32 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, third_entry);
                }

                // test little endian without name, but varnames and fields of type i8, i16, i32
                {
                    const MAX_CONTENT_LEN: usize = 182;
                    const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN;
                    let name_1 = "Abc";
                    let name_2 = "Name";
                    let name_3 = "123";

                    let unit_1 = "XAI";
                    let unit_2 = "XYZ";
                    let unit_3 = "MÜ";

                    let var_info_1 = VariableInfoUnit { name: name_1, unit: unit_1};
                    let var_info_2 = VariableInfoUnit { name: name_2, unit: unit_2};
                    let var_info_3 = VariableInfoUnit { name: name_3, unit: unit_3};

                    let scaling_1 = Scaling { quantization: 1., offset: 1 };
                    let is_big_endian = false;
                    let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

                    let number_of_entries: u16 = 3;
                    let number_of_entries_be = number_of_entries.to_le_bytes();

                    let first_entry = I8Value { variable_info: Some(var_info_1), scaling: Some(scaling_1), value: 1 };
                    first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let second_entry = I16Value { variable_info: Some(var_info_2), scaling: None, value: 2 };
                    second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let third_entry = I32Value { variable_info: Some(var_info_3), scaling: None, value: 3 };
                    third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let struct_value = StructValue { number_of_entries, name: None, data: &deparsed_stuff[..] };
                    let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                    let mut content_msg = Vec::new();


                    content_msg.extend_from_slice(&[0b0000_0000, 0b0100_0000, 0b0000_0000, 0b0000_0000, number_of_entries_be[0], number_of_entries_be[1]]);
                    content_msg.extend_from_slice(&deparsed_stuff);
                    struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

                    prop_assert_eq!(&deparsed_struct[..], &content_msg[..]);

                    // Now wrap back
                    let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);

                    prop_assert_eq!(&parsed_back, &Ok((Struct(struct_value),&[] as &[u8])));

                    let parsed_struct = match parsed_back.unwrap().0 {
                        Struct(x) => x,
                        _ => return Err(TestCaseError::Fail("Expected struct on parsing".into())),
                    };

                        let (first_read_entry, rest) = VerboseValue::from_slice(parsed_struct.data, is_big_endian).unwrap();
                        let entry = match first_read_entry {
                            I8(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, first_entry);

                        let (second_read_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match second_read_entry {
                            I16(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, second_entry);

                        let (third_read_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match third_read_entry {
                            I32(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I32 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, third_entry);
                }

    // test little endian without name, but varnames and fields of type i8, i16, i32
                {
                    const MAX_CONTENT_LEN: usize = 182;
                    const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN;
                    let name_1 = "Abc";
                    let name_2 = "Name";
                    let name_3 = "123";

                    let unit_1 = "XAI";
                    let unit_2 = "XYZ";
                    let unit_3 = "MÜ";

                    let var_info_1 = VariableInfoUnit { name: name_1, unit: unit_1};
                    let var_info_2 = VariableInfoUnit { name: name_2, unit: unit_2};
                    let var_info_3 = VariableInfoUnit { name: name_3, unit: unit_3};

                    let scaling_1 = Scaling { quantization: 1., offset: 1 };
                    //let mut msg_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                    // let slice_len = name.len() + data.len() + BYTES_NEEDED_WITH_NAME;
                    let is_big_endian = false;
                    let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

                    let number_of_entries: u16 = 3;
                    let number_of_entries_be = number_of_entries.to_le_bytes();

                    let first_entry = I8Value { variable_info: Some(var_info_1), scaling: Some(scaling_1), value: 1 };
                    first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let second_entry = I16Value { variable_info: Some(var_info_2), scaling: None, value: 2 };
                    second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let third_entry = I32Value { variable_info: Some(var_info_3), scaling: None, value: 3 };
                    third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                    let struct_value = StructValue { number_of_entries, name: None, data: &deparsed_stuff[..] };
                    // let raw_value = StructValue {name: Some(name), data, number_of_entries};
                    let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                    let mut content_msg = Vec::new();


                    content_msg.extend_from_slice(&[0b0000_0000, 0b0100_0000, 0b0000_0000, 0b0000_0000, number_of_entries_be[0], number_of_entries_be[1]]);
                    content_msg.extend_from_slice(&deparsed_stuff);
                    struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

                    prop_assert_eq!(&deparsed_struct[..], &content_msg[..]);

                    // Now wrap back
                    let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);

                    prop_assert_eq!(&parsed_back, &Ok((Struct(struct_value),&[] as &[u8])));

                    let parsed_struct = match parsed_back.unwrap().0 {
                        Struct(x) => x,
                        _ => return Err(TestCaseError::Fail("Expected struct on parsing".into())),
                    };

                        let (first_read_entry, rest) = VerboseValue::from_slice(parsed_struct.data, is_big_endian).unwrap();
                        let entry = match first_read_entry {
                            I8(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, first_entry);

                        let (second_read_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match second_read_entry {
                            I16(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, second_entry);

                        let (third_read_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                        let entry = match third_read_entry {
                            I32(x) => x,
                            _ => return Err(TestCaseError::Fail("Expected I32 value on parsing struct".into())),
                        };
                        prop_assert_eq!(entry, third_entry);
                }
            // test little endian without name, but varnames and fields of type i8, i16, i32
            {
                const MAX_CONTENT_LEN: usize = 182;
                const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN;
                let name_1 = "Abc";
                let name_2 = "Name";
                let name_3 = "123";

                let unit_1 = "XAI";
                let unit_2 = "XYZ";
                let unit_3 = "MÜ";

                let var_info_1 = VariableInfoUnit { name: name_1, unit: unit_1};
                let var_info_2 = VariableInfoUnit { name: name_2, unit: unit_2};
                let var_info_3 = VariableInfoUnit { name: name_3, unit: unit_3};

                let scaling_1 = Scaling { quantization: 1., offset: 1 };
                let is_big_endian = false;
                let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

                let number_of_entries: u16 = 3;
                let number_of_entries_be = number_of_entries.to_le_bytes();

                let first_entry = I8Value { variable_info: Some(var_info_1), scaling: Some(scaling_1), value: 1 };
                first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                let second_entry = I16Value { variable_info: Some(var_info_2), scaling: None, value: 2 };
                second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                let third_entry = I32Value { variable_info: Some(var_info_3), scaling: None, value: 3 };
                third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                let struct_value = StructValue { number_of_entries, name: None, data: &deparsed_stuff[..] };
                let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let mut content_msg = Vec::new();


                content_msg.extend_from_slice(&[0b0000_0000, 0b0100_0000, 0b0000_0000, 0b0000_0000, number_of_entries_be[0], number_of_entries_be[1]]);
                content_msg.extend_from_slice(&deparsed_stuff);
                struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

                prop_assert_eq!(&deparsed_struct[..], &content_msg[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);

                prop_assert_eq!(&parsed_back, &Ok((Struct(struct_value),&[] as &[u8])));

                let parsed_struct = match parsed_back.unwrap().0 {
                    Struct(x) => x,
                    _ => return Err(TestCaseError::Fail("Expected struct on parsing".into())),
                };

                    let (first_read_entry, rest) = VerboseValue::from_slice(parsed_struct.data, is_big_endian).unwrap();
                    let entry = match first_read_entry {
                        I8(x) => x,
                        _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                    };
                    prop_assert_eq!(entry, first_entry);

                    let (second_read_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                    let entry = match second_read_entry {
                        I16(x) => x,
                        _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                    };
                    prop_assert_eq!(entry, second_entry);

                    let (third_read_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                    let entry = match third_read_entry {
                        I32(x) => x,
                        _ => return Err(TestCaseError::Fail("Expected I32 value on parsing struct".into())),
                    };
                    prop_assert_eq!(entry, third_entry);
            }

                        // test big endian with name and fields of type struct
                        {
                            const MAX_CONTENT_LEN: usize = 182;
                            const BUFFER_SIZE: usize = (STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN) * 4;
                            let is_big_endian = true;
                            let mut deparsed_entries: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                            let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

                            let number_of_entries: u16 = 3;
                            let number_of_entries_be = number_of_entries.to_be_bytes();

                            let entry_one_struct =  I8Value { variable_info: None, scaling: None, value: 1 };
                            entry_one_struct.add_to_msg(&mut deparsed_entries, is_big_endian).unwrap();
                            let deparsed_entries_1 = deparsed_entries.clone();

                            let first_entry = StructValue { number_of_entries: 1, name: None, data: &deparsed_entries_1 };
                            first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                            let entry_two_struct = I16Value { variable_info: None, scaling: None, value: 2 };
                            entry_two_struct.add_to_msg(&mut deparsed_entries, is_big_endian).unwrap();
                            let deparsed_entries_2 = deparsed_entries.clone();

                            let second_entry = StructValue { number_of_entries: 2, name: None, data: &deparsed_entries_2 };
                            second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                            let entry_three_struct = I32Value { variable_info: None, scaling: None, value: 3 };
                            entry_three_struct.add_to_msg(&mut deparsed_entries, is_big_endian).unwrap();

                            let third_entry = StructValue { number_of_entries: 3, name: None, data: &deparsed_entries };
                            third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                            let struct_value = StructValue { number_of_entries: 3, name: Some(&name[..]), data: &deparsed_stuff[..] };
                            let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                            let mut content_msg = Vec::new();


                            content_msg.extend_from_slice(&[0b0000_0000, 0b0100_1000, 0b0000_0000, 0b0000_0000, number_of_entries_be[0], number_of_entries_be[1], len_name_be[0], len_name_be[1]]);
                            content_msg.extend_from_slice(name.as_bytes());
                            content_msg.push(0);
                            content_msg.extend_from_slice(&deparsed_stuff);
                            struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

                            prop_assert_eq!(&deparsed_struct[..], &content_msg[..]);

                            // Now wrap back
                            let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);

                            prop_assert_eq!(&parsed_back, &Ok((Struct(struct_value),&[] as &[u8])));

                            let parsed_struct = match parsed_back.unwrap().0 {
                                Struct(x) => x,
                                _ => return Err(TestCaseError::Fail("Expected struct on parsing".into())),
                            };

                                let (first_read_entry, rest) = VerboseValue::from_slice(parsed_struct.data, is_big_endian).unwrap();
                                let entry = match first_read_entry {
                                    Struct(x) => {
                                        let (first_entry, _) = VerboseValue::from_slice(x.data, is_big_endian).unwrap();
                                        match first_entry {
                                            I8(x) => x,
                                            _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                                        }
                                    },
                                    _ => return Err(TestCaseError::Fail("Expected struct value on parsing struct".into())),
                                };
                                prop_assert_eq!(entry, entry_one_struct);

                                let (second_read_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                                let entry = match second_read_entry {
                                    Struct(x) => {
                                        let (first_entry, rest) = VerboseValue::from_slice(x.data, is_big_endian).unwrap();
                                        match first_entry {
                                            I8(_) => (),
                                            _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                                        };
                                        let (second_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                                        match second_entry {
                                            I16(x) => x,
                                            _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                                        }
                                    },
                                    _ => return Err(TestCaseError::Fail("Expected struct value on parsing struct".into())),
                                };
                                prop_assert_eq!(entry, entry_two_struct);

                                let (third_read_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                                let entry = match third_read_entry {
                                    Struct(x) => {
                                        let (first_entry, rest) = VerboseValue::from_slice(x.data, is_big_endian).unwrap();
                                        match first_entry {
                                            I8(_) => (),
                                            _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                                        };
                                        let (second_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                                        match second_entry {
                                            I16(x) => x,
                                            _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                                        };
                                        let (third_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                                        match third_entry {
                                            I32(x) => x,
                                            _ => return Err(TestCaseError::Fail("Expected I32 value on parsing struct".into())),
                                        }
                                    },
                                    _ => return Err(TestCaseError::Fail("Expected struct value on parsing struct".into())),
                                };
                                prop_assert_eq!(entry, entry_three_struct);

                        }


             // test little endian with name and fields of type struct
             {
                const MAX_CONTENT_LEN: usize = 182;
                const BUFFER_SIZE: usize = (STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN) * 4;
                let is_big_endian = false;
                let mut deparsed_entries: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

                let number_of_entries: u16 = 3;
                let number_of_entries_le = number_of_entries.to_le_bytes();

                let entry_one_struct =  I8Value { variable_info: None, scaling: None, value: 1 };
                entry_one_struct.add_to_msg(&mut deparsed_entries, is_big_endian).unwrap();
                let deparsed_entries_1 = deparsed_entries.clone();

                let first_entry = StructValue { number_of_entries: 1, name: None, data: &deparsed_entries_1 };
                first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                let entry_two_struct = I16Value { variable_info: None, scaling: None, value: 2 };
                entry_two_struct.add_to_msg(&mut deparsed_entries, is_big_endian).unwrap();
                let deparsed_entries_2 = deparsed_entries.clone();

                let second_entry = StructValue { number_of_entries: 2, name: None, data: &deparsed_entries_2 };
                second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                let entry_three_struct = I32Value { variable_info: None, scaling: None, value: 3 };
                entry_three_struct.add_to_msg(&mut deparsed_entries, is_big_endian).unwrap();

                let third_entry = StructValue { number_of_entries: 3, name: None, data: &deparsed_entries };
                third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

                let struct_value = StructValue { number_of_entries: 3, name: Some(&name[..]), data: &deparsed_stuff[..] };
                let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
                let mut content_msg = Vec::new();


                content_msg.extend_from_slice(&[0b0000_0000, 0b0100_1000, 0b0000_0000, 0b0000_0000, number_of_entries_le[0], number_of_entries_le[1], len_name_le[0], len_name_le[1]]);
                content_msg.extend_from_slice(name.as_bytes());
                content_msg.push(0);
                content_msg.extend_from_slice(&deparsed_stuff);
                struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

                prop_assert_eq!(&deparsed_struct[..], &content_msg[..]);

                // Now wrap back
                let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);

                prop_assert_eq!(&parsed_back, &Ok((Struct(struct_value),&[] as &[u8])));

                let parsed_struct = match parsed_back.unwrap().0 {
                    Struct(x) => x,
                    _ => return Err(TestCaseError::Fail("Expected struct on parsing".into())),
                };

                    let (first_read_entry, rest) = VerboseValue::from_slice(parsed_struct.data, is_big_endian).unwrap();
                    let entry = match first_read_entry {
                        Struct(x) => {
                            let (first_entry, _) = VerboseValue::from_slice(x.data, is_big_endian).unwrap();
                            match first_entry {
                                I8(x) => x,
                                _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                            }
                        },
                        _ => return Err(TestCaseError::Fail("Expected struct value on parsing struct".into())),
                    };
                    prop_assert_eq!(entry, entry_one_struct);

                    let (second_read_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                    let entry = match second_read_entry {
                        Struct(x) => {
                            let (first_entry, rest) = VerboseValue::from_slice(x.data, is_big_endian).unwrap();
                            match first_entry {
                                I8(_) => (),
                                _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                            };
                            let (second_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                            match second_entry {
                                I16(x) => x,
                                _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                            }
                        },
                        _ => return Err(TestCaseError::Fail("Expected struct value on parsing struct".into())),
                    };
                    prop_assert_eq!(entry, entry_two_struct);

                    let (third_read_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                    let entry = match third_read_entry {
                        Struct(x) => {
                            let (first_entry, rest) = VerboseValue::from_slice(x.data, is_big_endian).unwrap();
                            match first_entry {
                                I8(_) => (),
                                _ => return Err(TestCaseError::Fail("Expected I8 value on parsing struct".into())),
                            };
                            let (second_entry, rest) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                            match second_entry {
                                I16(x) => x,
                                _ => return Err(TestCaseError::Fail("Expected I16 value on parsing struct".into())),
                            };
                            let (third_entry, _) = VerboseValue::from_slice(rest, is_big_endian).unwrap();
                            match third_entry {
                                I32(x) => x,
                                _ => return Err(TestCaseError::Fail("Expected I32 value on parsing struct".into())),
                            }
                        },
                        _ => return Err(TestCaseError::Fail("Expected struct value on parsing struct".into())),
                    };
                    prop_assert_eq!(entry, entry_three_struct);

            }


        // test big endian failing parsing of garbage msg
        {
            const MAX_CONTENT_LEN: usize = 182;
            const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN;

            let is_big_endian = true;
            let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

            let number_of_entries: u16 = 3;
            deparsed_stuff.try_extend_from_slice(data_str.as_bytes()).unwrap();

            if data_str.len() > 0 {
                deparsed_stuff[0] = 0b1111_1111;  // Make sure that type info is invalid
            }

            let struct_value = StructValue { number_of_entries, name: None, data: &deparsed_stuff[..] };

            let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
            struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

            // Now wrap back
            let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);
            match parsed_back {
                Err(_) => (),
                _ => return Err(TestCaseError::Fail("Expected Error on deparsing".into())),
            }
        }

        // test little endian failing parsing of garbage msg
        {
            const MAX_CONTENT_LEN: usize = 182;
            const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 3 * MAX_CONTENT_LEN;

            let is_big_endian = false;
            let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

            let number_of_entries: u16 = 3;
            deparsed_stuff.try_extend_from_slice(data_str.as_bytes()).unwrap();

            if data_str.len() > 0 {
                deparsed_stuff[0] = 0b1111_1111;  // Make sure that type info is invalid
            }

            let struct_value = StructValue { number_of_entries, name: None, data: &deparsed_stuff[..] };

            let mut deparsed_struct: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
            struct_value.add_to_msg(&mut deparsed_struct, is_big_endian).unwrap();

            // Now wrap back
            let parsed_back = VerboseValue::from_slice(&deparsed_struct, is_big_endian);
            match parsed_back {
                Err(_) => (),
                _ => return Err(TestCaseError::Fail("Expected Error on deparsing".into())),
            }
        }



        // test capacity error big endian
        {
            const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 15;

            let is_big_endian = true;
            let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

            let number_of_entries: u16 = 3;

            let first_entry = I8Value { variable_info: None, scaling: None, value: 1 };
            first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

            let second_entry = I16Value { variable_info: None, scaling: None, value: 2 };
            second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

            let third_entry = I32Value { variable_info: None, scaling: None, value: 3 };
            third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

            let struct_value = StructValue { number_of_entries, name: None, data: &deparsed_stuff[..] };

            let mut zero_buff: ArrayVec<u8, 0> = ArrayVec::new();
            let err = struct_value.add_to_msg(&mut zero_buff, is_big_endian);
            prop_assert_eq!(err, Err(CapacityError::new(())));

            let mut off_by_one_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
            let err = struct_value.add_to_msg(&mut off_by_one_buff, is_big_endian);
            prop_assert_eq!(err, Err(CapacityError::new(())));


        }

        // test capacity error little endian
        {
            const BUFFER_SIZE: usize = STRUCT_INIT_LEN_WITH_NAME + 15;

            let is_big_endian = false;
            let mut deparsed_stuff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();

            let number_of_entries: u16 = 3;

            let first_entry = I8Value { variable_info: None, scaling: None, value: 1 };
            first_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

            let second_entry = I16Value { variable_info: None, scaling: None, value: 2 };
            second_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

            let third_entry = I32Value { variable_info: None, scaling: None, value: 3 };
            third_entry.add_to_msg(&mut deparsed_stuff, is_big_endian).unwrap();

            let struct_value = StructValue { number_of_entries, name: None, data: &deparsed_stuff[..] };

            let mut zero_buff: ArrayVec<u8, 0> = ArrayVec::new();
            let err = struct_value.add_to_msg(&mut zero_buff, is_big_endian);
            prop_assert_eq!(err, Err(CapacityError::new(())));

            let mut off_by_one_buff: ArrayVec<u8, BUFFER_SIZE> = ArrayVec::new();
            let err = struct_value.add_to_msg(&mut off_by_one_buff, is_big_endian);
            prop_assert_eq!(err, Err(CapacityError::new(())));


        }


            }
        }
}
