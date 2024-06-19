#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
use crate::{error::*, ft::*};
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
use core::hash::Hash;
#[cfg(feature = "std")]
use std::{vec::Vec, collections::HashMap};

/// Pool of buffers to reconstruct multiple DLT file transfer packet streams in
/// parallel (re-uses buffers to minimize allocations).
///
/// # This implementation is NOT safe against "Out of Memory" attacks
///
/// If you use the [`DltFtPool`] in an untrusted environment an attacker could
/// cause an "out of memory error" by opening up multiple parallel file transfer streams,
/// never ending them and filling them up with as much data as possible.
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[derive(Debug, Clone)]
pub struct DltFtPool<ChannelId, Timestamp>
where
    ChannelId: Hash + Eq + PartialEq + Clone + Sized,
    Timestamp: Sized + core::fmt::Debug + Clone,
{
    /// Currently reconstructing file transfer streams.
    active: HashMap<(ChannelId, DltFtUInt), (DltFtBuffer, Timestamp)>,

    /// Buffers that have finished receiving data and can be re-used.
    finished: Vec<DltFtBuffer>,
}

impl<ChannelId, Timestamp: Sized> DltFtPool<ChannelId, Timestamp>
where
    ChannelId: Hash + Eq + PartialEq + Clone + Sized,
    Timestamp: core::fmt::Debug + Clone + Sized,
{
    pub fn new() -> DltFtPool<ChannelId, Timestamp> {
        DltFtPool {
            active: HashMap::new(),
            finished: Vec::new(),
        }
    }

    #[inline]
    pub fn active_bufs(&self) -> &HashMap<(ChannelId, DltFtUInt), (DltFtBuffer, Timestamp)> {
        &self.active
    }

    #[inline]
    pub fn finished_bufs(&self) -> &Vec<DltFtBuffer> {
        &self.finished
    }

    pub fn consume<'a: 'c, 'b: 'c, 'c: 'a + 'b>(
        &'a mut self,
        id: ChannelId,
        timestamp: Timestamp,
        pkg: &DltFtPkg,
    ) -> Result<Option<DltFtCompleteInMemFile<'a>>, FtPoolError> {
        use std::collections::hash_map::Entry::*;

        match pkg {
            DltFtPkg::Header(header_pkg) => {
                match self.active.entry((id, header_pkg.file_serial_number)) {
                    Occupied(mut entry) => {
                        let m = entry.get_mut();
                        m.0.reinit_from_header_pkg(header_pkg)?;
                        m.1 = timestamp;

                        // TODO add error
                        Ok(None)
                    },
                    Vacant(vac) => {
                        vac.insert((DltFtBuffer::new(header_pkg)?, timestamp));
                        Ok(None)
                    },
                }
            },
            DltFtPkg::Data(data_pkg) => {
                match self.active.entry((id, data_pkg.file_serial_number)) {
                    Occupied(mut entry) => {

                        // inject the new data & process 
                        let (buffer, last_ts) = entry.get_mut();
                        *last_ts = timestamp;
                        buffer.consume_data_pkg(data_pkg)?;

                        // check if the data is complete
                        if buffer.is_complete() {
                            // take out the buffer
                            let (buf, _) = entry.remove();
                            self.finished.push(buf);
                            Ok(Some(
                                self.finished.last().unwrap().try_finalize().unwrap()
                            ))
                        } else {
                            Ok(None)
                        }
                    },
                    Vacant(_) => {
                        return Err(FtPoolError::DataForUnknownStream { file_serial_number: data_pkg.file_serial_number });
                    },
                }
            },
            DltFtPkg::End(end_pkg) => {
                match self.active.entry((id, end_pkg.file_serial_number)) {
                    Occupied(mut entry) => {

                        // inject the new data & process 
                        let (buffer, last_ts) = entry.get_mut();
                        *last_ts = timestamp;
                        buffer.set_end_received();

                        // check if the data is complete
                        if buffer.is_complete() {
                            // take out the buffer
                            let (buf, _) = entry.remove();
                            self.finished.push(buf);
                            Ok(Some(
                                self.finished.last().unwrap().try_finalize().unwrap()
                            ))
                        } else {
                            Ok(None)
                        }
                    },
                    Vacant(_) => {
                        return Err(FtPoolError::EndForUnknownStream { file_serial_number: end_pkg.file_serial_number });
                    },
                }
            },
            
            DltFtPkg::Error(err) => {
                match self.active.entry((id, err.file_serial_number)) {
                    Occupied(entry) => {
                        // take out the buffer
                        let (buf, _) = entry.remove();
                        self.finished.push(buf);
                        Ok(None)
                    },
                    Vacant(_) => Ok(None),
                }
            },
            DltFtPkg::FileNotExistsError(_) => Ok(None),
            DltFtPkg::Info(_) => Ok(None),
        }
    }

    /// Retains only the elements specified by the predicate.
    pub fn retain<F>(&mut self, f: F)
    where
        F: Fn(&Timestamp) -> bool,
    {
        // check if any entry has to be removed
        if self.active.iter().any(|(_, (_, t))| false == f(t)) {
            self.active = self
                .active
                .drain()
                .filter_map(|(k, v)| {
                    if f(&v.1) {
                        Some((k, v))
                    } else {
                        self.finished.push(v.0);
                        None
                    }
                })
                .collect();
        }
    }
}

