#[cfg(feature = "std")]
use crate::{error::*, ft::*};
#[cfg(feature = "std")]
use core::hash::Hash;
#[cfg(feature = "std")]
use std::{collections::HashMap, vec::Vec};

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
#[derive(Debug, Default, Clone)]
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

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
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
                        // Note: Not sure if an error should be returned here as
                        // an other potentially active data stream was discarded.
                        //
                        // Or should the stream even be discarded to begin with?
                        Ok(None)
                    }
                    Vacant(vac) => {
                        vac.insert((
                            if let Some(mut buf) = self.finished.pop() {
                                buf.reinit_from_header_pkg(header_pkg)?;
                                buf
                            } else {
                                DltFtBuffer::new(header_pkg)?
                            },
                            timestamp,
                        ));
                        Ok(None)
                    }
                }
            }
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
                            Ok(Some(self.finished.last().unwrap().try_finalize().unwrap()))
                        } else {
                            Ok(None)
                        }
                    }
                    Vacant(_) => Err(FtPoolError::DataForUnknownStream {
                        file_serial_number: data_pkg.file_serial_number,
                    }),
                }
            }
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
                            Ok(Some(self.finished.last().unwrap().try_finalize().unwrap()))
                        } else {
                            Ok(None)
                        }
                    }
                    Vacant(_) => Err(FtPoolError::EndForUnknownStream {
                        file_serial_number: end_pkg.file_serial_number,
                    }),
                }
            }

            DltFtPkg::Error(err) => {
                match self.active.entry((id, err.file_serial_number)) {
                    Occupied(entry) => {
                        // take out the buffer
                        let (buf, _) = entry.remove();
                        self.finished.push(buf);
                        Ok(None)
                    }
                    Vacant(_) => Ok(None),
                }
            }
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

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<ChannelId, Timestamp: Sized> PartialEq for DltFtPool<ChannelId, Timestamp>
where
    ChannelId: Hash + Eq + PartialEq + Clone + Sized,
    Timestamp: core::fmt::Debug + Clone + Sized + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.active == other.active && self.finished == other.finished
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<ChannelId, Timestamp: Sized> Eq for DltFtPool<ChannelId, Timestamp>
where
    ChannelId: Hash + Eq + PartialEq + Clone + Sized,
    Timestamp: core::fmt::Debug + Clone + Sized + PartialEq + Eq,
{
}

