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

use anyhow::{bail, Result};

use crate::cbor_object::ToTree;
use crate::cbor_parser::parse_cbor;

pub fn parse_hex(hex_content: &str) -> Result<Vec<u8>> {
    let Ok((_, bytes_content)) = hex_parser::parse_hex(hex_content) else {
        bail!("Error parsing hex data");
    };
    Ok(bytes_content)
}

pub fn dump_cbor_tree(bytes_content: &[u8]) -> Result<String> {
    let Ok((rest, object)) = parse_cbor(bytes_content) else {
        bail!("Error parsing CBOR data");
    };

    let cbor_tree = object.into_tree();

    let mut output = String::new();
    cbor_tree.write(&mut output);
    if !rest.is_empty() {
        writeln!(
            &mut output,
            "trailing bytes {}",
            rest.iter().fold(String::new(), |mut acc, b| {
                write!(acc, "{:02x}", b).unwrap();
                acc
            })
        )?;
    }

    Ok(output)
}