impl<ChannelId, Timestamp: Sized> PartialEq for DltFtPool<ChannelId, Timestamp>
where
    ChannelId: Hash + Eq + PartialEq + Clone + Sized,
    Timestamp: core::fmt::Debug + Clone + Sized + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.active == other.active
            && self.finished == other.finished
    }
}

impl<ChannelId, Timestamp: Sized> Eq for DltFtPool<ChannelId, Timestamp>
where
    ChannelId: Hash + Eq + PartialEq + Clone + Sized,
    Timestamp: core::fmt::Debug + Clone + Sized + PartialEq + Eq,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn debug_clone_eq() {
        let pool: DltFtPool<(), ()> = DltFtPool::new(Default::default());
        let _ = format!("{:?}", pool);
        assert_eq!(pool, pool.clone());
        assert_eq!(pool.buf_config(), &TpBufConfig::default());
    }

    #[test]
    fn with_capacity() {
        let pool = DltFtPool::<(), ()>::with_capacity(Default::default(), 3);
        assert_eq!(3, pool.finished_bufs().len());
        assert!(pool.active.capacity() >= 3);
    }

    #[test]
    fn reserve() {
        let mut pool = DltFtPool::<(), ()>::new(Default::default());
        pool.reserve(2);
        assert_eq!(2, pool.finished_bufs().len());
        assert!(pool.active.capacity() >= 2);
        pool.reserve(3);
        assert_eq!(5, pool.finished_bufs().len());
        assert!(pool.active.capacity() >= 5);
    }

    struct TestPacket {
        request_id: u32,
        offset: u32,
        more_segments: bool,
        payload: Vec<u8>,
    }

    impl TestPacket {
        fn new(request_id: u32, offset: u32, more_segments: bool, payload: &[u8]) -> TestPacket {
            TestPacket {
                request_id,
                offset,
                more_segments,
                payload: payload.iter().copied().collect(),
            }
        }

        fn to_vec(&self) -> Vec<u8> {
            let header = SomeipHeader {
                message_id: 1234,
                length: 8 + 4 + self.payload.len() as u32,
                request_id: self.request_id,
                interface_version: 1,
                message_type: MessageType::Notification,
                return_code: 0,
                tp_header: {
                    let mut tp = TpHeader::new(self.more_segments);
                    tp.set_offset(self.offset).unwrap();
                    Some(tp)
                },
            };
            let mut result = Vec::with_capacity(SOMEIP_HEADER_LENGTH + 4 + self.payload.len());
            result.extend_from_slice(&header.base_to_bytes());
            result.extend_from_slice(&header.tp_header.as_ref().unwrap().to_bytes());
            result.extend_from_slice(&self.payload);
            result
        }

        fn result_header(&self, payload_length: u32) -> SomeipHeader {
            SomeipHeader {
                message_id: 1234,
                length: payload_length + 8,
                request_id: self.request_id,
                interface_version: 1,
                message_type: MessageType::Notification,
                return_code: 0,
                tp_header: None,
            }
        }
    }

    /// Returns a u8 vec counting up from "start" until len is reached (truncating bits greater then u8).
    fn sequence(start: usize, len: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(len);
        for i in start..start + len {
            result.push((i & 0xff) as u8);
        }
        result
    }

    #[rustfmt::skip]
    #[test]
    fn consume() {
        use err::TpReassembleError::*;

        // simple packet forwarding (without TP effect)
        {
            // build a non tp packet
            let header = SomeipHeader {
                message_id: 1234,
                length: 8 + 8 as u32,
                request_id: 234,
                interface_version: 1,
                message_type: MessageType::Notification,
                return_code: 0,
                // no tp header
                tp_header: None,
            };
            let mut result = Vec::with_capacity(SOMEIP_HEADER_LENGTH + 8);
            result.extend_from_slice(&header.base_to_bytes());
            result.extend_from_slice(&[0;8]);
            
            let someip_slice = SomeipMsgSlice::from_slice(&result).unwrap();

            let mut pool: DltFtPool<(), ()> = DltFtPool::new(TpBufConfig::new(1024, 2048).unwrap());
            let result = pool.consume((), (), someip_slice.clone()).unwrap();
            assert_eq!(Some(someip_slice), result);
        }

        // normal reconstruction (without additional id)
        {
            let mut pool: DltFtPool<(), ()> = DltFtPool::new(TpBufConfig::new(1024, 2048).unwrap());

            let actions = [
                // start two streams in parallel
                (TestPacket::new(1, 0, true, &sequence(1,16)), None, 1, 0),
                (TestPacket::new(2, 0, true, &sequence(2,32)), None, 2, 0),
                // stream 1 ends
                (TestPacket::new(1, 16, false, &sequence(1 + 16,16)), Some(sequence(1,32)), 1, 1),
                // stream 3 which imidiatly ends
                (TestPacket::new(3, 0, false, &sequence(3,16*4)), Some(sequence(3, 16*4)), 1, 1),
                // end stream 2
                (TestPacket::new(2, 32, true, &sequence(32 + 2,16*4)), None, 1, 1),
                (TestPacket::new(2, 16*6, false, &sequence(16*6 + 2,16*3)), Some(sequence(2, 16*9)), 0, 2),
            ];
            for a in actions {
                let packet = a.0.to_vec();
                let slice = SomeipMsgSlice::from_slice(&packet).unwrap();
                let result = pool.consume((), (), slice).unwrap();
                if let Some(expected_payload) = a.1 {
                    let msg = result.unwrap();
                    assert_eq!(msg.to_header(), a.0.result_header(expected_payload.len() as u32));
                    assert_eq!(msg.payload(), expected_payload);
                } else {
                    assert!(result.is_none());
                }
                assert_eq!(a.2, pool.active_bufs().len());
                assert_eq!(a.3, pool.finished_bufs().len());
            }
        }

        // normal reconstruction (with additional id)
        {
            let mut pool: DltFtPool<u32, ()> = DltFtPool::new(TpBufConfig::new(1024, 2048).unwrap());

            // all actions have the same request id have differing id's
            let actions = [
                // start two streams in parallel
                (123, TestPacket::new(1, 0, true, &sequence(1,16)), None),
                (234, TestPacket::new(1, 0, true, &sequence(2,32)), None),
                // stream 1 ends
                (123, TestPacket::new(1, 16, false, &sequence(1 + 16,16)), Some(sequence(1,32))),
                // stream 3 which imidiatly ends
                (345, TestPacket::new(1, 0, false, &sequence(3,16*4)), Some(sequence(3, 16*4))),
                // end stream 2
                (234, TestPacket::new(1, 32, true, &sequence(32 + 2,16*4)), None),
                (234, TestPacket::new(1, 16*6, false, &sequence(16*6 + 2,16*3)), Some(sequence(2, 16*9))),
            ];
            for a in actions {
                let packet = a.1.to_vec();
                let slice = SomeipMsgSlice::from_slice(&packet).unwrap();
                let result = pool.consume(a.0.clone(), (), slice).unwrap();
                if let Some(expected_payload) = a.2 {
                    let msg = result.unwrap();
                    assert_eq!(msg.to_header(), a.1.result_header(expected_payload.len() as u32));
                    assert_eq!(msg.payload(), expected_payload);
                } else {
                    assert!(result.is_none());
                }
            }
        }

        // error during reconstruction (at start)
        {
            let mut pool: DltFtPool<(), ()> = DltFtPool::new(TpBufConfig::new(1024, 2048).unwrap());

            // should trigger an error as the payload is not a multiple of 1
            let packet = TestPacket::new(1, 0, true, &sequence(1,15)).to_vec();
            let someip_slice = SomeipMsgSlice::from_slice(&packet).unwrap();
            assert_eq!(
                pool.consume((), (), someip_slice).unwrap_err(),
                UnalignedTpPayloadLen { offset: 0, payload_len: 15 }
            );
        }

        // error during reconstruction (after start)
        {
            let mut pool: DltFtPool<(), ()> = DltFtPool::new(TpBufConfig::new(1024, 2048).unwrap());

            {
                let packet = TestPacket::new(1, 0, true, &sequence(1,16)).to_vec();
                let someip_slice = SomeipMsgSlice::from_slice(&packet).unwrap();
                pool.consume((), (), someip_slice).unwrap();
            }

            // should trigger an error as the payload is not a multiple of 1
            let packet = TestPacket::new(1, 16, true, &sequence(1,15)).to_vec();
            let someip_slice = SomeipMsgSlice::from_slice(&packet).unwrap();
            assert_eq!(
                pool.consume((), (), someip_slice).unwrap_err(),
                UnalignedTpPayloadLen { offset: 16, payload_len: 15 }
            );
        }

    }

    #[test]
    fn retain() {
        let mut pool: DltFtPool<u16, u32> = DltFtPool::new(Default::default());
        // request id 1, channel id 2, timestamp 123
        {
            let packet = TestPacket::new(1, 0, true, &sequence(1, 16)).to_vec();
            let slice = SomeipMsgSlice::from_slice(&packet).unwrap();
            let result = pool.consume(2u16, 123u32, slice).unwrap();
            assert!(result.is_none());
            assert_eq!(123, pool.active_bufs().get(&(2u16, 1u32)).unwrap().1);
        }
        // request id 1, channel id 2, timestamp 124
        {
            let packet = TestPacket::new(1, 16, true, &sequence(16, 16)).to_vec();
            let slice = SomeipMsgSlice::from_slice(&packet).unwrap();
            let result = pool.consume(2u16, 124u32, slice).unwrap();
            assert!(result.is_none());
            // check the timestamp was overwritten by the newer packet
            assert_eq!(124, pool.active_bufs().get(&(2u16, 1u32)).unwrap().1);
        }
        // request id 1, channel id 3, timestamp 125
        {
            let packet = TestPacket::new(1, 16, true, &sequence(16, 16)).to_vec();
            let slice = SomeipMsgSlice::from_slice(&packet).unwrap();
            let result = pool.consume(3u16, 125u32, slice).unwrap();
            assert!(result.is_none());
        }

        // discard streams with a timestamp smaller then 125
        pool.retain(|timestamp| *timestamp >= 125);

        assert_eq!(1, pool.active.len());
        assert_eq!(1, pool.finished.len());
        assert_eq!(125, pool.active_bufs().get(&(3u16, 1u32)).unwrap().1);
    }
     */
}