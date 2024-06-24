use crate::verbose::*;

/// Unsigned integer (either 32 or 64 bit).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DltFtUInt {
    U16(u16),
    U32(u32),
    U64(u64),
}

impl DltFtUInt {
    pub fn try_take_from_iter(iter: &mut VerboseIter) -> Option<DltFtUInt> {
        let Some(Ok(value)) = iter.next() else {
            return None;
        };
        if value.name().is_some() {
            return None;
        }
        match value {
            VerboseValue::U16(v) => Some(DltFtUInt::U16(v.value)),
            VerboseValue::U32(v) => Some(DltFtUInt::U32(v.value)),
            VerboseValue::U64(v) => Some(DltFtUInt::U64(v.value)),
            _ => None,
        }
    }
}

impl From<u32> for DltFtUInt {
    fn from(value: u32) -> Self {
        DltFtUInt::U32(value)
    }
}

impl From<u64> for DltFtUInt {
    fn from(value: u64) -> Self {
        DltFtUInt::U64(value)
    }
}

#[cfg(target_pointer_width = "32")]
impl From<usize> for DltFtUInt {
    fn from(value: usize) -> Self {
        DltFtUInt::U32(value as u32)
    }
}

#[cfg(target_pointer_width = "64")]
#[cfg_attr(docsrs, doc(cfg(target_pointer_width = "64")))]
impl From<usize> for DltFtUInt {
    fn from(value: usize) -> Self {
        DltFtUInt::U64(value as u64)
    }
}

#[cfg(target_pointer_width = "64")]
#[cfg_attr(docsrs, doc(cfg(target_pointer_width = "64")))]
impl From<DltFtUInt> for usize {
    fn from(value: DltFtUInt) -> Self {
        match value {
            DltFtUInt::U16(v) => usize::from(v),
            DltFtUInt::U32(v) => v as usize,
            DltFtUInt::U64(v) => v as usize,
        }
    }
}

impl From<DltFtUInt> for u64 {
    fn from(value: DltFtUInt) -> Self {
        match value {
            DltFtUInt::U16(v) => u64::from(v),
            DltFtUInt::U32(v) => u64::from(v),
            DltFtUInt::U64(v) => v,
        }
    }
}
