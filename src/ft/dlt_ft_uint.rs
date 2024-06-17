
/// Unsigned integer (either 32 or 64 bit).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DltFtUInt {
    U32(u32),
    U64(u64),
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
impl From<usize> for DltFtUInt {
    fn from(value: usize) -> Self {
        DltFtUInt::U64(value as u64)
    }
}

#[cfg(target_pointer_width = "64")]
impl From<DltFtUInt> for usize {
    fn from(value: DltFtUInt) -> Self {
        match value {
            DltFtUInt::U32(v) => v as usize,
            DltFtUInt::U64(v) => v as usize,
        }
    }
}

impl From<DltFtUInt> for u64 {
    fn from(value: DltFtUInt) -> Self {
        match value {
            DltFtUInt::U32(v) => v as u64,
            DltFtUInt::U64(v) => v,
        }
    }
}
