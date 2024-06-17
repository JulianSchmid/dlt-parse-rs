use super::*;

/// DLT file transfer package.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DltFtPkg<'a, 'b> {
    /// Packet sent at the start of a file transfer.
    Header(DltFtHeaderPkg<'a, 'b>),
    /// Package containing a chunk of data of a file.
    Data(DltFtDataPkg<'a>),
    /// Package sent after a file transfer is complete.
    End(DltFtEndPkg),
    /// Info packet for a file if only metadat is sent.
    Info(DltFtInfoPkg<'a, 'b>),
    /// Error package sent when an error occured with an
    /// existing file.
    Error(DltFtErrorPkg<'a, 'b>),
    /// Error package sent if a file that should have been
    /// transfered does not exists.
    FileNotExistsError(DltFtFileNotExistErrorPkg<'a>),
}
