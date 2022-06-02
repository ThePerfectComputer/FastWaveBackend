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

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf}


// TODO: implement any timescales greater than a second
#[derive(Debug)]
enum Timescale {ps, ns, us, ms, s, unit}

#[derive(Debug)]
struct Scope_Idx(usize);

#[derive(Debug)]
struct Signal_Idx(usize);

#[derive(Debug)]
struct Version(String);

#[derive(Debug)]
struct Metadata {
    date      : Option<DateTime<Utc>>,
    version   : Option<Version>,
    timescale : (Option<u32>, Timescale)}

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

#[derive(Debug, PartialEq)]
enum Date_Parser_State {Begin, Parsing}
#[derive(Debug, PartialEq)]
enum Version_Parser_State {Begin, Parsing}
#[derive(Debug, PartialEq)]
enum Timescale_Parser_State {Begin, Parsing}
#[derive(Debug, PartialEq)]
enum Signal_Tree_Parser_State {Begin, Parsing}


#[derive(Debug, PartialEq)]
enum Parser_State {
    Date(Date_Parser_State),
    Version(Version_Parser_State), 
    Timescale(Timescale_Parser_State), 
    Signal_Tree(Signal_Tree_Parser_State),
    Parse_Signal_Values}

struct VCD_Parser<'a> {
    vcd_parser_state   : Parser_State,
    buffer             : Option<String>,

    vcd                : &'a mut VCD,
    curr_scope         : Option<&'a Scope>,
    curr_parent_scope  : Option<&'a Scope>}

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

impl<'a> VCD_Parser<'a> {
    pub fn new(vcd : &'a mut VCD) -> Self {
        VCD_Parser {
            vcd_parser_state : Parser_State::Date(Date_Parser_State::Begin),

            buffer      : None,
            vcd : vcd,
            curr_scope : None,
            curr_parent_scope : None
        }
    }

    pub fn parse_word(&mut self, word : &str) -> Result<(), String> {
        let mut state = &mut self.vcd_parser_state;
        let t = &self.vcd;
        match state {
            Parser_State::Date(_) => self.parse_date(word),
            Parser_State::Version(_) => self.parse_version(word),
            Parser_State::Timescale(_) => self.parse_timescale(word),
            // TODO : Enable the following in production
            // _ => Err(format!("parser in bad state : {state:?}"))
            // TODO : Disable the following in production
            _ => {
                Err(format!("parser in bad state : {state:?}; {t:?}"))
            }
        }
    }

    #[named]
    pub fn parse_date(&mut self, word : &str) -> Result<(), String> {
        let mut state = &mut self.vcd_parser_state;
        match state {
            Parser_State::Date(Date_Parser_State::Begin) =>
                match word {
                    "$date" => {
                        *state = Parser_State::Date(Date_Parser_State::Parsing); 
                        Ok(())
                    }
                    _ => {
                        *state = Parser_State::Version(Version_Parser_State::Begin); 
                        self.parse_version(word)
                    }
                }
            Parser_State::Date(Date_Parser_State::Parsing) =>
                match word {
                    "$end" => {
                        let s  = self.buffer.take().unwrap();
                        let dt = Utc.datetime_from_str(s.as_str(), "%a %b %e %T %Y")
                        .expect(&format!("invalid date {s}").as_str());
                        *state = Parser_State::Version(Version_Parser_State::Begin); 
                        self.vcd.metadata.date = Some(dt);
                        Ok(())
                    }
                    _ => {
                        if let Some(ref mut buffer) = self.buffer {
                            buffer.push_str(" ");
                            buffer.push_str(word);
                        }
                        else {
                            self.buffer = Some(word.to_string());
                        }
                        Ok(())
                    }
                }
            _   => Err(format!("{state:?} should be unreachable within {}.",function_name!())),

        }
    }

