use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::collections::BTreeMap;
use chrono::prelude::*;
use ::function_name::named;

use num::*;
use clap::Parser;

use std::slice;
use std::str;

use std::collections::VecDeque;

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf}

#[derive(Debug)]
struct Version(String);

#[derive(Debug)]
enum Timescale {ps, ns, us, ms, s, unit}

#[derive(Debug)]
struct Metadata {
    date      : Option<DateTime<Utc>>,
    version   : Option<Version>,
    timescale : (Option<u32>, Timescale)}

#[derive(Debug)]
struct Scope_Idx(usize);

#[derive(Debug)]
struct Signal_Idx(usize);

#[derive(Debug)]
enum SignalGeneric{
    Signal{
        name           : String,
        timeline       : BTreeMap<BigInt, BigInt>,
        scope_parent   : Scope_Idx},
    SignalAlias{
        name          : String,
        signal_alias  : Signal_Idx}
}

#[derive(Debug)]
struct Scope {
    name          : String,
    child_signals : Vec<Signal_Idx>,
    child_scopes  : Vec<Scope_Idx>}


#[derive(Debug)]
struct VCD {
    metadata    : Metadata,
    all_signals : Vec<SignalGeneric>,
    // the root scope should always be placed at index 0
    all_scopes  : Vec<Scope>}

impl VCD {
    pub fn new() -> Self {
        let metadata = Metadata {
            date      : None,
            version   : None,
            timescale : (None, Timescale::unit)};
        VCD {
            metadata    : metadata,
            all_signals : Vec::<SignalGeneric>::new(),
            all_scopes  : Vec::<Scope>::new()}
        }
    }


#[derive(Debug)]
struct Line(usize);
#[derive(Debug)]
struct Word(usize);
#[derive(Debug)]
struct Cursor(Line, Word);

struct YieldByWord {
    reader       : io::BufReader<File>,
    EOF          : bool,
    buffers      : Vec<String>,
    curr_line    : usize,
    str_slices   : VecDeque<(*const u8, usize, Cursor)>,
}

impl YieldByWord {
    fn new(file : File) -> YieldByWord {
        let mut reader = io::BufReader::new(file);
        YieldByWord {
            reader       : reader,
            EOF          : false,
            buffers      : vec![],
            curr_line    : 0,
            str_slices   : VecDeque::new()
        }
    }

    fn next_word(&mut self) -> Option<(&str, Cursor)> {
        // if there are no more words, attempt to read more content
        // from the file
        if self.str_slices.is_empty() {
            self.buffers.clear();

            if self.EOF {return None}

            let num_buffers = 10;

            for buf_idx in 0..num_buffers {
                self.buffers.push(String::new());
                self.curr_line += 1;
                let bytes_read = self.reader.read_line(&mut self.buffers[buf_idx]).unwrap();

                // if we've reached the end of the file on the first attempt to read
                // a line in this for loop, no further attempts are necessary and we
                if bytes_read == 0 {
                    self.EOF = true; 
                    break;
                }

                let mut words = self.buffers[buf_idx].split_ascii_whitespace();
                
                for word in words.enumerate() {
                    let (word_idx, word) = word;
                    let position = Cursor(Line(self.curr_line), Word(word_idx + 1));
                    self.str_slices.push_back((word.as_ptr(), word.len(), position))
                }

            }
        }

        // if after we've attempted to read in more content from the file,
        // there are still no words...
        if self.str_slices.is_empty() {
            return None
        }

        // if we make it here, we return the next word
        unsafe {
            let (ptr, len, position) = self.str_slices.pop_front().unwrap();
            let slice = slice::from_raw_parts(ptr, len);
            return Some((str::from_utf8(slice).unwrap(), position));
        };
    }
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let file           = File::open(&args.path)?;
    let mut word_gen   = YieldByWord::new(file);
    let mut word_count = 0;

    while word_gen.next_word().is_some() {
        word_count += 1;
    }
    dbg!(word_count);

    // loop {
    //     let word = word_gen.next_word();
    //     if word.is_none() {break};

    //     dbg!(word.unwrap());
    // }


    Ok(())
}