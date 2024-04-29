use std::path::PathBuf;
use structopt::StructOpt;

use self::etherparse::*;
use etherparse;

use rpcap::read::PcapReader;

use std::fs::File;
use std::io::BufReader;

use dlt_parse::*;

/// Expected command line arguments
#[derive(StructOpt, Debug)]
#[structopt(name = "print_messages_ids")]
struct CommandLineArguments {
    /// Udp port on which dlt packets are send.
    #[structopt(short, long)]
    udp_port: u16,

    /// Path to pcap file.
    #[structopt(parse(from_os_str))]
    pcap_file: PathBuf,
}

fn main() -> Result<(), Error> {
    read(CommandLineArguments::from_args())
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

fn read(arguments: CommandLineArguments) -> Result<(), Error> {
    let (_, mut reader) = PcapReader::new(BufReader::new(File::open(arguments.pcap_file)?))?;

    while let Some(packet) = reader.next()? {
        let sliced = SlicedPacket::from_ethernet(packet.data);

        //only use the packet if the parsing from ethernet layer to transport layer was error free
        if let Ok(sliced_packet) = sliced {
            use crate::TransportSlice::*;

            //check that it is an udp packet
            if let Some(Udp(udp_slice)) = sliced_packet.transport {
                //check the port
                if udp_slice.destination_port() != arguments.udp_port {
                    // skip packet if the port is not matching
                    continue;
                }
            } else {
                // skip packet if it is not an udp packet
                continue;
            }

            // parse the dlt message in the udp payload
            for dlt_slice in SliceIterator::new(sliced_packet.payload) {
                let dlt_slice = match dlt_slice {
                    Ok(dlt_slice) => dlt_slice,
                    Err(err) => {
                        // error parsing the dlt packet
                        println!("ERROR: {:?}", err);
                        break;
                    }
                };

                // print application id & context id if available
                if let Some(extended_header) = dlt_slice.extended_header() {
                    use core::str::from_utf8;

                    println!(
                        "application_id: {:?}, context_id: {:?}",
                        from_utf8(&extended_header.application_id),
                        from_utf8(&extended_header.context_id)
                    );
                }

                // check what type of message was received
                match dlt_slice.typed_payload() {
                    Ok(typed_payload) => {
                        use dlt_parse::DltTypedPayload::*;
                        match typed_payload {
                            UnknownNv(p) => {
                                println!(
                                    "non verbose message 0x{:x} (unknown) with {} bytes of payload.",
                                    p.msg_id,
                                    p.payload.len(),
                                );
                            }
                            LogNv(p) => {
                                println!(
                                    "non verbose log message 0x{:x} with log level {:?} and {} bytes of payload.",
                                    p.msg_id,
                                    p.log_level,
                                    p.payload.len(),
                                );
                            }
                            LogV(p) => {
                                println!(
                                    "verbose log message with log level {:?} and values:",
                                    p.log_level
                                );
                                for value in p.iter {
                                    println!("  {:?}", value);
                                }
                            }
                            TraceNv(p) => {
                                println!(
                                    "non verbose trace message 0x{:x} of type {:?} and {} bytes of payload.",
                                    p.msg_id,
                                    p.trace_type,
                                    p.payload.len(),
                                );
                            }
                            TraceV(p) => {
                                println!(
                                    "verbose trace message with of type {:?} and values:",
                                    p.trace_type
                                );
                                for value in p.iter {
                                    println!("  {:?}", value);
                                }
                            }
                            NetworkNv(p) => {
                                println!(
                                    "non verbose network message 0x{:x} of type {:?} and {} bytes of payload.",
                                    p.msg_id,
                                    p.net_type,
                                    p.payload.len(),
                                );
                            }
                            NetworkV(p) => {
                                println!(
                                    "verbose network message with of type {:?} and values:",
                                    p.net_type
                                );
                                for value in p.iter {
                                    println!("  {:?}", value);
                                }
                            }
                            ControlNv(p) => {
                                println!("non verbose control message {:?} with service id: {} and {} bytes of payload.", p.msg_type, p.service_id, p.payload.len());
                            }
                            ControlV(p) => {
                                println!("verbose control message {:?} with values:", p.msg_type);
                                for value in p.iter {
                                    println!("  {:?}", value);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!("message with payload error received: {}", err);
                    }
                }
            }
        }
    }
    Ok(())
}
