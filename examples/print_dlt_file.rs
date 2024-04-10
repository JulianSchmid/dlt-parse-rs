use std::{
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

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
                LogV(LogVPayload { info, iter }) => {
                    println!(
                        "verbose message of type {:?} with values:",
                        info.into_message_type()
                    );
                    for value in iter {
                        println!("  {:?}", value);
                    }
                }
                LogNv(LogNvPayload {
                    info,
                    msg_id,
                    payload,
                }) => {
                    println!(
                        "non verbose message 0x{:x} of type {:?} and {} bytes of payload without control message.",
                        msg_id,
                        info.map(|v| v.into_message_type()),
                        payload.len(),
                    );
                }
                ControlNv(ControlNvPayload {
                    info,
                    msg_id,
                    payload,
                    control_message,
                }) => {
                    if let Some(mut control_message) = control_message {
                        println!(
                            "non verbose message 0x{:x} of type {:?} and {} bytes of payload.",
                            msg_id,
                            info.map(|v| v.into_message_type()),
                            payload.len(),
                        );
                        print!("With control message: ");
                        if let Err(err) = control_message.write(b"") {
                            println!("The following error occured while trying to write the control message content: {:?}" , err);
                        }
                    }
                }
                _ => {}
            }
        } else {
            println!("non verbose message with incomplete message id");
        }
    }

    Ok(())
}
