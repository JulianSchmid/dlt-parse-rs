/// "Set Log Level" service id
pub const CMD_ID_SET_LOG_LEVEL: u32 = 0x01;
/// "Set Log Level" name
pub const CMD_NAME_SET_LOG_LEVEL: &str = "SetLogLevel";

/// "Set Trace Status" service id
pub const CMD_ID_SET_TRACE_STATUS: u32 = 0x02;
/// "Set Trace Status" name
pub const CMD_NAME_SET_TRACE_STATUS: &str = "SetTraceStatus";

/// "Get Log Info" service id
pub const CMD_ID_GET_LOG_INFO: u32 = 0x03;
/// "Get Log Info" name
pub const CMD_NAME_GET_LOG_INFO: &str = "GetLogInfo";

/// "Get Default Log Level" service id.
pub const CMD_ID_GET_DEFAULT_LOG_LEVEL: u32 = 0x04;
/// "Get Default Log Level" name.
pub const CMD_NAME_GET_DEFAULT_LOG_LEVEL: &str = "GetDefaultLogLevel";

/// "Store Configuration" service id.
pub const CMD_ID_STORE_CONFIGURATION: u32 = 0x05;
/// "Store Configuration" name.
pub const CMD_NAME_STORE_CONFIGURATION: &str = "StoreConfiguration";

/// "Reset to Factory Default" service id.
pub const CMD_ID_RESET_TO_FACTORY_DEFAULT: u32 = 0x06;
/// "Reset to Factory Default" name.
pub const CMD_NAME_RESET_TO_FACTORY_DEFAULT: &str = "ResetToFactoryDefault";

/// "Set Message Filtering" service id.
pub const CMD_ID_SET_MESSAGE_FILTERING: u32 = 0x0A;
/// "Set Message Filtering" name.
pub const CMD_NAME_SET_MESSAGE_FILTERING: &str = "SetMessageFiltering";

/// "Set Default LogLevel" service id.
pub const CMD_ID_SET_DEFAULT_LOG_LEVEL: u32 = 0x11;
/// "Set Default LogLevel" name.
pub const CMD_NAME_SET_DEFAULT_LOG_LEVEL: &str = "SetDefaultLogLevel";

/// "Set Default Trace Status" service id.
pub const CMD_ID_SET_DEFAULT_TRACE_STATUS: u32 = 0x12;
/// "Set Default Trace Status" name.
pub const CMD_NAME_SET_DEFAULT_TRACE_STATUS: &str = "SetDefaultTraceStatus";

/// "Get ECU Software Version" service id.
pub const CMD_ID_GET_SOFTWARE_VERSION: u32 = 0x13;
/// "Get ECU Software Version" name.
pub const CMD_NAME_GET_SOFTWARE_VERSION: &str = "GetSoftwareVersion";

/// "Get Default Trace Status" service id.
pub const CMD_ID_GET_DEFAULT_TRACE_STATUS: u32 = 0x15;
/// "Get Default Trace Status" name.
pub const CMD_NAME_GET_DEFAULT_TRACE_STATUS: &str = "GetDefaultTraceStatus";

/// "Get LogChannel Names" service id.
pub const CMD_ID_GET_LOG_CHANNEL_NAMES: u32 = 0x17;
/// "Get LogChannel Names" name.
pub const CMD_NAME_GET_LOG_CHANNEL_NAMES: &str = "GetLogChannelNames";

/// "Get Trace Status" service id.
pub const CMD_ID_GET_TRACE_STATUS: u32 = 0x1F;
/// "Get Trace Status" name.
pub const CMD_NAME_GET_TRACE_STATUS: &str = "GetTraceStatus";

/// "Set LogChannel Assignment" service id.
pub const CMD_ID_SET_LOG_CHANNEL_ASSIGNMENT: u32 = 0x20;
/// "Set LogChannel Assignment" name.
pub const CMD_NAME_SET_LOG_CHANNEL_ASSIGNMENT: &str = "SetLogChannelAssignment";

/// "Set LogChannel Threshold" service id.
pub const CMD_ID_SET_LOG_CHANNEL_THRESHOLD: u32 = 0x21;
/// "Set LogChannel Threshold" name.
pub const CMD_NAME_SET_LOG_CHANNEL_THRESHOLD: &str = "SetLogChannelThreshold";

