use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::UNIX_EPOCH,
};

use dlt_parse::{
    storage::{DltStorageWriter, StorageHeader},
    SliceIterator,
};
use etherparse::{SlicedPacket, TransportSlice::Udp};
use rpcap::read::PcapReader;
use structopt::StructOpt;

/// Expected command line arguments
#[derive(StructOpt, Debug)]
#[structopt(name = "pcap2dlt")]
struct CommandLineArguments {
    /// Udp port on which dlt packets are send.
    #[structopt(short("u"), long("udp-dest-port"))]
    udp_dest_port: u16,

    /// Path to pcap file.
    #[structopt(short("o"), long("output-file"), parse(from_os_str))]
    output_file: PathBuf,

    /// Path to pcap file.
    #[structopt(parse(from_os_str))]
    pcap_file: PathBuf,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    PcapError(rpcap::PcapError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<rpcap::PcapError> for Error {
    fn from(err: rpcap::PcapError) -> Error {
        Error::PcapError(err)
    }
}

fn main() -> Result<(), Error> {
    let args = CommandLineArguments::from_args();

    let dlt_file = File::create(args.output_file)?;
    let mut dlt_writer = DltStorageWriter::new(BufWriter::new(dlt_file));

    let pcap_file = File::open(args.pcap_file)?;
    let (_, mut reader) = PcapReader::new(BufReader::new(pcap_file))?;

    while let Some(packet) = reader.next()? {
        // decode from ethernet to udp layer
        let sliced = match SlicedPacket::from_ethernet(packet.data) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("Error parsing packet: {}", err);
                continue;
            }
        };

        // verify the packet is an udp packet with the correct destination port
        if let Some(Udp(udp)) = sliced.transport {
            if udp.destination_port() != args.udp_dest_port {
                // skip packet if the port is not matching
                continue;
            }
        } else {
            // skip packet if not relevant
            continue;
        }

        // iterate over the dlt messages in the packet
        for dlt_packet in SliceIterator::new(sliced.payload) {
            let dlt_packet = match dlt_packet {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("Error parsing dlt: {}", err);
                    break;
                }
            };

            // determine ecu id
            let ecu_id = if let Some(ecu_id) = dlt_packet.header().ecu_id {
                ecu_id
            } else {
                // you might want to determine the ecu id via the ip here
                // if you have that option
                [0, 0, 0, 0]
            };

            // determine utc time (unwrap is ok, as all pcap timestamps start at UNIX_EPOCH)
            let d = packet.time.duration_since(UNIX_EPOCH).unwrap();

            // write the packet
            dlt_writer.write_slice(
                StorageHeader {
                    timestamp_seconds: d.as_secs() as u32,
                    timestamp_microseconds: d.subsec_micros(),
                    ecu_id,
                },
                dlt_packet,
            )?;
        }
    }

    Ok(())
}
