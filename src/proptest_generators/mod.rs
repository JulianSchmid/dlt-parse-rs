use super::*;
use proptest::prelude::*;
use proptest::*;

/// Maximum size of payload when testing
const TEST_MAX_PAYLOAD_SIZE: usize = 1234;

prop_compose! {
    pub fn extended_dlt_header_any()(message_info in any::<u8>(),
                                 number_of_arguments in any::<u8>(),
                                 application_id in any::<[u8;4]>(),
                                 context_id in any::<[u8;4]>()) -> DltExtendedHeader
    {
        DltExtendedHeader {
            message_info: DltMessageInfo(message_info),
            number_of_arguments: number_of_arguments,
            application_id: application_id,
            context_id: context_id
        }
    }
}

prop_compose! {
    pub fn dlt_header_with_payload_any()(
        payload_length in 0usize..TEST_MAX_PAYLOAD_SIZE
    )(
        is_big_endian in any::<bool>(),
        message_counter in any::<u8>(),
        ecu_id in any::<Option<[u8;4]>>(),
        session_id in any::<Option<u32>>(),
        timestamp in any::<Option<u32>>(),
        extended_header in option::of(extended_dlt_header_any()),
        payload in proptest::collection::vec(any::<u8>(), payload_length)
    ) -> (DltHeader, Vec<u8>)
    {
        (
            {
                let mut header = DltHeader {
                    is_big_endian,
                    message_counter,
                    length: payload.len() as u16,
                    ecu_id,
                    session_id,
                    timestamp,
                    extended_header
                };
                let header_size = header.header_len();
                header.length = header_size + (payload.len() as u16);
                header
            },
            payload
        )
    }
}

prop_compose! {
    pub fn dlt_header_any()(is_big_endian in any::<bool>(),
                        message_counter in any::<u8>(),
                        length in any::<u16>(),
                        ecu_id in any::<Option<[u8;4]>>(),
                        session_id in any::<Option<u32>>(),
                        timestamp in any::<Option<u32>>(),
                        extended_header in option::of(extended_dlt_header_any())) -> DltHeader
    {
        DltHeader {
            is_big_endian,
            message_counter,
            length,
            ecu_id,
            session_id,
            timestamp,
            extended_header
        }
    }
}

prop_compose! {
    pub fn storage_header_any()(
        timestamp_seconds in any::<u32>(),
        timestamp_microseconds in any::<u32>(),
        ecu_id in any::<[u8;4]>()
    ) -> storage::StorageHeader {
        storage::StorageHeader{
            timestamp_seconds,
            timestamp_microseconds,
            ecu_id
        }
    }
}

pub fn log_level_any() -> impl Strategy<Value = DltLogLevel> {
    use DltLogLevel::*;
    prop_oneof![
        Just(Fatal),
        Just(Error),
        Just(Warn),
        Just(Info),
        Just(Debug),
        Just(Verbose),
    ]
}

pub fn message_type_any() -> impl Strategy<Value = DltMessageType> {
    use DltControlMessageType::*;
    use DltLogLevel::*;
    use DltMessageType::*;
    use DltNetworkType::*;
    use DltTraceType::*;
    prop_oneof![
        Just(Log(Fatal)),
        Just(Log(Error)),
        Just(Log(Warn)),
        Just(Log(Info)),
        Just(Log(Debug)),
        Just(Log(Verbose)),
        Just(Trace(Variable)),
        Just(Trace(FunctionIn)),
        Just(Trace(FunctionOut)),
        Just(Trace(State)),
        Just(Trace(Vfb)),
        Just(NetworkTrace(Ipc)),
        Just(NetworkTrace(Can)),
        Just(NetworkTrace(Flexray)),
        Just(NetworkTrace(Most)),
        Just(NetworkTrace(Ethernet)),
        Just(NetworkTrace(SomeIp)),
        Just(NetworkTrace(UserDefined(0x7))),
        Just(NetworkTrace(UserDefined(0x8))),
        Just(NetworkTrace(UserDefined(0x9))),
        Just(NetworkTrace(UserDefined(0xA))),
        Just(NetworkTrace(UserDefined(0xB))),
        Just(NetworkTrace(UserDefined(0xC))),
        Just(NetworkTrace(UserDefined(0xD))),
        Just(NetworkTrace(UserDefined(0xE))),
        Just(NetworkTrace(UserDefined(0xF))),
        Just(Control(Request)),
        Just(Control(Response)),
    ]
}