    #[named]
    pub fn parse_statement(
        &'a mut self, 
        curr_word : &str,
        key_word  : &str,
        begin_state   : Parser_State,
        parsing_state : Parser_State,
        end_state     : Parser_State,
        next_parser   : fn(&'a mut VCD_Parser, &str) -> Result<(), String>
    ) -> Result<(), String> {
        let mut state = &mut self.vcd_parser_state;

        if (*state == begin_state) {
            return match curr_word {
                key_word => {
                    *state = Parser_State::Date(Date_Parser_State::Parsing); 
                    Ok(())
                }
                _ => {
                    *state = Parser_State::Version(Version_Parser_State::Begin); 
                    next_parser(self, curr_word)
                }
            }
        }
        else {
            Ok(())
        }
        // Ok(())

        // match state {
        //     Parser_State::Date(Date_Parser_State::Begin) =>
        //         match curr_word {
        //             key_word => {
        //                 *state = Parser_State::Date(Date_Parser_State::Parsing); 
        //                 Ok(())
        //             }
        //             _ => {
        //                 *state = Parser_State::Version(Version_Parser_State::Begin); 
        //                 self.parse_version(curr_word)
        //             }
        //         }
        //     Parser_State::Date(Date_Parser_State::Parsing) =>
        //         match curr_word {
        //             "$end" => {
        //                 let s  = self.buffer.take().unwrap();
        //                 let dt = Utc.datetime_from_str(s.as_str(), "%a %b %e %T %Y")
        //                 .expect(&format!("invalid date {s}").as_str());
        //                 *state = Parser_State::Version(Version_Parser_State::Begin); 
        //                 self.vcd.metadata.date = Some(dt);
        //                 Ok(())
        //             }
        //             _ => {
        //                 if let Some(ref mut buffer) = self.buffer {
        //                     buffer.push_str(" ");
        //                     buffer.push_str(curr_word);
        //                 }
        //                 else {
        //                     self.buffer = Some(curr_word.to_string());
        //                 }
        //                 Ok(())
        //             }
        //         }
        //     _   => Err(format!("{state:?} should be unreachable within {}.",function_name!())),

        // }
    }

    #[named]
    pub fn parse_version(&mut self, word : &str) -> Result<(), String> {
        let mut state = &mut self.vcd_parser_state;
        match state {
            Parser_State::Version(Version_Parser_State::Begin) =>
                match word {
                    "$version" => {
                        *state = Parser_State::Version(Version_Parser_State::Parsing); 
                        Ok(())
                    }
                    _ => {
                        *state = Parser_State::Timescale(Timescale_Parser_State::Begin); 
                        Ok(())
                    }
                }
            Parser_State::Version(Version_Parser_State::Parsing) =>
                match word {
                    "$end" => {
                        let s = self.buffer.take().unwrap();
                        self.vcd.metadata.version = Some(Version(s));
                        *state = Parser_State::Timescale(Timescale_Parser_State::Begin); 
                        Ok(())
                    }
                    _ => {
                        if let Some(ref mut buffer) = self.buffer {
                            buffer.push_str(" ");
                            buffer.push_str(word);
                        }
                        else {
                            self.buffer = Some(word.to_string());
                        }
                        Ok(())
                    }
                }
            _   => Err(format!("{state:?} should be unreachable within {}.",function_name!())),

        }
    }

    #[named]
    pub fn parse_timescale(&mut self, word : &str) -> Result<(), String> {
        let mut state = &mut self.vcd_parser_state;
        match state {
            Parser_State::Timescale(Timescale_Parser_State::Begin) =>
                match word {
                    "$timescale" => {
                        *state = Parser_State::Timescale(Timescale_Parser_State::Parsing); 
                        Ok(())
                    }
                    _ => {
                        *state = Parser_State::Signal_Tree(Signal_Tree_Parser_State::Begin); 
                        Ok(())
                    }
                }
            Parser_State::Timescale(Timescale_Parser_State::Parsing) =>
                match word {
                    "$end" => {
                        let s = self.buffer.take().unwrap();
                        let s = s.split_ascii_whitespace();
                        let s = s.collect::<Vec<&str>>();

                        let scalar = s[0].to_string().parse::<u32>().unwrap();
                        let unit = s[1];
                        let unit = match unit {
                            "ps" => Ok(Timescale::ps),
                            "ns" => Ok(Timescale::ns),
                            "us" => Ok(Timescale::us),
                            "ms" => Ok(Timescale::ms),
                            "s"  => Ok(Timescale::s),
                            // TODO : see if there is a way to easily print out all enum variants
                            // _    => Err(format!("{word} is not a valid unit of time in {Timescale}"))
                            _    => Err(format!("{unit} is not a valid unit"))
                        }.unwrap();

                        dbg!(s);
                        self.vcd.metadata.timescale = (Some(scalar), unit);
                        *state = Parser_State::Timescale(Timescale_Parser_State::Begin); 
                        Ok(())
                    }
                    _ => {
                        if let Some(ref mut buffer) = self.buffer {
                            buffer.push_str(" ");
                            buffer.push_str(word);
                        }
                        else {
                            self.buffer = Some(word.to_string());
                        }
                        Ok(())
                    }
                }
            _   => Err(format!("{state:?} should be unreachable within {}.",function_name!())),

        }
    }
}

struct Line(u32);
struct Col(u32);
struct Position(Line, Col);

fn yield_word_and_apply(file : File, mut f : impl FnMut(&[u8], Position) -> Result<(), String>) {
    let mut reader = io::BufReader::new(file);

    let mut buffer = String::new();

    let mut line = 0u32;
    while true {
        let bytes_read = reader.read_line(&mut buffer).unwrap();
        if bytes_read == 0 {break}

        line += 1;
        let mut col = 1u32;

        let mut words = buffer.split_ascii_whitespace();

        for word in words {
            let word = word.as_bytes();
            let position = Position(Line(line), Col(col));
            f(word, position).unwrap();
            col += (word.len() as u32) + 1;
        }
        
        buffer.clear();
    }

}

struct YieldByWord {
    reader       : io::BufReader<File>,
    words        : Vec<String>,
    EOF          : bool,
    buffer       : String,
    str_slices  : Vec<(*const u8, usize)>,
}

impl YieldByWord {
    fn new(file : File) -> YieldByWord {
        let mut reader = io::BufReader::new(file);
        YieldByWord {
            reader       : reader,
            words        : vec![],
            EOF          : false,
            buffer : "".to_string(),
            str_slices : vec![],
        }
    }

