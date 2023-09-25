use crate::error::{Layer, UnexpectedEndOfSliceError, VerboseDecodeError};

use super::{ArrayDimensions, RawF128, RawF16, Scaling};

/// Helper for parsing verbose messages.
pub(crate) struct FieldSlicer<'a> {
    /// Unparsed part of the verbose message.
    rest: &'a [u8],

    /// Offset since the parsing has started.
    offset: usize,
}

impl<'a> FieldSlicer<'a> {
    #[inline]
    pub fn new(data: &[u8], offset: usize) -> FieldSlicer {
        FieldSlicer { rest: data, offset }
    }

    #[inline]
    pub fn rest(&self) -> &'a [u8] {
        self.rest
    }

    pub fn read_u8(&mut self) -> Result<u8, VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check length
        if self.rest.is_empty() {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseValue,
                minimum_size: self.offset + 1,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // SAFETY: Length of at least 1 verified in the previous if.
        let result = unsafe { *self.rest.get_unchecked(0) };

        // move slice
        // SAFETY: Length of at least 1 verified in the previous if.
        self.rest =
            unsafe { core::slice::from_raw_parts(self.rest.as_ptr().add(1), self.rest.len() - 1) };
        self.offset += 1;

        Ok(result)
    }

    pub fn read_i8(&mut self) -> Result<i8, VerboseDecodeError> {
        Ok(i8::from_ne_bytes([self.read_u8()?]))
    }

    pub fn read_2bytes(&mut self) -> Result<[u8; 2], VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check length
        if self.rest.len() < 2 {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseValue,
                minimum_size: self.offset + 2,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // read value
        // SAFETY: Length of at least 2 verified in the previous if.
        let result = unsafe { [*self.rest.get_unchecked(0), *self.rest.get_unchecked(1)] };

        // move slice
        // SAFETY: Length of at least 2 verified in the previous if.
        self.rest =
            unsafe { core::slice::from_raw_parts(self.rest.as_ptr().add(2), self.rest.len() - 2) };
        self.offset += 2;

        Ok(result)
    }

    pub fn read_4bytes(&mut self) -> Result<[u8; 4], VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check length
        if self.rest.len() < 4 {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseValue,
                minimum_size: self.offset + 4,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // read value
        // SAFETY: Length of at least 4 verified in the previous if.
        let result = unsafe {
            [
                *self.rest.get_unchecked(0),
                *self.rest.get_unchecked(1),
                *self.rest.get_unchecked(2),
                *self.rest.get_unchecked(3),
            ]
        };

        // move slice
        // SAFETY: Length of at least 4 verified in the previous if.
        self.rest =
            unsafe { core::slice::from_raw_parts(self.rest.as_ptr().add(4), self.rest.len() - 4) };
        self.offset += 4;

        Ok(result)
    }

    pub fn read_8bytes(&mut self) -> Result<[u8; 8], VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check length
        if self.rest.len() < 8 {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseValue,
                minimum_size: self.offset + 8,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // read value
        // SAFETY: Length of at least 8 verified in the previous if.
        let result = unsafe {
            [
                *self.rest.get_unchecked(0),
                *self.rest.get_unchecked(1),
                *self.rest.get_unchecked(2),
                *self.rest.get_unchecked(3),
                *self.rest.get_unchecked(4),
                *self.rest.get_unchecked(5),
                *self.rest.get_unchecked(6),
                *self.rest.get_unchecked(7),
            ]
        };

        // move slice
        // SAFETY: Length of at least 8 verified in the previous if.
        self.rest =
            unsafe { core::slice::from_raw_parts(self.rest.as_ptr().add(8), self.rest.len() - 8) };
        self.offset += 8;

        Ok(result)
    }

    pub fn read_16bytes(&mut self) -> Result<[u8; 16], VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check length
        if self.rest.len() < 16 {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseValue,
                minimum_size: self.offset + 16,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // read value
        // SAFETY: Length of at least 16 verified in the previous if.
        let result = unsafe {
            [
                *self.rest.get_unchecked(0),
                *self.rest.get_unchecked(1),
                *self.rest.get_unchecked(2),
                *self.rest.get_unchecked(3),
                *self.rest.get_unchecked(4),
                *self.rest.get_unchecked(5),
                *self.rest.get_unchecked(6),
                *self.rest.get_unchecked(7),
                *self.rest.get_unchecked(8),
                *self.rest.get_unchecked(9),
                *self.rest.get_unchecked(10),
                *self.rest.get_unchecked(11),
                *self.rest.get_unchecked(12),
                *self.rest.get_unchecked(13),
                *self.rest.get_unchecked(14),
                *self.rest.get_unchecked(15),
            ]
        };

        // move slice
        // SAFETY: Length of at least 8 verified in the previous if.
        self.rest = unsafe {
            core::slice::from_raw_parts(self.rest.as_ptr().add(16), self.rest.len() - 16)
        };
        self.offset += 16;

        Ok(result)
    }

    pub fn read_u16(&mut self, is_big_endian: bool) -> Result<u16, VerboseDecodeError> {
        self.read_2bytes().map(|bytes| {
            if is_big_endian {
                u16::from_be_bytes(bytes)
            } else {
                u16::from_le_bytes(bytes)
            }
        })
    }

    pub fn read_i16(&mut self, is_big_endian: bool) -> Result<i16, VerboseDecodeError> {
        self.read_2bytes().map(|bytes| {
            if is_big_endian {
                i16::from_be_bytes(bytes)
            } else {
                i16::from_le_bytes(bytes)
            }
        })
    }

    pub fn read_u32(&mut self, is_big_endian: bool) -> Result<u32, VerboseDecodeError> {
        self.read_4bytes().map(|bytes| {
            if is_big_endian {
                u32::from_be_bytes(bytes)
            } else {
                u32::from_le_bytes(bytes)
            }
        })
    }

    pub fn read_i32(&mut self, is_big_endian: bool) -> Result<i32, VerboseDecodeError> {
        self.read_4bytes().map(|bytes| {
            if is_big_endian {
                i32::from_be_bytes(bytes)
            } else {
                i32::from_le_bytes(bytes)
            }
        })
    }

    pub fn read_u64(&mut self, is_big_endian: bool) -> Result<u64, VerboseDecodeError> {
        self.read_8bytes().map(|bytes| {
            if is_big_endian {
                u64::from_be_bytes(bytes)
            } else {
                u64::from_le_bytes(bytes)
            }
        })
    }

    pub fn read_i64(&mut self, is_big_endian: bool) -> Result<i64, VerboseDecodeError> {
        self.read_8bytes().map(|bytes| {
            if is_big_endian {
                i64::from_be_bytes(bytes)
            } else {
                i64::from_le_bytes(bytes)
            }
        })
    }

    pub fn read_u128(&mut self, is_big_endian: bool) -> Result<u128, VerboseDecodeError> {
        self.read_16bytes().map(|bytes| {
            if is_big_endian {
                u128::from_be_bytes(bytes)
            } else {
                u128::from_le_bytes(bytes)
            }
        })
    }

    pub fn read_i128(&mut self, is_big_endian: bool) -> Result<i128, VerboseDecodeError> {
        self.read_16bytes().map(|bytes| {
            if is_big_endian {
                i128::from_be_bytes(bytes)
            } else {
                i128::from_le_bytes(bytes)
            }
        })
    }

    pub fn read_f16(&mut self, is_big_endian: bool) -> Result<RawF16, VerboseDecodeError> {
        self.read_u16(is_big_endian).map(|v| RawF16::from_bits(v))
    }

    pub fn read_f128(&mut self, is_big_endian: bool) -> Result<RawF128, VerboseDecodeError> {
        self.read_u128(is_big_endian).map(|v| RawF128::from_bits(v))
    }

    pub fn read_f32(&mut self, is_big_endian: bool) -> Result<f32, VerboseDecodeError> {
        self.read_4bytes().map(|bytes| {
            if is_big_endian {
                f32::from_be_bytes(bytes)
            } else {
                f32::from_le_bytes(bytes)
            }
        })
    }

    pub fn read_f64(&mut self, is_big_endian: bool) -> Result<f64, VerboseDecodeError> {
        self.read_8bytes().map(|bytes| {
            if is_big_endian {
                f64::from_be_bytes(bytes)
            } else {
                f64::from_le_bytes(bytes)
            }
        })
    }

    pub fn read_var_name(&mut self, is_big_endian: bool) -> Result<&'a str, VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check length
        if self.rest.len() < 2 {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseValue,
                minimum_size: self.offset + 2,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // read lengths
        let name_length = {
            // SAFETY: Length of at least 2 verified in the previous if.
            let bytes = unsafe { [*self.rest.get_unchecked(0), *self.rest.get_unchecked(1)] };
            if is_big_endian {
                u16::from_be_bytes(bytes) as usize
            } else {
                u16::from_le_bytes(bytes) as usize
            }
        };

        // check length of slice
        let total_size = 2 + name_length;
        if self.rest.len() < total_size {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseValue,
                minimum_size: self.offset + total_size,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // read name
        let name = if name_length > 0 {
            // SAFETY: Length of at least 2 + name_length verified in the previous if.
            //         Additionally name_length is guranteed to be at least 1.
            let name_raw = unsafe {
                core::slice::from_raw_parts(
                    self.rest.as_ptr().add(2),
                    // substract 1 to skip the zero termination
                    name_length - 1,
                )
            };
            // SAFETY: Length of at least 2 + name_length verified in the previous if.
            //         Additionally name_length is guranteed to be at least 1.
            let last = unsafe { *self.rest.as_ptr().add(2 + name_length - 1) };

            // check for zero termination
            if last != 0 {
                return Err(VariableNameStringMissingNullTermination);
            }

            core::str::from_utf8(name_raw)?
        } else {
            ""
        };

        // move slice
        // SAFETY: Length of at least total_size verfied in previous if.
        self.rest = unsafe {
            core::slice::from_raw_parts(
                self.rest.as_ptr().add(total_size),
                self.rest.len() - total_size,
            )
        };
        self.offset += total_size;

        Ok(name)
    }

    pub fn read_var_name_and_unit(
        &mut self,
        is_big_endian: bool,
    ) -> Result<(&'a str, &'a str), VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check length
        if self.rest.len() < 4 {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseValue,
                minimum_size: self.offset + 4,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // read lengths
        let name_length = {
            // SAFETY: Length of at least 4 verified in the previous if.
            let bytes = unsafe { [*self.rest.get_unchecked(0), *self.rest.get_unchecked(1)] };
            if is_big_endian {
                u16::from_be_bytes(bytes) as usize
            } else {
                u16::from_le_bytes(bytes) as usize
            }
        };
        let unit_length = {
            // SAFETY: Length of at least 4 verified in the previous if.
            let bytes = unsafe { [*self.rest.get_unchecked(2), *self.rest.get_unchecked(3)] };
            if is_big_endian {
                u16::from_be_bytes(bytes) as usize
            } else {
                u16::from_le_bytes(bytes) as usize
            }
        };

        // check length of slice
        let total_size = 4 + name_length + unit_length;
        if self.rest.len() < total_size {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseValue,
                minimum_size: self.offset + total_size,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // read name
        let name = if name_length > 0 {
            // SAFETY: Length of at least 4 + name_length verified in the previous if.
            //         Additionally name_length is guranteed to be at least 1.
            let name_raw = unsafe {
                core::slice::from_raw_parts(
                    self.rest.as_ptr().add(4),
                    // substract 1 to skip the zero termination
                    name_length - 1,
                )
            };
            // SAFETY: Length of at least 4 + name_length verified in the previous if.
            //         Additionally name_length is guranteed to be at least 1.
            let last = unsafe { *self.rest.as_ptr().add(4 + name_length - 1) };

            // check for zero termination
            if last != 0 {
                return Err(VariableNameStringMissingNullTermination);
            }

            core::str::from_utf8(name_raw)?
        } else {
            ""
        };

        // read unit
        let unit = if unit_length > 0 {
            // SAFETY: Length of at least 4 + name_length + unit_length verified in the previous if.
            //         Additionally unit_length is guranteed to be at least 1.
            let unit_raw = unsafe {
                core::slice::from_raw_parts(
                    self.rest.as_ptr().add(4 + name_length),
                    // substract 1 to skip the zero termination
                    unit_length - 1,
                )
            };
            // SAFETY: Length of at least 4 + name_length + unit_length verified in the previous if.
            //         Additionally unit_length is guranteed to be at least 1.
            let last = unsafe { *self.rest.as_ptr().add(4 + name_length + unit_length - 1) };

            // check for zero termination
            if last != 0 {
                return Err(VariableUnitStringMissingNullTermination);
            }

            core::str::from_utf8(unit_raw)?
        } else {
            ""
        };

        // move slice
        // SAFETY: Length of at least total_size verfied in previous if.
        self.rest = unsafe {
            core::slice::from_raw_parts(
                self.rest.as_ptr().add(total_size),
                self.rest.len() - total_size,
            )
        };
        self.offset += total_size;

        // done
        Ok((name, unit))
    }

    pub fn read_raw(&mut self, len: usize) -> Result<&'a [u8], VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check that the string length is present
        if self.rest.len() < len {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseValue,
                minimum_size: self.offset + len,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // SAFETY: Slice length checked above to be at least len
        let result = unsafe { core::slice::from_raw_parts(self.rest.as_ptr(), len) };

        // move rest & offset
        self.rest = unsafe {
            core::slice::from_raw_parts(self.rest.as_ptr().add(len), self.rest.len() - len)
        };
        self.offset += len;

        Ok(result)
    }

    const FIXED_POINT_FLAG_1: u8 = 0b0001_0000;

    pub fn read_i32_scaling(
        &mut self,
        is_big_endian: bool,
        type_info: [u8; 4],
    ) -> Result<Option<Scaling<i32>>, VerboseDecodeError> {
        if 0 != type_info[1] & Self::FIXED_POINT_FLAG_1 {
            Ok(Some(Scaling {
                quantization: self.read_f32(is_big_endian)?,
                offset: self.read_i32(is_big_endian)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn read_i64_scaling(
        &mut self,
        is_big_endian: bool,
        type_info: [u8; 4],
    ) -> Result<Option<Scaling<i64>>, VerboseDecodeError> {
        if 0 != type_info[1] & Self::FIXED_POINT_FLAG_1 {
            Ok(Some(Scaling {
                quantization: self.read_f32(is_big_endian)?,
                offset: self.read_i64(is_big_endian)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn read_i128_scaling(
        &mut self,
        is_big_endian: bool,
        type_info: [u8; 4],
    ) -> Result<Option<Scaling<i128>>, VerboseDecodeError> {
        if 0 != type_info[1] & Self::FIXED_POINT_FLAG_1 {
            Ok(Some(Scaling {
                quantization: self.read_f32(is_big_endian)?,
                offset: self.read_i128(is_big_endian)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn read_array_dimesions(
        &mut self,
        is_big_endian: bool,
    ) -> Result<ArrayDimensions<'a>, VerboseDecodeError> {
        use VerboseDecodeError::*;

        // first read the number of dimensions
        let num_dims = self.read_u16(is_big_endian)?;

        // check if enough data is present for the dimensions
        let len = usize::from(num_dims) * 2;
        if self.rest.len() < len {
            return Err(UnexpectedEndOfSlice(UnexpectedEndOfSliceError {
                layer: Layer::VerboseTypeInfo,
                minimum_size: self.offset + len,
                actual_size: self.offset + self.rest.len(),
            }));
        }

        // safe array dimensions slice
        let result = ArrayDimensions {
            is_big_endian,
            dimensions: unsafe { core::slice::from_raw_parts(self.rest.as_ptr(), len) },
        };

        // move rest and offset
        self.rest = unsafe {
            core::slice::from_raw_parts(self.rest.as_ptr().add(len), self.rest.len() - len)
        };
        self.offset += len;

        Ok(result)
    }
}

#[cfg(test)]
mod test_field_slicer {
    use super::*;
    use crate::error::{Layer, UnexpectedEndOfSliceError, VerboseDecodeError};
    use alloc::vec::Vec;
    use proptest::arbitrary::any;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use std::format;

    proptest! {
        #[test]
        fn new(
            data in prop::collection::vec(any::<u8>(), 0..10),
            offset in any::<usize>()
        ) {
            let s = FieldSlicer::new(
                &data,
                offset
            );
            prop_assert_eq!(s.rest(), &data);
            prop_assert_eq!(s.offset, offset);
        }
    }

    proptest! {
        #[test]
        fn read_2bytes(
            value in any::<[u8;2]>(),
            slice_len in 2usize..4,
            offset in 0usize..usize::MAX-1,
            bad_len in 0usize..2,
        ) {
            // ok
            {
                let data = [value[0], value[1], 1, 2];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_2bytes(),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 2);
                prop_assert_eq!(slicer.rest, &[1,2][..slice_len - 2]);
            }

            // length error
            {
                let mut slicer = FieldSlicer::new(&value[..bad_len], offset);
                prop_assert_eq!(
                    slicer.read_2bytes(),
                    Err(VerboseDecodeError::UnexpectedEndOfSlice(
                        UnexpectedEndOfSliceError{
                            layer: Layer::VerboseValue,
                            actual_size: offset + bad_len,
                            minimum_size: offset + 2,
                        }
                    ))
                );
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &value[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_4bytes(
            value in any::<[u8;4]>(),
            slice_len in 4usize..8,
            offset in 0usize..usize::MAX-3,
            bad_len in 0usize..4,
        ) {
            // ok
            {
                let data = [value[0], value[1], value[2], value[3], 1, 2, 3, 4];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_4bytes(),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 4);
                prop_assert_eq!(slicer.rest, &[1,2,3,4][..slice_len - 4]);
            }

            // length error
            {
                let mut slicer = FieldSlicer::new(&value[..bad_len], offset);
                prop_assert_eq!(
                    slicer.read_4bytes(),
                    Err(VerboseDecodeError::UnexpectedEndOfSlice(
                        UnexpectedEndOfSliceError{
                            layer: Layer::VerboseValue,
                            actual_size: offset + bad_len,
                            minimum_size: offset + 4,
                        }
                    ))
                );
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &value[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_8bytes(
            value in any::<[u8;8]>(),
            slice_len in 8usize..16,
            offset in 0usize..usize::MAX-7,
            bad_len in 0usize..8,
        ) {
            // ok
            {
                let data = [value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7], 1, 2, 3, 4, 5, 6, 7, 8];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_8bytes(),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 8);
                prop_assert_eq!(slicer.rest, &[1,2,3,4,5,6,7,8][..slice_len - 8]);
            }

            // length error
            {
                let mut slicer = FieldSlicer::new(&value[..bad_len], offset);
                prop_assert_eq!(
                    slicer.read_8bytes(),
                    Err(VerboseDecodeError::UnexpectedEndOfSlice(
                        UnexpectedEndOfSliceError{
                            layer: Layer::VerboseValue,
                            actual_size: offset + bad_len,
                            minimum_size: offset + 8,
                        }
                    ))
                );
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &value[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_16bytes(
            value in any::<[u8;16]>(),
            slice_len in 16usize..32,
            offset in 0usize..usize::MAX-15,
            bad_len in 0usize..16,
        ) {
            // ok
            {
                let data = [value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7], value[8], value[9], value[10], value[11], value[12], value[13], value[14], value[15], 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_16bytes(),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 16);
                prop_assert_eq!(slicer.rest, &[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16][..slice_len - 16]);
            }

            // length error
            {
                let mut slicer = FieldSlicer::new(&value[..bad_len], offset);
                prop_assert_eq!(
                    slicer.read_16bytes(),
                    Err(VerboseDecodeError::UnexpectedEndOfSlice(
                        UnexpectedEndOfSliceError{
                            layer: Layer::VerboseValue,
                            actual_size: offset + bad_len,
                            minimum_size: offset + 16,
                        }
                    ))
                );
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &value[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_u8(
            value in any::<u8>(),
            slice_len in 1usize..3,
            offset in 0usize..usize::MAX,
        ) {
            // ok
            {
                let data = [value, 123, 234];
                let mut slicer = FieldSlicer{
                    rest: &data[..slice_len],
                    offset,
                };
                prop_assert_eq!(
                    slicer.read_u8(),
                    Ok(value)
                );
                prop_assert_eq!(slicer.rest, &data[1..slice_len]);
                prop_assert_eq!(slicer.offset, offset + 1);
            }
            // length error
            {
                let mut slicer = FieldSlicer{
                    rest: &[],
                    offset,
                };
                prop_assert_eq!(
                    slicer.read_u8(),
                    Err(VerboseDecodeError::UnexpectedEndOfSlice(
                        UnexpectedEndOfSliceError{
                            layer: Layer::VerboseValue,
                            actual_size: offset,
                            minimum_size: offset + 1,
                        }
                    ))
                );
            }
        }
    }

    proptest! {
        #[test]
        fn read_i8(
            value in any::<i8>(),
            slice_len in 1usize..3,
            offset in 0usize..usize::MAX,
        ) {
            // ok
            {
                let data = [value as u8, i8::MIN as u8, i8::MAX as u8];
                let mut slicer = FieldSlicer{
                    rest: &data[..slice_len],
                    offset,
                };
                prop_assert_eq!(
                    slicer.read_i8(),
                    Ok(value)
                );
                prop_assert_eq!(slicer.rest, &data[1..slice_len]);
                prop_assert_eq!(slicer.offset, offset + 1);
            }
            // length error
            {
                let mut slicer = FieldSlicer{
                    rest: &[],
                    offset,
                };
                prop_assert_eq!(
                    slicer.read_i8(),
                    Err(VerboseDecodeError::UnexpectedEndOfSlice(
                        UnexpectedEndOfSliceError{
                            layer: Layer::VerboseValue,
                            actual_size: offset,
                            minimum_size: offset + 1,
                        }
                    ))
                );
            }
        }
    }

    proptest! {
        #[test]
        fn read_u16(
            value in any::<u16>(),
            slice_len in 2usize..4,
            offset in 0usize..usize::MAX-1,
            bad_len in 0usize..2
        ) {

            // ok big endian
            {
                let value_be = value.to_be_bytes();
                let data = [value_be[0], value_be[1], 1, 2,];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_u16(true),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 2);
                prop_assert_eq!(slicer.rest, &[1,2][..slice_len - 2]);
            }
            // ok little endian
            {
                let value_le = value.to_le_bytes();
                let data = [
                    value_le[0], value_le[1], 1, 2,
                ];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_u16(false),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 2);
                prop_assert_eq!(slicer.rest, &[1,2][..slice_len - 2]);
            }

            // length error
            {
                let expected = Err(VerboseDecodeError::UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + 2,
                    }
                ));
                let data = value.to_le_bytes();
                let mut slicer = FieldSlicer::new(&data[..bad_len], offset);

                // little endian
                prop_assert_eq!(slicer.read_u16(false), expected.clone());
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);

                // big endian
                prop_assert_eq!(slicer.read_u16(true), expected);
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_i16(
            value in any::<i16>(),
            slice_len in 2usize..4,
            offset in 0usize..usize::MAX-1,
            bad_len in 0usize..2
        ) {

            // ok big endian
            {
                let value_be = value.to_be_bytes();
                let data = [value_be[0], value_be[1], 1, 2];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_i16(true),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 2);
                prop_assert_eq!(slicer.rest, &[1,2][..slice_len - 2]);
            }
            // ok little endian
            {
                let value_le = value.to_le_bytes();
                let data = [
                    value_le[0], value_le[1], 1, 2,
                ];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_i16(false),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 2);
                prop_assert_eq!(slicer.rest, &[1,2][..slice_len - 2]);
            }

            // length error
            {
                let expected = Err(VerboseDecodeError::UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + 2,
                    }
                ));
                let data = value.to_le_bytes();
                let mut slicer = FieldSlicer::new(&data[..bad_len], offset);

                // little endian
                prop_assert_eq!(slicer.read_i16(false), expected.clone());
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);

                // big endian
                prop_assert_eq!(slicer.read_i16(true), expected);
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_u32(
            value in any::<u32>(),
            slice_len in 4usize..8,
            offset in 0usize..usize::MAX-3,
            bad_len in 0usize..4
        ) {

            // ok big endian
            {
                let value_be = value.to_be_bytes();
                let data = [value_be[0], value_be[1], value_be[2], value_be[3], 1, 2, 3, 4];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_u32(true),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 4);
                prop_assert_eq!(slicer.rest, &[1,2,3,4][..slice_len - 4]);
            }
            // ok little endian
            {
                let value_le = value.to_le_bytes();
                let data = [
                    value_le[0], value_le[1], value_le[2], value_le[3], 1, 2, 3, 4
                ];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_u32(false),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 4);
                prop_assert_eq!(slicer.rest, &[1,2,3,4][..slice_len - 4]);
            }

            // length error
            {
                let expected = Err(VerboseDecodeError::UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + 4,
                    }
                ));
                let data = value.to_le_bytes();
                let mut slicer = FieldSlicer::new(&data[..bad_len], offset);

                // little endian
                prop_assert_eq!(slicer.read_u32(false), expected.clone());
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);

                // big endian
                prop_assert_eq!(slicer.read_u32(true), expected);
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_i32(
            value in any::<i32>(),
            slice_len in 4usize..8,
            offset in 0usize..usize::MAX-3,
            bad_len in 0usize..4
        ) {

            // ok big endian
            {
                let value_be = value.to_be_bytes();
                let data = [value_be[0], value_be[1], value_be[2], value_be[3], 1, 2, 3, 4];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_i32(true),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 4);
                prop_assert_eq!(slicer.rest, &[1,2,3,4][..slice_len - 4]);
            }
            // ok little endian
            {
                let value_le = value.to_le_bytes();
                let data = [
                    value_le[0], value_le[1], value_le[2], value_le[3], 1, 2, 3, 4
                ];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_i32(false),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 4);
                prop_assert_eq!(slicer.rest, &[1,2,3,4][..slice_len - 4]);
            }

            // length error
            {
                let expected = Err(VerboseDecodeError::UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + 4,
                    }
                ));
                let data = value.to_le_bytes();
                let mut slicer = FieldSlicer::new(&data[..bad_len], offset);

                // little endian
                prop_assert_eq!(slicer.read_i32(false), expected.clone());
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);

                // big endian
                prop_assert_eq!(slicer.read_i32(true), expected);
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_u64(
            value in any::<u64>(),
            slice_len in 8usize..16,
            offset in 0usize..usize::MAX-7,
            bad_len in 0usize..8
        ) {

            // ok big endian
            {
                let value_be = value.to_be_bytes();
                let data = [value_be[0], value_be[1], value_be[2], value_be[3], value_be[4], value_be[5], value_be[6], value_be[7], 1, 2, 3, 4, 5, 6, 7, 8];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_u64(true),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 8);
                prop_assert_eq!(slicer.rest, &[1,2,3,4,5,6,7,8][..slice_len - 8]);
            }
            // ok little endian
            {
                let value_le = value.to_le_bytes();
                let data = [
                    value_le[0], value_le[1], value_le[2], value_le[3], value_le[4], value_le[5], value_le[6], value_le[7], 1, 2, 3, 4, 5, 6, 7, 8
                ];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_u64(false),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 8);
                prop_assert_eq!(slicer.rest, &[1,2,3,4,5,6,7,8][..slice_len - 8]);
            }

            // length error
            {
                let expected = Err(VerboseDecodeError::UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + 8,
                    }
                ));
                let data = value.to_le_bytes();
                let mut slicer = FieldSlicer::new(&data[..bad_len], offset);

                // little endian
                prop_assert_eq!(slicer.read_u64(false), expected.clone());
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);

                // big endian
                prop_assert_eq!(slicer.read_u64(true), expected);
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_i64(
            value in any::<i64>(),
            slice_len in 8usize..16,
            offset in 0usize..usize::MAX-7,
            bad_len in 0usize..8
        ) {

            // ok big endian
            {
                let value_be = value.to_be_bytes();
                let data = [value_be[0], value_be[1], value_be[2], value_be[3], value_be[4], value_be[5], value_be[6], value_be[7], 1, 2, 3, 4, 5, 6, 7, 8];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_i64(true),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 8);
                prop_assert_eq!(slicer.rest, &[1,2,3,4,5,6,7,8][..slice_len - 8]);
            }
            // ok little endian
            {
                let value_le = value.to_le_bytes();
                let data = [
                    value_le[0], value_le[1], value_le[2], value_le[3], value_le[4], value_le[5], value_le[6], value_le[7], 1, 2, 3, 4, 5, 6, 7, 8
                ];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_i64(false),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 8);
                prop_assert_eq!(slicer.rest, &[1,2,3,4,5,6,7,8][..slice_len - 8]);
            }

            // length error
            {
                let expected = Err(VerboseDecodeError::UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + 8,
                    }
                ));
                let data = value.to_le_bytes();
                let mut slicer = FieldSlicer::new(&data[..bad_len], offset);

                // little endian
                prop_assert_eq!(slicer.read_i64(false), expected.clone());
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);

                // big endian
                prop_assert_eq!(slicer.read_i64(true), expected);
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_u128(
            value in any::<u128>(),
            slice_len in 16usize..32,
            offset in 0usize..usize::MAX-15,
            bad_len in 0usize..16
        ) {

            // ok big endian
            {
                let value_be = value.to_be_bytes();
                let data = [value_be[0], value_be[1], value_be[2], value_be[3], value_be[4], value_be[5], value_be[6], value_be[7], value_be[8], value_be[9], value_be[10], value_be[11], value_be[12], value_be[13], value_be[14], value_be[15], 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_u128(true),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 16);
                prop_assert_eq!(slicer.rest, &[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15][..slice_len - 16]);
            }
            // ok little endian
            {
                let value_le = value.to_le_bytes();
                let data = [
                    value_le[0], value_le[1], value_le[2], value_le[3], value_le[4], value_le[5], value_le[6], value_le[7], value_le[8], value_le[9], value_le[10], value_le[11], value_le[12], value_le[13], value_le[14], value_le[15], 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
                ];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_u128(false),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 16);
                prop_assert_eq!(slicer.rest, &[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15][..slice_len - 16]);
            }

            // length error
            {
                let expected = Err(VerboseDecodeError::UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + 16,
                    }
                ));
                let data = value.to_le_bytes();
                let mut slicer = FieldSlicer::new(&data[..bad_len], offset);

                // little endian
                prop_assert_eq!(slicer.read_u128(false), expected.clone());
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);

                // big endian
                prop_assert_eq!(slicer.read_u128(true), expected);
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_i128(
            value in any::<i128>(),
            slice_len in 16usize..32,
            offset in 0usize..usize::MAX-15,
            bad_len in 0usize..16
        ) {

            // ok big endian
            {
                let value_be = value.to_be_bytes();
                let data = [value_be[0], value_be[1], value_be[2], value_be[3], value_be[4], value_be[5], value_be[6], value_be[7], value_be[8], value_be[9], value_be[10], value_be[11], value_be[12], value_be[13], value_be[14], value_be[15], 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_i128(true),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 16);
                prop_assert_eq!(slicer.rest, &[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15][..slice_len - 16]);
            }
            // ok little endian
            {
                let value_le = value.to_le_bytes();
                let data = [
                    value_le[0], value_le[1], value_le[2], value_le[3], value_le[4], value_le[5], value_le[6], value_le[7], value_le[8], value_le[9], value_le[10], value_le[11], value_le[12], value_le[13], value_le[14], value_le[15], 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
                ];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_i128(false),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 16);
                prop_assert_eq!(slicer.rest, &[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15][..slice_len - 16]);
            }

            // length error
            {
                let expected = Err(VerboseDecodeError::UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + 16,
                    }
                ));
                let data = value.to_le_bytes();
                let mut slicer = FieldSlicer::new(&data[..bad_len], offset);

                // little endian
                prop_assert_eq!(slicer.read_i128(false), expected.clone());
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);

                // big endian
                prop_assert_eq!(slicer.read_i128(true), expected);
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);
            }
        }
    }

    proptest! {
        #[test]
        fn read_var_name(
            ref value in "\\PC*",
            offset in 0usize..1024,
            bad_len in 0usize..1024,
            rest in vec(any::<u8>(), 0..4)
        ) {
            use VerboseDecodeError::*;
            // big endian version
            {
                let mut buffer = Vec::with_capacity(2 + value.len() + 1);
                buffer.extend_from_slice(&((value.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(value.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name(true), Ok(value.as_str()));
                prop_assert_eq!(slicer.offset, offset + 2 + value.len() + 1);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // little endian version
            {
                let mut buffer = Vec::with_capacity(2 + value.len() + 1);
                buffer.extend_from_slice(&((value.len() + 1) as u16).to_le_bytes());
                buffer.extend_from_slice(value.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name(false), Ok(value.as_str()));
                prop_assert_eq!(slicer.offset, offset + 2 + value.len() + 1);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // length error (length field)
            for len in 0..2 {
                let expected = Err(UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + len,
                        minimum_size: offset + 2,
                    }
                ));
                {
                    let data = 2u16.to_le_bytes();
                    let mut slicer = FieldSlicer::new(&data[..len], offset);
                    prop_assert_eq!(slicer.read_var_name(false), expected.clone());
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &data[..len]);
                }
                {
                    let data = 2u16.to_be_bytes();
                    let mut slicer = FieldSlicer::new(&data[..len], offset);
                    prop_assert_eq!(slicer.read_var_name(true), expected);
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &data[..len]);
                }
            }

            // length error (string value)
            if value.len() > 0 {

                // make sure the len is actually smaller
                let bad_len = if bad_len >= value.len() {
                    value.len() - 1
                } else {
                    bad_len
                };

                let expected = Err(UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + 2 + bad_len,
                        minimum_size: offset + 2 + value.len(),
                    }
                ));

                // little endian
                {
                    let mut buffer = Vec::with_capacity(2 + value.len());
                    buffer.extend_from_slice(&(value.len() as u16).to_le_bytes());
                    buffer.extend_from_slice(&value.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name(false), expected.clone());
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
                // big endian
                {
                    let mut buffer = Vec::with_capacity(2 + value.len());
                    buffer.extend_from_slice(&(value.len() as u16).to_be_bytes());
                    buffer.extend_from_slice(&value.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name(true), expected);
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
            }

            // zero termination missing
            if value.len() > 0 {
                let mut buffer = Vec::with_capacity(2 + value.len() + rest.len());
                buffer.extend_from_slice(&((value.len()) as u16).to_be_bytes());
                buffer.extend_from_slice(value.as_bytes());

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name(true), Err(VariableNameStringMissingNullTermination));

                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &buffer);
            } else {
                let mut buffer = Vec::with_capacity(2 + value.len() + rest.len());
                buffer.extend_from_slice(&0u16.to_be_bytes());
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name(true), Ok(""));

                prop_assert_eq!(slicer.offset, offset + 2);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // utf8 error
            {
                let mut buffer = Vec::with_capacity(2 + value.len() + 4 + 1 + rest.len());
                buffer.extend_from_slice(&((value.len() + 4 + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(value.as_bytes());
                // some invalid utf8 data
                buffer.extend_from_slice(&[0, 159, 146, 150]);
                buffer.push(0);
                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_var_name(true),
                    Err(Utf8(core::str::from_utf8(&buffer[2..(2 + value.len() + 4)]).unwrap_err()))
                );
            }
        }
    }

    proptest! {
        #[test]
        fn read_var_name_and_unit(
            ref name in "\\PC*",
            ref unit in "\\PC*",
            offset in 0usize..1024,
            bad_len in 0usize..1024,
            rest in vec(any::<u8>(), 0..4)
        ) {
            use VerboseDecodeError::*;

            // big endian version
            {
                let mut buffer = Vec::with_capacity(4 + name.len() + unit.len() + 2);
                buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(unit.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_var_name_and_unit(true),
                    Ok((name.as_str(), unit.as_str()))
                );
                prop_assert_eq!(slicer.offset, offset + 4 + name.len() + unit.len() + 2);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // little endian version
            {
                let mut buffer = Vec::with_capacity(4 + name.len() + unit.len() + 2);
                buffer.extend_from_slice(&((name.len() + 1) as u16).to_le_bytes());
                buffer.extend_from_slice(&((unit.len() + 1) as u16).to_le_bytes());
                buffer.extend_from_slice(name.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(unit.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_var_name_and_unit(false),
                    Ok((name.as_str(), unit.as_str()))
                );
                prop_assert_eq!(slicer.offset, offset + 4 + name.len() + unit.len() + 2);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // length error (length values)
            for len in 0..4 {
                let expected = Err(UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + len,
                        minimum_size: offset + 4,
                    }
                ));
                {
                    let data = [0, 0, 0, 0];
                    let mut slicer = FieldSlicer::new(&data[..len], offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(false), expected.clone());
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &data[..len]);
                }
                {
                    let data = [0, 0, 0, 0];
                    let mut slicer = FieldSlicer::new(&data[..len], offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(true), expected);
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &data[..len]);
                }
            }

            // length error (string name)
            {
                // make sure the len is actually smaller
                let bad_len = if bad_len > name.len() {
                    name.len()
                } else {
                    bad_len
                };

                let expected = Err(UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + 4 + bad_len,
                        minimum_size: offset + 4 + name.len() + 1 + unit.len() + 1,
                    }
                ));

                // little endian
                {
                    let mut buffer = Vec::with_capacity(4 + name.len() + unit.len());
                    buffer.extend_from_slice(&((name.len() + 1) as u16).to_le_bytes());
                    buffer.extend_from_slice(&((unit.len() + 1) as u16).to_le_bytes());
                    buffer.extend_from_slice(&name.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(false), expected.clone());
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
                // big endian
                {
                    let mut buffer = Vec::with_capacity(4 + name.len() + unit.len());
                    buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                    buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                    buffer.extend_from_slice(&name.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(true), expected);
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
            }

            // length error (string unit)
            {
                // make sure the len is actually smaller
                let bad_len = if bad_len > unit.len() {
                    unit.len()
                } else {
                    bad_len
                };

                let expected = Err(UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + 4 + name.len() + 1 + bad_len,
                        minimum_size: offset + 4 + name.len() + 1 + unit.len() + 1,
                    }
                ));

                // little endian
                {
                    let mut buffer = Vec::with_capacity(4 + name.len() + unit.len());
                    buffer.extend_from_slice(&((name.len() + 1) as u16).to_le_bytes());
                    buffer.extend_from_slice(&((unit.len() + 1) as u16).to_le_bytes());
                    buffer.extend_from_slice(&name.as_bytes());
                    buffer.push(0);
                    buffer.extend_from_slice(&unit.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(false), expected.clone());
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
                // big endian
                {
                    let mut buffer = Vec::with_capacity(4 + name.len() + unit.len());
                    buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                    buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                    buffer.extend_from_slice(&name.as_bytes());
                    buffer.push(0);
                    buffer.extend_from_slice(&unit.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(true), expected);
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
            }

            // zero termination error (name)
            if name.len() > 0 {
                let mut buffer = Vec::with_capacity(4 + name.len() + unit.len() + 1 + rest.len());
                buffer.extend_from_slice(&(name.len() as u16).to_be_bytes());
                buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                // skip zero termination
                buffer.extend_from_slice(unit.as_bytes());
                buffer.push(0);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name_and_unit(true), Err(VariableNameStringMissingNullTermination));

                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &buffer);
            } else {
                // strings with length 0 are allowed to have no zero termination
                let mut buffer = Vec::with_capacity(4 + unit.len() + 0 + rest.len());
                buffer.extend_from_slice(&(0 as u16).to_be_bytes());
                buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                // skip name as it has len 0,
                buffer.extend_from_slice(unit.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name_and_unit(true), Ok(("", unit.as_str())));

                prop_assert_eq!(slicer.offset, offset + 4 + unit.len() + 1);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // zero termination error (unit)
            if unit.len() > 0 {
                let mut buffer = Vec::with_capacity(4 + name.len() + 1 + unit.len() + rest.len());
                buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(&(unit.len() as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(unit.as_bytes());
                // skip zero termination

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name_and_unit(true), Err(VariableUnitStringMissingNullTermination));

                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &buffer);
            } else {
                // strings with length 0 are allowed to have no zero termination
                let mut buffer = Vec::with_capacity(4 + name.len() + 1 + rest.len());
                buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(&(0 as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                buffer.push(0);
                // skip unit as it has len 0,
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name_and_unit(true), Ok((name.as_str(), "")));

                prop_assert_eq!(slicer.offset, offset + 4 + name.len() + 1);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // utf8 error (name)
            {
                let mut buffer = Vec::with_capacity(4 + name.len() + 4 + 1 + unit.len() + 1 + rest.len());
                buffer.extend_from_slice(&((name.len() + 4 + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                // some invalid utf8 data
                buffer.extend_from_slice(&[0, 159, 146, 150]);
                buffer.push(0);
                buffer.extend_from_slice(unit.as_bytes());
                buffer.push(0);
                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_var_name_and_unit(true),
                    Err(Utf8(core::str::from_utf8(&buffer[4..(4 + name.len() + 4)]).unwrap_err()))
                );
            }

            // utf8 error (name)
            {
                let mut buffer = Vec::with_capacity(4 + name.len() + 1 + unit.len() + 4 + 1 + rest.len());
                buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(&((unit.len() + 4 + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(unit.as_bytes());
                // some invalid utf8 data
                buffer.extend_from_slice(&[0, 159, 146, 150]);
                buffer.push(0);
                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_var_name_and_unit(true),
                    Err(Utf8(core::str::from_utf8(&buffer[(4 + name.len() + 1)..(4 + name.len() + 1 + unit.len() + 4)]).unwrap_err()))
                );
            }
        }
    }

    proptest! {
        #[test]
        fn read_raw(
            data in prop::collection::vec(any::<u8>(), 0..1024),
            offset in 0usize..1024,
            rest in prop::collection::vec(any::<u8>(), 0..10),
            bad_len in 0usize..1024,
        ) {
            // ok case
            {
                let mut buffer = Vec::with_capacity(data.len() + rest.len());
                buffer.extend_from_slice(&data);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_raw(data.len()), Ok(&data[..]));
                prop_assert_eq!(slicer.offset, offset + data.len());
                prop_assert_eq!(slicer.rest, &rest);
            }

            // length error
            if data.len() > 0 {

                // make sure the len is actually smaller
                let bad_len = if bad_len >= data.len() {
                    data.len() - 1
                } else {
                    bad_len
                };

                let mut buffer = Vec::with_capacity(data.len());
                buffer.extend_from_slice(&data[..bad_len]);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_raw(data.len()),
                    Err(VerboseDecodeError::UnexpectedEndOfSlice(UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + data.len(),
                    }))
                );
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &buffer[..]);
            }
        }
    }
}
