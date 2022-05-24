use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::collections::BTreeMap;
use chrono::prelude::*;
use std::rc::Rc;
use ::function_name::named;

use num::*;
use clap::Parser;

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
    timescale : Timescale}

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

#[derive(Debug)]
enum Date_Parser_State {Begin, Parsing}
#[derive(Debug)]
enum Version_Parser_State {Begin, Parsing}
#[derive(Debug)]
enum Timescale_Parser_State {Begin, Parsing}


#[derive(Debug)]
enum Parser_State {
    Date(Date_Parser_State),
    Version(Version_Parser_State), 
    Timescale(Timescale_Parser_State), 
    Parse_Signal_Tree,
    Parse_Signal_Values}

struct VCD_Parser<'a> {
    vcd_parser_state   : Parser_State,
    buffer             : Option<String>,

    vcd                : &'a mut VCD,
    curr_scope         : Option<&'a Scope>,
    curr_parent_scope  : Option<&'a Scope>}

impl VCD {
    pub fn new() -> Self {
        let dt = Utc
                 .datetime_from_str("Thu Jan 1 00:00:00 1970", "%a %b %e %T %Y")
                 .unwrap();
        let metadata = Metadata {
            date      : None,
            version   : None,
            timescale : Timescale::unit};
        let signal = Vec::<SignalGeneric>::new();
        VCD {
            metadata    : metadata,
            all_signals : Vec::<SignalGeneric>::new(),
            all_scopes  : Vec::<Scope>::new()}}}

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
            // TODO : Enable the following in production
            // _ => Err(format!("parser in bad state : {state:?}"))TODO : Disable the following in production
            // TODO : Disable the following in production
            _ => Err(format!("parser in bad state : {state:?}; {t:?}"))
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
                        self.parse_version(word);
                        Ok(())
                    }
                }
            Parser_State::Date(Date_Parser_State::Parsing) =>
                match word {
                    "$end" => {
                        *state = Parser_State::Version(Version_Parser_State::Begin); 
                        let s  = self.buffer.take().unwrap();
                        let dt = Utc.datetime_from_str(s.as_str(), "%a %b %e %T %Y")
                            .expect(&format!("invalid date {s}").as_str());
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
                        *state = Parser_State::Timescale(Timescale_Parser_State::Begin); 
                        let s = self.buffer.take().unwrap();
                        self.vcd.metadata.version = Some(Version(s));
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

fn yield_word_and_apply(file : File, mut f : impl FnMut(&str) -> Result<(), String>) {
    let mut reader = io::BufReader::new(file);

    let mut buffer = String::new();
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
            f(word).unwrap();
        }
        
        buffer.clear();
    }

}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let file           = File::open(&args.path)?;

    let mut vcd = VCD::new();
    let mut parser = VCD_Parser::new(&mut vcd);

    yield_word_and_apply(file, |word| {parser.parse_word(word)});
    dbg!(&vcd);

    Ok(())
}