use crate::*;

/// A non verbose message of unknown type.
///
/// This type is used if the DLT message did not contain an extended
/// header and the type of message can only be determined via the
/// message id. In this the type of th message can only be determined
/// based on the message id and an additional information source describing
/// how to decode the message payloads and what type of message it is
/// (e.g. a Fibex file).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NvPayload<'a> {
    pub msg_id: u32,
    pub payload: &'a [u8],
}

impl<'a> From<LogNvPayload<'a>> for NvPayload<'a> {
    fn from(value: LogNvPayload<'a>) -> Self {
        NvPayload {
            msg_id: value.msg_id,
            payload: value.payload,
        }
    }
}

impl<'a> From<TraceNvPayload<'a>> for NvPayload<'a> {
    fn from(value: TraceNvPayload<'a>) -> Self {
        NvPayload {
            msg_id: value.msg_id,
            payload: value.payload,
        }
    }
}

impl<'a> From<NetworkNvPayload<'a>> for NvPayload<'a> {
    fn from(value: NetworkNvPayload<'a>) -> Self {
        NvPayload {
            msg_id: value.msg_id,
            payload: value.payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from() {
        let data = [5, 6, 7, 8];
        let msg_id = 1234_5678u32;
        let payload = &data;

        // LogNvPayload
        assert_eq!(
            NvPayload::from(LogNvPayload {
                msg_id,
                payload,
                log_level: DltLogLevel::Info
            }),
            NvPayload { msg_id, payload }
        );

        // TraceNvPayload
        assert_eq!(
            NvPayload::from(TraceNvPayload {
                msg_id,
                payload,
                trace_type: DltTraceType::State
            }),
            NvPayload { msg_id, payload }
        );

        // TraceNvPayload
        assert_eq!(
            NvPayload::from(NetworkNvPayload {
                msg_id,
                payload,
                net_type: DltNetworkType::Flexray
            }),
            NvPayload { msg_id, payload }
        );
    }
}
