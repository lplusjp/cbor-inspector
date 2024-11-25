mod cbor_object;
mod cbor_parser;
mod hex_parser;
mod tree;
mod type_array;
mod type_byte_string;
mod type_common;
mod type_map;
mod type_negative;
mod type_simple_or_float;
mod type_tag;
mod type_text_string;
mod type_unsigned;

#[cfg(test)]
mod test_parse_cbor_and_build_tree;

use std::fmt::Write;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::cbor_object::ToTree;
use crate::cbor_parser::parse_cbor;
use crate::hex_parser::parse_hex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H', long = "hex", default_value_t = false)]
    hex: bool,

    filepath: Option<PathBuf>,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    let bytes_content = if args.hex {
        let hex_content = if let Some(filepath) = args.filepath {
            fs::read_to_string(filepath)?
        } else {
            io::read_to_string(io::stdin())?
        };

        let Ok((rest, bytes_content)) = parse_hex(&hex_content) else {
            eprintln!("Error parsing hex data");
            std::process::exit(1);
        };
        if !rest.is_empty() {
            eprintln!("Error parsing hex data: trailing bytes {:02x?}", rest);
            std::process::exit(1);
        }
        bytes_content
    } else {
        if let Some(filepath) = args.filepath {
            fs::read(filepath)?
        } else {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            buffer
        }
    };

    let Ok((rest, object)) = parse_cbor(&bytes_content) else {
        eprintln!("Error parsing CBOR data");
        std::process::exit(1);
    };

    let cbor_tree = object.into_tree();

    let mut output = String::new();
    cbor_tree.write(&mut output);
    print!("{}", output);
    if !rest.is_empty() {
        println!(
            "trailing bytes {}",
            rest.iter().fold(String::new(), |mut acc, b| {
                write!(acc, "{:02x}", b).unwrap();
                acc
            })
        );
    }

    Ok(())
}
