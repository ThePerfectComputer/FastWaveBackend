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
enum Timescale {ps, ns, us, ms, s}

struct Scope_Idx(usize);
struct Signal_Idx(usize);

struct Metadata {
    date      : DateTime<Utc>,
    version   : String,
    timescale : Timescale}

struct Signal {
    name           : String,
    timeline       : BTreeMap<BigInt, BigInt>,
    scope_parent   : Scope_Idx} 

struct SignalAlias {
    name          : String,
    signal_alias  : Signal_Idx}

enum SignalGeneric{
    Signal(Signal),
    SignalAlias(SignalAlias)}

struct Scope {
    name          : String,
    child_signals : Vec<Signal_Idx>,
    child_scopes  : Vec<Scope_Idx>}

struct VCD {
    metadata    : Metadata,
    all_signals : Vec<SignalGeneric>,
    // the root scope should always be placed at index 0
    all_scopes  : Vec<Scope>}

#[derive(Debug)]
enum Date_Parser_State {Weekday, Month, Day, HHMMSS, Year}

#[derive(Debug)]
enum VCD_Parser_State {
    Begin, 
    Date(Date_Parser_State),
    Parse_Version, 
    Parse_Signal_Tree,
    Parse_Signal_Values}

struct DateBuffer {
    Weekday : String,
    Month   : String,
    Day     : String,
    HHMMSS  : String,
    Year    : String}

struct VCD_Parser<'a> {
    vcd_parser_state   : VCD_Parser_State,
    date_parser_state  : Date_Parser_State,
    date_buffer        : DateBuffer,

    vcd                : &'a mut VCD,
    curr_scope         : &'a Scope,
    curr_parent_scope  : &'a Scope}

impl VCD {
    pub fn new() -> Self {
        let dt = Utc
                 .datetime_from_str("Thu Jan 1 00:00:00 1970", "%a %b %e %T %Y")
                 .unwrap();
        let metadata = Metadata {
            date      : dt,
            version   : "".to_string(),
            timescale : Timescale::ps};
        let signal = Vec::<SignalGeneric>::new();
        VCD {
            metadata    : metadata,
            all_signals : Vec::<SignalGeneric>::new(),
            all_scopes  : Vec::<Scope>::new()}}}

impl<'a> VCD_Parser<'a> {
    pub fn new(&mut self, vcd : &'a mut VCD) {
        self.vcd_parser_state  = VCD_Parser_State::Begin;
        self.date_parser_state = Date_Parser_State::Weekday;
        self.vcd = vcd;}

    pub fn parse_word(&mut self, word : &str) -> Result<(), String> {
        let mut state = &mut self.vcd_parser_state;
        match state {
            VCD_Parser_State::Begin => {
                match word {
                    "$date"      => 
                    {
                        *state = VCD_Parser_State::Date(Date_Parser_State::Weekday);
                        Ok(())
                    }
                    // "$version"   => {*state = VCD_Parser_State::VERSION_ENTER; Ok(())},
                    // "$timescale" => {*state = VCD_Parser_State::TIMESCALE_ENTER; Ok(())},
                    _            => Err(format!("unsure what to do with {word:?}"))}},

            VCD_Parser_State::Date(_) => self.parse_date(word),
            _   => Err(format!("parser in bad state : {state:?}"))}
    }

    pub fn parse_date(&mut self, word : &str) -> Result<(), String> {
        let mut state = &mut self.vcd_parser_state;
        match state {
            VCD_Parser_State::Date(Date_Parser_State::Weekday) =>
                {
                    self.date_buffer.Weekday = word.to_string();
                    *state = VCD_Parser_State::Date(Date_Parser_State::Month);
                    Ok(())
                }
            VCD_Parser_State::Date(Date_Parser_State::Month) =>
                {
                    self.date_buffer.Month = word.to_string();
                    *state = VCD_Parser_State::Date(Date_Parser_State::Day);
                    Ok(())
                }
            VCD_Parser_State::Date(Date_Parser_State::Day) =>
                {
                    self.date_buffer.Day = word.to_string();
                    *state = VCD_Parser_State::Date(Date_Parser_State::HHMMSS);
                    Ok(())
                }
            VCD_Parser_State::Date(Date_Parser_State::HHMMSS) =>
                {
                    self.date_buffer.HHMMSS = word.to_string();
                    *state = VCD_Parser_State::Date(Date_Parser_State::Year);
                    Ok(())
                }
            VCD_Parser_State::Date(Date_Parser_State::Year) =>
                {
                    self.date_buffer.Year = word.to_string();

                    // now that we've successfully parsed all the date information,
                    // we store it to a d
                    let weekday = &self.date_buffer.Weekday;
                    let month   = &self.date_buffer.Month;
                    let day     = &self.date_buffer.Day;
                    let hhmmss  = &self.date_buffer.HHMMSS;
                    let year    = &self.date_buffer.Year;

                    let date = &format!("{weekday} {month} {day} {hhmmss} {year}")[..];
                    let dt   = Utc.datetime_from_str(date, "%a %b %e %T %Y").unwrap();

                    self.vcd.metadata.date = dt;

                    *state = VCD_Parser_State::Parse_Version;
                    Ok(())
                }
            _   => Err(format!("{state:?} should be unreachable within DateParser.")),
        }
    }
}

fn advance_VCD_parser_FSM(word: &str, mut state : VCD_Parser_State) {}
fn advance_Date_parser_FSM(word : &str, mut state : Date_Parser_State) {}

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