    fn next_word(&mut self) -> Option<&str> {
        // if there are no more words, attempt to read more content
        // from the file
        if self.str_slices.is_empty() {
            self.buffer.clear();

            if self.EOF {return None}

            let line_chunk_size = 10;

            for _ in 0..line_chunk_size {
                let bytes_read = self.reader.read_line(&mut self.buffer).unwrap();
                // we hit the end of the file, so we go ahead and return None
                if bytes_read == 0 {self.EOF = true}
            }

            let words = self.buffer.split_ascii_whitespace();
            self.str_slices = words
                                .rev()
                                .map(|s| (s.as_ptr(), s.len()))
                                .collect();
        }

        // if we make it here, we return the next word
        unsafe {
            let (ptr, len) = self.str_slices.pop().unwrap();
            let slice = slice::from_raw_parts(ptr, len);
            return Some(str::from_utf8(slice).unwrap());
        };
    }
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let file           = File::open(&args.path)?;
    let mut word_gen   = YieldByWord::new(file);
    let mut word_count = 0;
    let mut last_word = String::new();

    // for word in 0..5 {
    //     dbg!(word_gen.next_word());
    // }
    while word_gen.next_word().is_some() {
        word_count += 1;
    }
    dbg!(word_count);

    // loop {
    //     let next_word = word_gen.next_word();
    //     if next_word.is_some() {
    //         last_word = next_word.unwrap();
    //     }
    //     else {
    //         break
    //     }
    // }

    // dbg!(last_word);

    Ok(())
}