use dlt_parse::{error::ReadError, storage::DltStorageReader};
use std::{fs::File, io::BufReader, path::PathBuf};
use structopt::StructOpt;

/// Expected command line arguments
#[derive(StructOpt, Debug)]
#[structopt(name = "print_dlt_file")]
struct CommandLineArguments {
    /// Path to pcap file.
    #[structopt(parse(from_os_str))]
    dlt_file: PathBuf,
}

fn main() -> Result<(), ReadError> {
    let args = CommandLineArguments::from_args();

    let dlt_file = File::open(args.dlt_file)?;
    let mut reader = DltStorageReader::new(BufReader::new(dlt_file));

    while let Some(msg) = reader.next_packet() {
        let msg = msg?;

        println!("{:?}", msg.storage_header);

        if let Some(extended_header) = msg.packet.extended_header() {
            use core::str::from_utf8;

            println!(
                "application_id: {:?}, context_id: {:?}",
                from_utf8(&extended_header.application_id),
                from_utf8(&extended_header.context_id)
            );
        }

        // check what type of message was received
        match msg.packet.typed_payload() {
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
                        println!(
                            "non verbose control message {:?} with service id: {} and {} bytes of payload.",
                            p.msg_type,
                            p.service_id,
                            p.payload.len(),
                        );
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

    Ok(())
}
