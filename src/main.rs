use std::io::prelude::*;
use std::io;
use std::fs::File;

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

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let file       = File::open(&args.path)?;
    let mut reader = io::BufReader::new(file);

    let mut buffer             = String::new();
    let mut timestamp_offsets  = Vec::new();
    let mut timestamps         = Vec::new();

    while {
        let bytes_read = reader.read_line(&mut buffer).unwrap();
        bytes_read > 0
    } {
        if &buffer[0..1] == "#" {
            let pos = reader.stream_position().unwrap();
            timestamp_offsets.push(pos);

            let timestamp = {
                let len = buffer.len();
                let str_val = &buffer[1..(len - 1)].as_bytes();
                BigInt::parse_bytes(str_val, 10).unwrap()
            };
            timestamps.push(timestamp);
        }
        buffer.clear()
    }

    let index = 4;
    let timestamp_offset = timestamp_offsets.get(index).unwrap();
    let timestamp        = timestamps.get(index).unwrap();
    dbg!((timestamp_offset, timestamp));

    // seek to where we found the first timestamp and read
    // out the next line
    reader.seek(io::SeekFrom::Start(*timestamp_offset));
    reader.read_line(&mut buffer);
    dbg!(buffer);

    Ok(())
}
