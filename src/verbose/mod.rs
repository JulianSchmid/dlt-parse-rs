mod field_slicer;
use field_slicer::*;

mod values;
pub use values::*;

mod verbose_iter;
pub use verbose_iter::*;

mod pre_checked_verbose_iter;
pub use pre_checked_verbose_iter::*;

mod verbose_value;
pub use verbose_value::*;

use super::*;
use core::str;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Scaling<T: Sized> {
    quantization: f32,
    offset: T,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VariableInfoUnit<'a> {
    name: &'a str,
    unit: &'a str,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArrayDimensions<'a> {
    /// If true the dimesions u16 are encoded in big endian.
    is_big_endian: bool,
    /// Pointer to the raw dimensions data.
    dimensions: &'a [u8],
}

impl<'a> ArrayDimensions<'a> {
    pub fn iter(&'a self) -> ArrayDimensionIterator<'a> {
        ArrayDimensionIterator {
            is_big_endian: self.is_big_endian,
            rest: self.dimensions,
        }
    }
}

impl<'a> IntoIterator for &'a ArrayDimensions<'a> {
    type Item = u16;
    type IntoIter = ArrayDimensionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub struct ArrayDimensionIterator<'a> {
    is_big_endian: bool,
    rest: &'a [u8],
}

impl<'a> Iterator for ArrayDimensionIterator<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.len() < 2 {
            None
        } else {
            let result = if self.is_big_endian {
                u16::from_be_bytes([self.rest[0], self.rest[1]])
            } else {
                u16::from_le_bytes([self.rest[0], self.rest[1]])
            };
            self.rest = &self.rest[2..];
            Some(result)
        }
    }
}
