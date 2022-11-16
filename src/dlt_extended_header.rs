use super::*;

///Extended dlt header (optional header in the dlt header)
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DltExtendedHeader {
    pub message_info: u8,
    pub number_of_arguments: u8,
    pub application_id: [u8; 4],
    pub context_id: [u8; 4],
}

impl DltExtendedHeader {
    ///Create a extended header for a non verbose log message with given application id & context id.
    pub fn new_non_verbose_log(
        log_level: DltLogLevel,
        application_id: [u8; 4],
        context_id: [u8; 4],
    ) -> DltExtendedHeader {
        DltExtendedHeader {
            message_info: DltMessageType::Log(log_level).to_byte().unwrap(),
            number_of_arguments: 0,
            application_id,
            context_id,
        }
    }

    ///Create a extended header for a non verbose message with given message type, application id & context id.
    pub fn new_non_verbose(
        message_type: DltMessageType,
        application_id: [u8; 4],
        context_id: [u8; 4],
    ) -> Result<DltExtendedHeader, error::RangeError> {
        Ok(DltExtendedHeader {
            message_info: message_type.to_byte()?,
            number_of_arguments: 0,
            application_id,
            context_id,
        })
    }

    ///Returns true if the extended header flags the message as a verbose message.
    #[inline]
    pub fn is_verbose(&self) -> bool {
        0 != self.message_info & 0b1
    }

    ///Sets or unsets the is_verbose bit in the DltExtendedHeader.
    #[inline]
    pub fn set_is_verbose(&mut self, is_verbose: bool) {
        if is_verbose {
            self.message_info |= 0b1;
        } else {
            self.message_info &= 0b1111_1110;
        }
    }

    ///Returns message type info or `Option::None` for reserved values.
    #[inline]
    pub fn message_type(&self) -> Option<DltMessageType> {
        DltMessageType::from_byte(self.message_info)
    }

    ///Set message type info and based on that the message type.
    #[inline]
    pub fn set_message_type(&mut self, value: DltMessageType) -> Result<(), error::RangeError> {
        let encoded = value.to_byte()?;

        //unset old message type & set the new one
        self.message_info &= 0b0000_0001;
        self.message_info |= encoded;

        //all good
        Ok(())
    }
}

/// Tests for `DltExtendedHeader` methods
#[cfg(test)]
mod dlt_extended_header_tests {

    use super::*;
    use crate::proptest_generators::*;
    use proptest::prelude::*;

    #[test]
    fn clone_eq() {
        let header: DltExtendedHeader = Default::default();
        assert_eq!(header, header.clone());
    }

    #[test]
    fn debug() {
        let header: DltExtendedHeader = Default::default();
        assert_eq!(
                format!(
                    "DltExtendedHeader {{ message_info: {:?}, number_of_arguments: {:?}, application_id: {:?}, context_id: {:?} }}",
                    header.message_info,
                    header.number_of_arguments,
                    header.application_id,
                    header.context_id
                ),
                format!("{:?}", header)
            );
    }

    #[test]
    fn default() {
        let header: DltExtendedHeader = Default::default();
        assert_eq!(header.message_info, 0);
        assert_eq!(header.number_of_arguments, 0);
        assert_eq!(header.application_id, [0, 0, 0, 0]);
        assert_eq!(header.context_id, [0, 0, 0, 0]);
    }

    proptest! {
        #[test]
        fn new_non_verbose_log(
            log_level in log_level_any(),
            application_id in any::<[u8;4]>(),
            context_id in any::<[u8;4]>())
        {
            use DltMessageType::Log;
            let header = DltExtendedHeader::new_non_verbose_log(log_level.clone(), application_id, context_id);
            assert_eq!(Log(log_level).to_byte().unwrap(), header.message_info);
            assert_eq!(0, header.number_of_arguments);
            assert_eq!(application_id, header.application_id);
            assert_eq!(context_id, header.context_id);
        }
    }

