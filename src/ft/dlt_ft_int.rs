use crate::verbose::*;

/// Signed integer (either 32 or 64 bit).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DltFtInt {
    I32(i32),
    I64(i64),
}

impl DltFtInt {
    pub fn try_take_from_iter(iter: &mut VerboseIter) -> Option<DltFtInt> {
        let Some(Ok(value)) = iter.next() else {
            return None;
        };
        if value.name().is_some() {
            return None;
        }
        match value {
            VerboseValue::I32(v) => Some(DltFtInt::I32(v.value)),
            VerboseValue::I64(v) => Some(DltFtInt::I64(v.value)),
            _ => None,
        }
    }
}

impl From<i32> for DltFtInt {
    fn from(value: i32) -> Self {
        DltFtInt::I32(value)
    }
}

impl From<i64> for DltFtInt {
    fn from(value: i64) -> Self {
        DltFtInt::I64(value)
    }
}

#[cfg(target_pointer_width = "32")]
impl From<isize> for DltFtInt {
    fn from(value: isize) -> Self {
        DltFtInt::I32(value as i32)
    }
}

#[cfg(target_pointer_width = "64")]
#[cfg_attr(docsrs, doc(cfg(target_pointer_width = "64")))]
impl From<isize> for DltFtInt {
    fn from(value: isize) -> Self {
        DltFtInt::I64(value as i64)
    }
}

#[cfg(target_pointer_width = "64")]
#[cfg_attr(docsrs, doc(cfg(target_pointer_width = "64")))]
impl From<DltFtInt> for isize {
    fn from(value: DltFtInt) -> Self {
        match value {
            DltFtInt::I32(v) => v as isize,
            DltFtInt::I64(v) => v as isize,
        }
    }
}

impl From<DltFtInt> for i64 {
    fn from(value: DltFtInt) -> Self {
        match value {
            DltFtInt::I32(v) => v as i64,
            DltFtInt::I64(v) => v,
        }
    }
}
