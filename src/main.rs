use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H', long = "hex", default_value_t = false)]
    hex: bool,

    filepath: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    #[allow(clippy::collapsible_else_if)]
    let bytes_content = if args.hex {
        let hex_content = if let Some(filepath) = args.filepath {
            fs::read_to_string(filepath)?
        } else {
            io::read_to_string(io::stdin())?
        };

        match cbor_inspector::parse_hex(&hex_content) {
            Ok(bytes_content) => bytes_content,
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
    } else {
        if let Some(filepath) = args.filepath {
            fs::read(filepath)?
        } else {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            buffer
        }
    };
    let output = match cbor_inspector::dump_cbor_tree(&bytes_content) {
        Ok(output) => output,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
    print!("{}", output);
    Ok(())
}
