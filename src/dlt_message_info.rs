use crate::{
    DltMessageType, EXT_MSIN_MSTP_TYPE_CONTROL, EXT_MSIN_MSTP_TYPE_LOG,
    EXT_MSIN_MSTP_TYPE_NW_TRACE, EXT_MSIN_MSTP_TYPE_TRACE,
};

/// Message info identifying the type of message (e.g. log, trace, network trace & control).
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DltMessageInfo(pub u8);

impl DltMessageInfo {
    /// Returns if the message is a verbose dlt message.
    #[inline]
    pub fn is_verbose(&self) -> bool {
        0 != self.0 & 0b0000_0001
    }

    /// Returns if the message is a log message.
    #[inline]
    pub fn is_log(&self) -> bool {
        EXT_MSIN_MSTP_TYPE_LOG == self.0 & 0b0000_1110
    }

    /// Returns if the message is a trace message.
    #[inline]
    pub fn is_trace(&self) -> bool {
        EXT_MSIN_MSTP_TYPE_TRACE == self.0 & 0b0000_1110
    }

    /// Returns if the message is a trace message.
    #[inline]
    pub fn is_network(&self) -> bool {
        EXT_MSIN_MSTP_TYPE_NW_TRACE == self.0 & 0b0000_1110
    }

    /// Returns if the message is a control message.
    #[inline]
    pub fn is_control(&self) -> bool {
        EXT_MSIN_MSTP_TYPE_CONTROL == self.0 & 0b0000_1110
    }

    /// Returns the message type.
    #[inline]
    pub fn into_message_type(&self) -> Option<DltMessageType> {
        DltMessageType::from_byte(self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_checks() {
        for v in 0..=u8::MAX {
            let info = DltMessageInfo(v);
            if 0 != v & 0b1 {
                assert!(info.is_verbose());
            } else {
                assert_eq!(false, info.is_verbose());
            }
            let mstp = (v & 0b0000_1110) >> 1;
            if mstp == 0 {
                assert!(info.is_log());
            } else {
                assert_eq!(false, info.is_log());
            }
            if mstp == 1 {
                assert!(info.is_trace());
            } else {
                assert_eq!(false, info.is_trace());
            }
            if mstp == 2 {
                assert!(info.is_network());
            } else {
                assert_eq!(false, info.is_network());
            }
            if mstp == 3 {
                assert!(info.is_control());
            } else {
                assert_eq!(false, info.is_control());
            }
        }
    }

    #[test]
    fn into_message_type() {
        for v in 0..=u8::MAX {
            let info = DltMessageInfo(v);
            assert_eq!(info.into_message_type(), DltMessageType::from_byte(v));
        }
    }
}