    proptest! {
        #[test]
        fn new_non_verbose(
            message_type in message_type_any(),
            application_id in any::<[u8;4]>(),
            context_id in any::<[u8;4]>(),
            invalid_user_defined in 0x10..0xffu8
        ) {
            // valid data
            {
                let header = DltExtendedHeader::new_non_verbose(
                    message_type.clone(),
                    application_id,
                    context_id
                ).unwrap();
                assert_eq!(message_type.to_byte().unwrap(), header.message_info);
                assert_eq!(0, header.number_of_arguments);
                assert_eq!(application_id, header.application_id);
                assert_eq!(context_id, header.context_id);
            }

            // invalid data
            {
                use DltMessageType::NetworkTrace;
                use DltNetworkType::UserDefined;
                use error::RangeError::NetworkTypekUserDefinedOutsideOfRange;

                let result = DltExtendedHeader::new_non_verbose(
                    NetworkTrace(UserDefined(invalid_user_defined)),
                    application_id,
                    context_id
                ).unwrap_err();
                assert_eq!(NetworkTypekUserDefinedOutsideOfRange(invalid_user_defined), result);
            }
        }
    }

    #[test]
    fn set_is_verbose() {
        let mut header: DltExtendedHeader = Default::default();
        let original = header.clone();
        header.set_is_verbose(true);
        assert_eq!(true, header.is_verbose());
        header.set_is_verbose(false);
        assert_eq!(false, header.is_verbose());
        assert_eq!(original, header);
    }

    proptest! {
        #[test]
        fn set_message_type(
            verbose in any::<bool>(),
            message_type0 in message_type_any(),
            message_type1 in message_type_any())
        {
            let mut header: DltExtendedHeader = Default::default();

            //set verbose (stored in same field, to ensure no side effects)
            header.set_is_verbose(verbose);
            assert_eq!(header.is_verbose(), verbose);

            //set to first message type
            header.set_message_type(message_type0.clone()).unwrap();
            assert_eq!(header.is_verbose(), verbose);
            assert_eq!(header.message_type(), Some(message_type0));

            //set to second message type (to make sure the old type is correctly cleaned)
            header.set_message_type(message_type1.clone()).unwrap();
            assert_eq!(header.is_verbose(), verbose);
            assert_eq!(header.message_type(), Some(message_type1));
        }
    }

    #[test]
    fn message_type() {
        use {
            DltControlMessageType::*, DltLogLevel::*, DltMessageType::*, DltNetworkType::*,
            DltTraceType::*,
        };

        //check that setting & resetting does correctly reset the values
        {
            let mut header = DltExtendedHeader::new_non_verbose_log(
                Fatal,
                Default::default(),
                Default::default(),
            );

            header.set_message_type(NetworkTrace(SomeIp)).unwrap();
            assert_eq!(false, header.is_verbose());
            assert_eq!(Some(NetworkTrace(SomeIp)), header.message_type());

            //set to a different value with non overlapping bits (to make sure the values are reset)
            header.set_message_type(Trace(FunctionIn)).unwrap();
            assert_eq!(false, header.is_verbose());
            assert_eq!(Some(Trace(FunctionIn)), header.message_type());
        }

        //check None return type when a unknown value is presented
        //message type
        for message_type_id in 4..=0b111 {
            let mut header = DltExtendedHeader::new_non_verbose_log(
                Fatal,
                Default::default(),
                Default::default(),
            );
            header.message_info = message_type_id << 1;
            assert_eq!(None, header.message_type());
        }

        //msin bad values
        let bad_values = [
            //bad log level 0 & everything above 6
            (Log(Fatal), (0u8..1).chain(7u8..=0xf)),
            //bad trace source (0 & everything above 5)
            (Trace(FunctionIn), (0u8..1).chain(6u8..=0xf)),
            //bad control message type (0 & everything above 2)
            (Control(Request), (0u8..1).chain(3u8..=0xf)),
        ];

        for t in bad_values.iter() {
            for value in t.1.clone() {
                let mut header = DltExtendedHeader::new_non_verbose(
                    t.0.clone(),
                    Default::default(),
                    Default::default(),
                )
                .unwrap();
                header.message_info &= 0b0000_1111;
                header.message_info |= value << 4;
                assert_eq!(None, header.message_type());
            }
        }

        //check set out of range error
        {
            use error::RangeError::*;
            use DltLogLevel::Fatal;
            for i in 0x10..=0xff {
                let mut header = DltExtendedHeader::new_non_verbose_log(
                    Fatal,
                    Default::default(),
                    Default::default(),
                );
                assert_eq!(
                    Err(NetworkTypekUserDefinedOutsideOfRange(i)),
                    header.set_message_type(NetworkTrace(UserDefined(i)))
                );
            }
        }
    }
} // mod dlt_extended_header_tests
