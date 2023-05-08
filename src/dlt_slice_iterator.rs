use super::*;

/// Allows iterating over the someip message in a udp or tcp payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliceIterator<'a> {
    slice: &'a [u8],
}

impl<'a> SliceIterator<'a> {
    #[inline]
    pub fn new(slice: &'a [u8]) -> SliceIterator<'a> {
        SliceIterator { slice }
    }

    /// Returns the slice of data still left in the iterator.
    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }
}

impl<'a> Iterator for SliceIterator<'a> {
    type Item = Result<DltPacketSlice<'a>, error::PacketSliceError>;

    #[inline]
    fn next(&mut self) -> Option<Result<DltPacketSlice<'a>, error::PacketSliceError>> {
        if !self.slice.is_empty() {
            //parse
            let result = DltPacketSlice::from_slice(self.slice);

            //move the slice depending on the result
            match &result {
                Err(_) => {
                    //error => move the slice to an len = 0 position so that the iterator ends
                    let len = self.slice.len();
                    self.slice = &self.slice[len..];
                }
                Ok(ref value) => {
                    //by the length just taken by the slice
                    self.slice = &self.slice[value.slice().len()..];
                }
            }

            //return parse result
            Some(result)
        } else {
            None
        }
    }
}

/// Tests for `SliceIterator`
#[cfg(test)]
mod slice_interator_tests {

    use super::*;
    use crate::proptest_generators::*;
    use proptest::prelude::*;

    #[test]
    fn clone_eq() {
        let it = SliceIterator { slice: &[] };
        assert_eq!(it, it.clone());
    }

    #[test]
    fn debug() {
        let it = SliceIterator { slice: &[] };
        assert_eq!(
            format!("SliceIterator {{ slice: {:?} }}", it.slice),
            format!("{:?}", it)
        );
    }

    #[test]
    fn slice() {
        let buffer: [u8;4] = [1, 2, 3, 4];
        let it = SliceIterator { slice: &buffer };
        assert_eq!(it.slice(), &buffer);
    }

    proptest! {
        #[test]
        fn iterator(ref packets in prop::collection::vec(dlt_header_with_payload_any(), 1..5)) {
            use error::PacketSliceError::*;

            //serialize the packets
            let mut buffer = Vec::with_capacity(
                (*packets).iter().fold(0, |acc, x| acc + usize::from(x.0.header_len()) + x.1.len())
            );

            let mut offsets: Vec<(usize, usize)> = Vec::with_capacity(packets.len());

            for packet in packets {

                //save the start for later processing
                let start = buffer.len();

                //header & payload
                buffer.extend_from_slice(&packet.0.to_bytes());
                buffer.extend_from_slice(&packet.1);

                //safe the offset for later
                offsets.push((start, buffer.len()));
            }

            //determine the expected output
            let mut expected: Vec<DltPacketSlice<'_>> = Vec::with_capacity(packets.len());
            for offset in &offsets {
                //create the expected slice
                let slice = &buffer[offset.0..offset.1];
                let e = DltPacketSlice::from_slice(slice).unwrap();
                assert_eq!(e.slice(), slice);
                expected.push(e);
            }

            //iterate over packets
            assert_eq!(expected, SliceIterator::new(&buffer).map(|x| x.unwrap()).collect::<Vec<DltPacketSlice<'_>>>());

            //check for error return when the slice is too small
            //first entry
            {
                let o = offsets.first().unwrap();
                let mut it = SliceIterator::new(&buffer[..(o.1 - 1)]);

                assert_matches!(it.next(), Some(Err(UnexpectedEndOfSlice(_))));
                //check that the iterator does not continue
                assert_matches!(it.next(), None);
            }
            //last entry
            {
                let o = offsets.last().unwrap();
                let it = SliceIterator::new(&buffer[..(o.1 - 1)]);
                let mut it = it.skip(offsets.len()-1);

                assert_matches!(it.next(), Some(Err(UnexpectedEndOfSlice(_))));
                //check that the iterator does not continue
                assert_matches!(it.next(), None);
            }
        }
    }
} // mod slice_iterator_tests
