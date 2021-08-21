use structopt;

use std::path::PathBuf;
use structopt::StructOpt;

use etherparse;
use self::etherparse::*;

use rpcap;
use self::rpcap::read::PcapReader;

use std::io::BufReader;
use std::fs::File;


use dlt_parse::*;

/// Expected command line arguments
#[derive(StructOpt, Debug)]
#[structopt(name = "print_messages_ids")]
struct CommandLineArguments {

    /// Udp port on which dlt packets are send.
    udp_port: u16,

    /// Path to pcap file.
    pcap_file: PathBuf,
}

fn main() -> Result<(), Error> {
    read(CommandLineArguments::from_args())
}

#[derive(Debug)]
enum Error {
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

fn read(arguments: CommandLineArguments) -> Result<(),Error> {
    let mut reader = PcapReader::new(BufReader::new(File::open(arguments.pcap_file)?))?;

    while let Some(packet) = reader.next()? {
        let sliced = SlicedPacket::from_ethernet(&packet.data);

        //only use the packet if the parsing from ethernet layer to transport layer was error free
        if let Ok(sliced_packet) = sliced {
            use crate::TransportSlice::*;

            //check that it is an udp packet
            if let Some(Udp(udp_slice)) = sliced_packet.transport {

                //check the port
                if udp_slice.destination_port() == arguments.udp_port {

                    //trying parsing dlt messages located in a udp payload
                    for dlt_message in SliceIterator::new(sliced_packet.payload) {
                        match dlt_message {
                            Ok(dlt_slice) => {
                                //check if the message is verbose or non verbose (non verbose messages have message ids)
                                if let Some(message_id) = dlt_slice.message_id() {
                                    if let Some(extended_header) = dlt_slice.extended_header() {
                                        use core::str::from_utf8;

                                        if let Some(message_type) = extended_header.message_type() {
                                            println!(
                                                "non verbose message 0x{:x} (type: {:?}, application_id: {:?}, context_id: {:?})", 
                                                message_id,
                                                message_type,
                                                from_utf8(&extended_header.application_id),
                                                from_utf8(&extended_header.context_id)
                                            );
                                        } else {
                                            println!(
                                                "non verbose message 0x{:x} (application_id: {:?}, context_id: {:?})",
                                                message_id,
                                                from_utf8(&extended_header.application_id),
                                                from_utf8(&extended_header.context_id)
                                            );
                                        }
                                    } else {
                                        println!("non verbose message 0x{:x}", message_id);
                                    }
                                    println!("  with payload {:?}", dlt_slice.non_verbose_payload());
                                } else {
                                    println!("verbose message (parsing not yet supported)");
                                }
                            },
                            Err(err) => {
                                //error parsing the dlt packet
                                println!("ERROR: {:?}", err);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}