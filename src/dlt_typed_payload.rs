use crate::{verbose::VerboseIter, ControlMessage, DltMessageInfo};

/// Payload of a DLT log message.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DltTypedPayload<'a> {
    /// Verbose DLT message payload.
    Verbose {
        info: DltMessageInfo,
        iter: VerboseIter<'a>,
    },
    /// Non-verbose DLT message info, message id and payload.
    NonVerbose {
        info: Option<DltMessageInfo>,
        msg_id: u32,
        payload: &'a [u8],
        control_message: Option<ControlMessage<'a>>,
    },
}
