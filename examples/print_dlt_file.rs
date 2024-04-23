use std::{fs::File, io::BufReader, path::PathBuf};

use dlt_parse::{
    error::ReadError, storage::DltStorageReader, ControlNvPayload, LogNvPayload, LogVPayload,
};
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

        if let Some(typed_payload) = msg.packet.typed_payload() {
            use dlt_parse::DltTypedPayload::*;
            match typed_payload {
                LogV(LogVPayload { iter, log_level }) => {
                    println!(
                        "verbose log message with log level {:?} and values:",
                        log_level
                    );
                    for value in iter {
                        println!("  {:?}", value);
                    }
                }
                LogNv(LogNvPayload {
                    log_level,
                    msg_id,
                    payload,
                }) => {
                    println!(
                        "non verbose log message 0x{:x} with log level {:?} and {} bytes of payload.",
                        msg_id,
                        log_level,
                        payload.len(),
                    );
                }
                ControlNv(ControlNvPayload {
                    msg_type,
                    service_id,
                    payload,
                }) => {
                    println!(
                        "non verbose control message {:?} with service id: {} and {} bytes of payload.",
                        msg_type,
                        service_id,
                        payload.len(),
                    );
                }
                UnknownNv(_) => println!("generic non verbose message received"),
                TraceV(_) => println!("verbose trace message received"),
                TraceNv(_) => println!("non verbose trace message received"),
                NetworkV(_) => println!("verbose network message received"),
                NetworkNv(_) => println!("non verbose network message received"),
                ControlV(_) => println!("verbose control message received"),
            }
        } else {
            println!("non verbose message with incomplete message id");
        }
    }

    Ok(())
}
