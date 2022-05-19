use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::collections::BTreeMap;
use chrono::prelude::*;
use std::rc::Rc;

use num::*;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}


// TODO: implement any timescales greater than a second
enum Timescale {ps, ns, us, ms, s}

struct Metadata {
    date      : DateTime<FixedOffset>,
    version   : String,
    timescale : Timescale
}

struct Signal {
    name          : String,
    timeline      : BTreeMap<BigInt, BigInt>,
    children_arena: Vec<usize>,
    parent_index  : usize

} 

struct SignalAlias {
    name          : String,
    signal_alias  : Rc<Signal>
}

enum SignalGeneric{
    Signal(Signal),
    SignalAlias(SignalAlias),
}

struct Scope {
    name    : String,
    signals : Vec<SignalGeneric>,
    scopes  : Vec<Scope>,
}

struct VCD {
    metadata   : Metadata,
    top_scopes : Vec<Scope>
}


enum VCD_Parser_State {Date, Version, Timescale, SignalTree, Values}
enum Date_Parser_State {Date, Day, Month, HHMMSS, Year}

fn parse_vcd(word: &str, mut state : VCD_Parser_State) {}
fn parse_date(word : &str, mut state : Date_Parser_State) {}

fn yield_word_and_apply(file : File, mut f : impl FnMut(&str)) {
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
            f(word);
        }
        
        buffer.clear();
    }

}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    // let dt = Utc.datetime_from_str("Fri Nov 28 12:00:09 2014", "%a %b %e %T %Y");

    let file           = File::open(&args.path)?;
    let mut word_count = 0;

    yield_word_and_apply(file, |word| {word_count += 1});
    dbg!(word_count);
    Ok(())
}
