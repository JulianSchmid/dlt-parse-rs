use std::{fs::File, io::BufReader, path::PathBuf};

use dlt_parse::{
    error::{ReadError, VerboseDecodeError},
    storage::DltStorageReader,
    verbose::VerboseIter,
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
                Verbose { info, iter } => {
                    println!(
                        "verbose message of type {:?} with values:",
                        info.into_message_type()
                    );
                    let field_print_result = print_fields(iter, 1);
                    if let Err(err) = field_print_result {
                        println!("  ERROR decoding value: {}", err);
                    }
                }
                NonVerbose {
                    info: _,
                    msg_id: _,
                    payload: _,
                } => {}
            }
        } else {
            println!("non verbose message with incomplete message id");
        }
    }

    Ok(())
}

fn print_fields(iter: VerboseIter, indent: usize) -> Result<(), VerboseDecodeError> {
    let print_indent = || {
        for _ in 0..indent {
            print!("  ");
        }
    };

    for value in iter {
        let value = value?;
        // print name
        if let Some(name) = value.name() {
            print_indent();
            println!("name = '{}'", name);
        }
        if let Some(unit) = value.unit() {
            print_indent();
            println!("unit = '{}'", unit);
        }

        // print value
        use dlt_parse::verbose::VerboseValue::*;

        print_indent();

        match value {
            Bool(v) => println!("value = {}", v.value),
            Str(v) => println!("value = {}", v.value),
            TraceInfo(v) => println!("value = {}", v.value),
            I8(v) => println!("value = {}", v.value),
            I16(v) => println!("value = {}", v.value),
            I32(v) => println!("value = {}", v.value),
            I64(v) => println!("value = {}", v.value),
            I128(v) => println!("value = {}", v.value),
            U8(v) => println!("value = {}", v.value),
            U16(v) => println!("value = {}", v.value),
            U32(v) => println!("value = {}", v.value),
            U64(v) => println!("value = {}", v.value),
            U128(v) => println!("value = {}", v.value),
            F16(v) => println!("value = {}", v.value.to_f32()),
            F32(v) => println!("value = {}", v.value),
            F64(v) => println!("value = {}", v.value),
            F128(v) => println!("value = F128Bits({})", v.value.to_bits()),
            ArrBool(v) => print_arr(v.iter()),
            ArrI8(v) => print_arr(v.iter()),
            ArrI16(v) => print_arr(v.iter()),
            ArrI32(v) => print_arr(v.iter()),
            ArrI64(v) => print_arr(v.iter()),
            ArrI128(v) => print_arr(v.iter()),
            ArrU8(v) => print_arr(v.iter()),
            ArrU16(v) => print_arr(v.iter()),
            ArrU32(v) => print_arr(v.iter()),
            ArrU64(v) => print_arr(v.iter()),
            ArrU128(v) => print_arr(v.iter()),
            ArrF16(v) => print_arr(v.iter().map(|v| v.to_f32())),
            ArrF32(v) => print_arr(v.iter()),
            ArrF64(v) => print_arr(v.iter()),
            ArrF128(v) => print_arr(v.iter().map(|v| format!("RawF128(bits={})", v.to_bits()))),
            Struct(v) => print_fields(v.entries(), indent + 1)?,
            Raw(v) => {
                println!("raw = {:?}", v.data);
            }
        }
    }

    Ok(())
}

fn print_arr<T: core::fmt::Display + Sized>(iter: impl Iterator<Item = T>) {
    print!("value = [");
    for value in iter {
        print!("{}, ", value);
    }
    println!("]");
}