/// "Get LogChannel Threshold" service id.
pub const CMD_ID_GET_LOG_CHANNEL_THRESHOLD: u32 = 0x22;
/// "Get LogChannel Threshold" name.
pub const CMD_NAME_GET_LOG_CHANNEL_THRESHOLD: &str = "GetLogChannelThreshold";

/// "BufferOverflowNotification" service id.
pub const CMD_ID_BUFFER_OVERFLOW_NOTIFICATION: u32 = 0x23;
/// "BufferOverflowNotification" name.
pub const CMD_NAME_BUFFER_OVERFLOW_NOTIFICATION: &str = "BufferOverflowNotification";

/// "Call SWC Injection" service ids range.
pub const CMD_IDS_CALL_SWC_INJECTIONS: core::ops::RangeInclusive<u32> = 0xFFF..=0xFFFFFFFF;
/// "Call SWC Injection" name.
pub const CMD_NAME_CALL_SWC_INJECTIONS: &str = "CallSWCInjection";

/// Get the name of the service based on the service id given.
pub fn get_control_command_name(service_id: u32) -> Option<&'static str> {
    match service_id {
        0x01 => Some(CMD_NAME_SET_LOG_LEVEL),
        0x02 => Some(CMD_NAME_SET_TRACE_STATUS),
        0x03 => Some(CMD_NAME_GET_LOG_INFO),
        0x04 => Some(CMD_NAME_GET_DEFAULT_LOG_LEVEL),
        0x05 => Some(CMD_NAME_STORE_CONFIGURATION),
        0x06 => Some(CMD_NAME_RESET_TO_FACTORY_DEFAULT),
        0x0A => Some(CMD_NAME_SET_MESSAGE_FILTERING),
        0x11 => Some(CMD_NAME_SET_DEFAULT_LOG_LEVEL),
        0x12 => Some(CMD_NAME_SET_DEFAULT_TRACE_STATUS),
        0x13 => Some(CMD_NAME_GET_SOFTWARE_VERSION),
        0x15 => Some(CMD_NAME_GET_DEFAULT_TRACE_STATUS),
        0x17 => Some(CMD_NAME_GET_LOG_CHANNEL_NAMES),
        0x1F => Some(CMD_NAME_GET_TRACE_STATUS),
        0x20 => Some(CMD_NAME_SET_LOG_CHANNEL_ASSIGNMENT),
        0x21 => Some(CMD_NAME_SET_LOG_CHANNEL_THRESHOLD),
        0x22 => Some(CMD_NAME_GET_LOG_CHANNEL_THRESHOLD),
        0x23 => Some(CMD_NAME_BUFFER_OVERFLOW_NOTIFICATION),
        0xFFF..=0xFFFFFFFF => Some(CMD_NAME_CALL_SWC_INJECTIONS),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_get_control_command_name(
            unknown_id in 0x24..0xFFFu32,
            sw_injections_id in 0xFFF..=0xFFFFFFFFu32
        ) {
            let tests = [
                (0x00, None),
                (0x01, Some("SetLogLevel")),
                (0x02, Some("SetTraceStatus")),
                (0x03, Some("GetLogInfo")),
                (0x04, Some("GetDefaultLogLevel")),
                (0x05, Some("StoreConfiguration")),
                (0x06, Some("ResetToFactoryDefault")),
                (0x07, None),
                (0x08, None),
                (0x09, None),
                (0x0A, Some("SetMessageFiltering")),
                (0x0B, None),
                (0x0C, None),
                (0x0D, None),
                (0x0E, None),
                (0x0F, None),
                (0x10, None),
                (0x11, Some("SetDefaultLogLevel")),
                (0x12, Some("SetDefaultTraceStatus")),
                (0x13, Some("GetSoftwareVersion")),
                (0x14, None),
                (0x15, Some("GetDefaultTraceStatus")),
                (0x16, None),
                (0x17, Some("GetLogChannelNames")),
                (0x18, None),
                (0x19, None),
                (0x1A, None),
                (0x1B, None),
                (0x1C, None),
                (0x1D, None),
                (0x1F, Some("GetTraceStatus")),
                (0x20, Some("SetLogChannelAssignment")),
                (0x21, Some("SetLogChannelThreshold")),
                (0x22, Some("GetLogChannelThreshold")),
                (0x23, Some("BufferOverflowNotification")),
            ];
            for test in tests {
                assert_eq!(test.1, get_control_command_name(test.0))
            }
            assert_eq!(None, get_control_command_name(unknown_id));
            assert_eq!(Some("CallSWCInjection"), get_control_command_name(sw_injections_id));
        }
    }
}
