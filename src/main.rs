use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::collections::BTreeMap;

use num::*;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

struct Timestamp{
    file_offset: u64,
    timestamp:   BigInt
}

struct Cursor{
    line: u64,
    col : u64
}

enum Tokens {
    Date,
    End,
    String,
    Version,
    Time,
}

struct Signal {
    name          : String,
    timeline      : BTreeMap<BigInt, BigInt>,
    children_arena: Vec<usize>,
    parent_index  : usize
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    let space = " ".as_bytes()[0];

    let file       = File::open(&args.path)?;
    let mut reader = io::BufReader::new(file);

    let mut buffer = String::new();
    let mut word_count = 0u64;
    let mut do_break = false;
    let line_chunk_size = 25;

    while {!do_break} {
        for _ in 0..line_chunk_size {
            let bytes_read = reader.read_line(&mut buffer).unwrap();
            if bytes_read == 0 {
                do_break = true;
                break
            }
        }

        let words = buffer.split_ascii_whitespace();

        for word in words {
            word_count += 1;
        }
        
        buffer.clear();
    }

    dbg!(word_count);

    Ok(())
}
