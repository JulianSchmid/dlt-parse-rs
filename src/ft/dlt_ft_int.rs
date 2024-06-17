
/// Signed integer (either 32 or 64 bit).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DltFtInt {
    I32(i32),
    I64(i64),
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
impl From<isize> for DltFtInt {
    fn from(value: isize) -> Self {
        DltFtInt::I64(value as i64)
    }
}

#[cfg(target_pointer_width = "64")]
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
