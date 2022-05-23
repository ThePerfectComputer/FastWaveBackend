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

// TODO : Date_PArser_State -> Parse_Date
#[derive(Debug)]
enum Date_Parser_State {Weekday, Month, Day, HHMMSS, Year, End}
#[derive(Debug)]
enum Version_Parser_State {Parsing, Done}


#[derive(Debug)]
enum VCD_Parser_State {
    Begin, 
    Date(Date_Parser_State),
    Parse_Version(Version_Parser_State), 
    Parse_Signal_Tree,
    Parse_Signal_Values}

struct DateBuffer {
    Weekday : Option<String>,
    Month   : Option<String>,
    Day     : Option<String>,
    HHMMSS  : Option<String>,
    Year    : Option<String>}

struct VCD_Parser<'a> {
    vcd_parser_state   : VCD_Parser_State,
    date_parser_state  : Date_Parser_State,
    date_buffer        : DateBuffer,

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
        let date_buffer = DateBuffer{
            Weekday : None,
            Month   : None,
            Day     : None,
            HHMMSS  : None,
            Year    : None
        };
        VCD_Parser {
            vcd_parser_state : VCD_Parser_State ::Begin,
            date_parser_state : Date_Parser_State::Weekday,
            date_buffer : date_buffer,
            vcd : vcd,
            curr_scope : None,
            curr_parent_scope : None

        }
    }

    pub fn parse_word(&mut self, word : &str) -> Result<(), String> {
        let mut state = &mut self.vcd_parser_state;
        let t = &self.vcd;
        match state {
            VCD_Parser_State::Begin =>  
                match word {
                    "$date" => {*state = VCD_Parser_State::Date(Date_Parser_State::Weekday); Ok(())}
                    _ => Err(format!("unsure what to do with {word:?} in state `{state:?}`"))
                }
            VCD_Parser_State::Date(_) => self.parse_date(word),
            VCD_Parser_State::Parse_Version(_) => self.parse_date(word),
            // TODO : Enable the following in production
            // _ => Err(format!("parser in bad state : {state:?}"))TODO : Disable the following in production
            // TODO : Disable the following in production
            _ => Err(format!("parser in bad state : {state:?}; {t:?}"))
        }
    }

    pub fn parse_date(&mut self, word : &str) -> Result<(), String> {
        let mut state = &mut self.vcd_parser_state;
        match state {
            VCD_Parser_State::Date(Date_Parser_State::Weekday) =>
                {
                    self.date_buffer.Weekday = Some(word.to_string());
                    *state = VCD_Parser_State::Date(Date_Parser_State::Month);
                    Ok(())
                }
            VCD_Parser_State::Date(Date_Parser_State::Month) =>
                {
                    self.date_buffer.Month = Some(word.to_string());
                    *state = VCD_Parser_State::Date(Date_Parser_State::Day);
                    Ok(())
                }
            VCD_Parser_State::Date(Date_Parser_State::Day) =>
                {
                    self.date_buffer.Day = Some(word.to_string());
                    *state = VCD_Parser_State::Date(Date_Parser_State::HHMMSS);
                    Ok(())
                }
            VCD_Parser_State::Date(Date_Parser_State::HHMMSS) =>
                {
                    self.date_buffer.HHMMSS = Some(word.to_string());
                    *state = VCD_Parser_State::Date(Date_Parser_State::Year);
                    Ok(())
                }
            VCD_Parser_State::Date(Date_Parser_State::Year) =>
                {
                    self.date_buffer.Year = Some(word.to_string());

                    // now that we've successfully parsed all the date information,
                    // we store it to the metadata.date struct
                    let weekday = &self.date_buffer.Weekday.as_ref().unwrap();
                    let month   = &self.date_buffer.Month.as_ref().unwrap();
                    let day     = &self.date_buffer.Day.as_ref().unwrap();
                    let hhmmss  = &self.date_buffer.HHMMSS.as_ref().unwrap();
                    let year    = &self.date_buffer.Year.as_ref().unwrap();

                    let date = &format!("{weekday} {month} {day} {hhmmss} {year}")[..];
                    let dt   = Utc.datetime_from_str(date, "%a %b %e %T %Y")
                    .expect(&format!("invalid date {date}")[..]);

                    self.vcd.metadata.date = Some(dt);

                    *state = VCD_Parser_State::Date(Date_Parser_State::End);
                    Ok(())
                }
            VCD_Parser_State::Date(Date_Parser_State::End) =>
                {
                let expected_word = "$end";
                match word {
                    expected_word => {
                        *state = VCD_Parser_State::Parse_Version(Version_Parser_State::Parsing);
                        Ok(())
                    }
                    _ => Err(format!("expected `{expected_word}` but found `{word}`"))
                }
                }
            _   => Err(format!("{state:?} should be unreachable within DateParser.")),
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