#[cfg(all(feature = "std", test))]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn debug_clone_eq() {
        let pool: DltFtPool<(), ()> = Default::default();
        let _ = format!("{:?}", pool);
        assert_eq!(pool, pool.clone());
    }

    #[test]
    fn consume() {
        let get_data = |from: u64, to: u64| -> Vec<u8> {
            let mut result = Vec::with_capacity((to - from) as usize);
            for i in from..to {
                result.push(i as u8);
            }
            result
        };

        // normal package reconstruction
        {
            let mut pool = DltFtPool::<u8, u32>::new();
            {
                assert_eq!(
                    pool.consume(
                        1,
                        2,
                        &DltFtPkg::Header(DltFtHeaderPkg {
                            file_serial_number: DltFtUInt::U32(123),
                            file_name: "a.txt",
                            file_size: DltFtUInt::U32(20 * 3),
                            creation_date: "2024-06-28",
                            number_of_packages: DltFtUInt::U32(3),
                            buffer_size: DltFtUInt::U32(20),
                        })
                    ),
                    Ok(None)
                );
                assert_eq!(
                    pool.consume(
                        1,
                        3,
                        &DltFtPkg::Data(DltFtDataPkg {
                            file_serial_number: DltFtUInt::U32(123),
                            package_nr: DltFtUInt::U32(1),
                            data: &get_data(0, 20),
                        })
                    ),
                    Ok(None)
                );
                assert_eq!(
                    pool.consume(
                        1,
                        4,
                        &DltFtPkg::Data(DltFtDataPkg {
                            file_serial_number: DltFtUInt::U32(123),
                            package_nr: DltFtUInt::U32(2),
                            data: &get_data(20, 40),
                        })
                    ),
                    Ok(None)
                );
                assert_eq!(
                    pool.consume(
                        1,
                        5,
                        &DltFtPkg::Data(DltFtDataPkg {
                            file_serial_number: DltFtUInt::U32(123),
                            package_nr: DltFtUInt::U32(3),
                            data: &get_data(40, 60),
                        })
                    ),
                    Ok(None)
                );
                assert_eq!(
                    pool.consume(
                        1,
                        5,
                        &DltFtPkg::End(DltFtEndPkg {
                            file_serial_number: DltFtUInt::U32(123)
                        })
                    ),
                    Ok(Some(DltFtCompleteInMemFile {
                        file_serial_number: DltFtUInt::U32(123),
                        file_name: "a.txt",
                        creation_date: "2024-06-28",
                        data: &get_data(0, 60)
                    }))
                );
            }

            // reconstruction with end in data (twice to check that buffer re-using works correctly)
            for _ in 0..2 {
                assert!(pool.active.is_empty());
                assert_eq!(1, pool.finished.len());
                assert_eq!(
                    pool.consume(
                        1,
                        2,
                        &DltFtPkg::Header(DltFtHeaderPkg {
                            file_serial_number: DltFtUInt::U32(123),
                            file_name: "a.txt",
                            file_size: DltFtUInt::U32(20 * 3),
                            creation_date: "2024-06-28",
                            number_of_packages: DltFtUInt::U32(3),
                            buffer_size: DltFtUInt::U32(20),
                        })
                    ),
                    Ok(None)
                );
                assert_eq!(
                    pool.consume(
                        1,
                        3,
                        &DltFtPkg::Data(DltFtDataPkg {
                            file_serial_number: DltFtUInt::U32(123),
                            package_nr: DltFtUInt::U32(1),
                            data: &get_data(0, 20),
                        })
                    ),
                    Ok(None)
                );
                assert_eq!(
                    pool.consume(
                        1,
                        4,
                        &DltFtPkg::Data(DltFtDataPkg {
                            file_serial_number: DltFtUInt::U32(123),
                            package_nr: DltFtUInt::U32(2),
                            data: &get_data(20, 40),
                        })
                    ),
                    Ok(None)
                );
                assert_eq!(
                    pool.consume(
                        1,
                        5,
                        &DltFtPkg::End(DltFtEndPkg {
                            file_serial_number: DltFtUInt::U32(123)
                        })
                    ),
                    Ok(None)
                );
                assert_eq!(
                    pool.consume(
                        1,
                        5,
                        &DltFtPkg::Data(DltFtDataPkg {
                            file_serial_number: DltFtUInt::U32(123),
                            package_nr: DltFtUInt::U32(3),
                            data: &get_data(40, 60),
                        })
                    ),
                    Ok(Some(DltFtCompleteInMemFile {
                        file_serial_number: DltFtUInt::U32(123),
                        file_name: "a.txt",
                        creation_date: "2024-06-28",
                        data: &get_data(0, 60)
                    }))
                );
            }
        }

        // package reconstruction with a stream overwrite even though not finished
        {
            let base = {
                let mut pool = DltFtPool::<u8, u32>::new();
                assert_eq!(
                    pool.consume(
                        1,
                        2,
                        &DltFtPkg::Header(DltFtHeaderPkg {
                            file_serial_number: DltFtUInt::U32(123),
                            file_name: "a.txt",
                            file_size: DltFtUInt::U32(20 * 3),
                            creation_date: "2024-06-28",
                            number_of_packages: DltFtUInt::U32(3),
                            buffer_size: DltFtUInt::U32(20),
                        })
                    ),
                    Ok(None)
                );
                assert_eq!(
                    pool.active.get(&(1, DltFtUInt::U32(123))).unwrap().0.file_name(),
                    "a.txt"
                );
                pool
            };

            // ok case
            {
                let mut pool = base.clone();
                assert_eq!(
                    pool.consume(
                        1,
                        2,
                        &DltFtPkg::Header(DltFtHeaderPkg {
                            file_serial_number: DltFtUInt::U32(123), // same fid
                            file_name: "b.txt", // different name
                            file_size: DltFtUInt::U32(20 * 3),
                            creation_date: "2024-06-28",
                            number_of_packages: DltFtUInt::U32(3),
                            buffer_size: DltFtUInt::U32(20),
                        })
                    ),
                    Ok(None)
                );
                assert_eq!(
                    pool.active.get(&(1, DltFtUInt::U32(123))).unwrap().0.file_name(),
                    "b.txt"
                );
            }

            // error case
            {
                let mut pool = base.clone();
                pool.consume(
                    1,
                    2,
                    &DltFtPkg::Header(DltFtHeaderPkg {
                        file_serial_number: DltFtUInt::U32(123), // same fid
                        file_name: "b.txt",
                        file_size: DltFtUInt::U32(20 * 3),
                        creation_date: "2024-06-28",
                        number_of_packages: DltFtUInt::U32(2), // bad num of package
                        buffer_size: DltFtUInt::U32(20),
                    })
                ).unwrap_err();
                assert_eq!(
                    pool.active.get(&(1, DltFtUInt::U32(123))).unwrap().0.file_name(),
                    "a.txt"
                );
            }
        }

        // error in new stream (no buffer)
        {
            let mut pool = DltFtPool::<u8, u32>::new();
            pool.consume(
                1,
                2,
                &DltFtPkg::Header(DltFtHeaderPkg {
                    file_serial_number: DltFtUInt::U32(123),
                    file_name: "a.txt",
                    file_size: DltFtUInt::U32(20 * 3),
                    creation_date: "2024-06-28",
                    number_of_packages: DltFtUInt::U32(2), // bad number of packages
                    buffer_size: DltFtUInt::U32(20),
                })
            ).unwrap_err();
        }

        // error in new stream (with buffer)
        {
            let mut pool = DltFtPool::<u8, u32>::new();

            // start and end a stream
            pool.consume(
                1,
                2,
                &DltFtPkg::Header(DltFtHeaderPkg {
                    file_serial_number: DltFtUInt::U32(123),
                    file_name: "a.txt",
                    file_size: DltFtUInt::U32(0),
                    creation_date: "2024-06-28",
                    number_of_packages: DltFtUInt::U32(0),
                    buffer_size: DltFtUInt::U32(20),
                })
            ).unwrap();
            pool.consume(
                1,
                2,
                &DltFtPkg::End(DltFtEndPkg {
                    file_serial_number: DltFtUInt::U32(123),
                })
            ).unwrap();

            assert!(pool.active_bufs().is_empty());
            assert_eq!(1, pool.finished_bufs().len());

            // trigger an error
            pool.consume(
                1,
                2,
                &DltFtPkg::Header(DltFtHeaderPkg {
                    file_serial_number: DltFtUInt::U32(123),
                    file_name: "a.txt",
                    file_size: DltFtUInt::U32(20 * 3),
                    creation_date: "2024-06-28",
                    number_of_packages: DltFtUInt::U32(2), // bad number of packages
                    buffer_size: DltFtUInt::U32(20),
                })
            ).unwrap_err();
        }

        // error in data package
        {
            let mut pool = DltFtPool::<u8, u32>::new();

            // start and end a stream
            pool.consume(
                1,
                2,
                &DltFtPkg::Header(DltFtHeaderPkg {
                    file_serial_number: DltFtUInt::U32(123),
                    file_name: "a.txt",
                    file_size: DltFtUInt::U32(20),
                    creation_date: "2024-06-28",
                    number_of_packages: DltFtUInt::U32(1),
                    buffer_size: DltFtUInt::U32(20),
                })
            ).unwrap();
            pool.consume(
                1,
                2,
                &DltFtPkg::Data(DltFtDataPkg {
                    file_serial_number: DltFtUInt::U32(123),
                    package_nr: DltFtUInt::U32(2), // bad package number
                    data: &[],
                })
            ).unwrap_err();
        }

        // error unknown data package stream
        {
            let mut pool = DltFtPool::<u8, u32>::new();

            // start and end a stream
            pool.consume(
                1,
                2,
                &DltFtPkg::Header(DltFtHeaderPkg {
                    file_serial_number: DltFtUInt::U32(123),
                    file_name: "a.txt",
                    file_size: DltFtUInt::U32(20),
                    creation_date: "2024-06-28",
                    number_of_packages: DltFtUInt::U32(1),
                    buffer_size: DltFtUInt::U32(20),
                })
            ).unwrap();
            assert_eq!(
                    pool.consume(
                    1,
                    2,
                    &DltFtPkg::Data(DltFtDataPkg {
                        file_serial_number: DltFtUInt::U32(234), // unknown data stream
                        package_nr: DltFtUInt::U32(2), 
                        data: &[],
                    })
                ),
                Err(FtPoolError::DataForUnknownStream { file_serial_number: DltFtUInt::U32(234) })
            );
        }

        // end unknown data stream error
        {
            let mut pool = DltFtPool::<u8, u32>::new();
            assert_eq!(
                    pool.consume(
                    1,
                    2,
                    &DltFtPkg::End(DltFtEndPkg {
                        file_serial_number: DltFtUInt::U32(234), // unknown data stream
                    })
                ),
                Err(FtPoolError::EndForUnknownStream { file_serial_number: DltFtUInt::U32(234) })
            );
        }

        // error package for unknown stream
        {
            let mut pool = DltFtPool::<u8, u32>::new();
            assert_eq!(
                    pool.consume(
                    1,
                    2,
                    &DltFtPkg::Error(DltFtErrorPkg {
                        file_serial_number:DltFtUInt::U32(234),
                        error_code: DltFtErrorCode(DltFtInt::I32(123)),
                        linux_error_code: DltFtInt::I32(123),
                        file_name: "a.txt",
                        file_size: DltFtUInt::U32(123),
                        creation_date: "0-0-0",
                        number_of_packages: DltFtUInt::U32(1)
                    })
                ),
                Ok(None)
            );
        }

        // error package for known stream
        {
            let mut pool = DltFtPool::<u8, u32>::new();
            pool.consume(
                1,
                2,
                &DltFtPkg::Header(DltFtHeaderPkg {
                    file_serial_number: DltFtUInt::U32(123),
                    file_name: "a.txt",
                    file_size: DltFtUInt::U32(20),
                    creation_date: "2024-06-28",
                    number_of_packages: DltFtUInt::U32(1),
                    buffer_size: DltFtUInt::U32(20),
                })
            ).unwrap();
            assert_eq!(
                    pool.consume(
                    1,
                    2,
                    &DltFtPkg::Error(DltFtErrorPkg {
                        file_serial_number:DltFtUInt::U32(123),
                        error_code: DltFtErrorCode(DltFtInt::I32(123)),
                        linux_error_code: DltFtInt::I32(123),
                        file_name: "a.txt",
                        file_size: DltFtUInt::U32(123),
                        creation_date: "0-0-0",
                        number_of_packages: DltFtUInt::U32(1)
                    })
                ),
                Ok(None)
            );
            assert!(pool.active.is_empty());
            assert_eq!(pool.finished.len(), 1);
        }

        // file not exist error package
        {
            let mut pool = DltFtPool::<u8, u32>::new();
            assert_eq!(
                    pool.consume(
                    1,
                    2,
                    &DltFtPkg::FileNotExistsError(DltFtFileNotExistErrorPkg {
                        error_code: DltFtErrorCode(DltFtInt::I32(123)),
                        linux_error_code: DltFtInt::I32(123),
                        file_name: "a.txt",
                    })
                ),
                Ok(None)
            );
            assert!(pool.active.is_empty());
            assert!(pool.finished.is_empty());
        }

        // file info package
        {
            let mut pool = DltFtPool::<u8, u32>::new();
            assert_eq!(
                    pool.consume(
                    1,
                    2,
                    &DltFtPkg::Info(DltFtInfoPkg {
                        file_serial_number: DltFtUInt::U32(123),
                        file_name: "a.txt",
                        file_size: DltFtUInt::U32(20),
                        creation_date: "2024-06-28",
                        number_of_packages: DltFtUInt::U32(1),
                    })
                ),
                Ok(None)
            );
            assert!(pool.active.is_empty());
            assert!(pool.finished.is_empty());
        }

    }

    #[test]
    fn retain() {
        let mut pool = DltFtPool::<u8, u32>::new();

        // start two streams
        assert_eq!(
            pool.consume(
                1,
                2,
                &DltFtPkg::Header(DltFtHeaderPkg {
                    file_serial_number: DltFtUInt::U32(123),
                    file_name: "a.txt",
                    file_size: DltFtUInt::U32(20 * 3),
                    creation_date: "2024-06-28",
                    number_of_packages: DltFtUInt::U32(3),
                    buffer_size: DltFtUInt::U32(20),
                })
            ),
            Ok(None)
        );
        assert_eq!(
            pool.consume(
                1,
                3,
                &DltFtPkg::Header(DltFtHeaderPkg {
                    file_serial_number: DltFtUInt::U32(234),
                    file_name: "b.txt",
                    file_size: DltFtUInt::U32(20 * 3),
                    creation_date: "2024-06-29",
                    number_of_packages: DltFtUInt::U32(3),
                    buffer_size: DltFtUInt::U32(20),
                })
            ),
            Ok(None)
        );

        // retain the second stream only
        pool.retain(|t| *t > 2);

        assert_eq!(pool.active.len(), 1);
        assert_eq!(pool.finished.len(), 1);
        assert!(pool.active.get(&(1, DltFtUInt::U32(234))).is_some());
    }

}
