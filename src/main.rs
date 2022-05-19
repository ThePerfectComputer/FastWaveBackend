use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::collections::BTreeMap;
use ::next_gen::prelude::*;

use num::*;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

struct Signal {
    name          : String,
    timeline      : BTreeMap<BigInt, BigInt>,
    children_arena: Vec<usize>,
    parent_index  : usize
}


#[generator(yield(String))]
fn yield_words(file : File) {
    let mut reader = io::BufReader::new(file);

    let mut buffer = String::new();
    let mut word_count = 0u64;
    let mut EOF = false;
    let line_chunk_size = 25;

    while {!EOF} {
        for _ in 0..line_chunk_size {
            let bytes_read = reader.read_line(&mut buffer).unwrap();
            if bytes_read == 0 {
                EOF = true;
                break
            }
        }

        let words = buffer.split_ascii_whitespace();

        for word in words {
            yield_!(word.to_string());
        }
        
        buffer.clear();
    }


}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let file       = File::open(&args.path)?;
    let mut word_count = 0;
    mk_gen!(let mut generator = yield_words(file));

    for word in generator {
        word_count += 1;
    }
    Ok(())
}
