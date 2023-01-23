use std::{fs::File, io::BufReader, path::PathBuf};

use dlt_parse::{error::ReadError, storage::DltStorageReader};
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

        if let Some((message_id, non_verbose_payload)) = msg.packet.message_id_and_payload() {
            if let Some(extended_header) = msg.packet.extended_header() {
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
            println!("  with payload {:?}", non_verbose_payload);
        } else {
            println!("verbose message (parsing not yet supported)");
        }
    }

    Ok(())
}
