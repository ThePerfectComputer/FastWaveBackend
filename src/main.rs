use std::io::prelude::*;
use std::io;
use std::fs::File;

use genawaiter::{sync::gen, yield_};
use num::*;
use clap::Parser;
use chrono::prelude::*;
use std::collections::BTreeMap;

use nom_bufreader::bufreader::BufReader;
use nom_bufreader::{Error, Parse};
use nom::{
    branch::alt,
    bytes::streaming::{tag, take_until, take_while1},
    character::streaming::{space0,alpha1, multispace1, digit1},
    combinator::map_res,
    IResult,
    sequence::tuple
};

// TODO : Remove
use std::str::from_utf8;

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

fn method(i: &[u8]) -> IResult<&[u8], String, ()> {
    map_res(alt((tag("GET"), tag("POST"), tag("HEAD"))), |s| {
        from_utf8(s).map(|s| s.to_string())
    })(i)
}

fn path(i: &[u8]) -> IResult<&[u8], String, ()> {
    map_res(take_until(" "), |s| from_utf8(s).map(|s| s.to_string()))(i)
}

fn date(i: &[u8]) -> IResult<&[u8], DateTime<Utc>, ()> {
    let (i, _) = tag("$date")(i)?;
    
    let (i, _) = multispace1(i)?;
    let (i, weekday) = alt((
        tag("Mon"), tag("Tue"), tag("Wed"), tag("Thu"), 
        tag("Fri"), tag("Sat"), tag("Sun")))(i)?;
        
    let (i, _) = multispace1(i)?;
    let (i, month) = alt((
        tag("Jan"), tag("Feb"), tag("Mar"), tag("Apr"), 
        tag("May"), tag("Jun"), tag("July"), tag("Sept"), 
        tag("Oct"), tag("Nov"), tag("Dec")))(i)?;
    
    let (i, _) = multispace1(i)?;
    let (i, day) = digit1(i)?;

    let (i, _) = multispace1(i)?;
    let (i, hour) = digit1(i)?;

    let (i, _) = tag(":")(i)?;
    let (i, minute) = digit1(i)?;

    let (i, _) = tag(":")(i)?;
    let (i, second) = digit1(i)?;

    let (i, _) = multispace1(i)?;
    let (i, year) = digit1(i)?;

    let (i, _) = multispace1(i)?;
    let (i, _) = tag("$end")(i)?;

    let (weekday, month, day, hour, minute, second, year) = (
        from_utf8(weekday).unwrap(),
        from_utf8(month).unwrap(),
        from_utf8(day).unwrap(),
        from_utf8(hour).unwrap(),
        from_utf8(minute).unwrap(),
        from_utf8(second).unwrap(),
        from_utf8(year).unwrap(),
    );

    let dt = Utc.datetime_from_str(
        format!("{weekday} {month} {day} {hour}:{minute}:{second} {year}")
        .as_str(), "%a %b %e %T %Y")
        .unwrap();
    
    Ok((i, dt))
    
}

fn space(i: &[u8]) -> IResult<&[u8], (), ()> {
    let (i, _) = space0(i)?;
    Ok((i, ()))
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let file       = File::open(&args.path)?;
    let mut reader = BufReader::new(file);
    let m = reader.parse(date).expect("failed to parse date");
    dbg!(m.to_rfc2822());

    // let mut file_by_line = gen!({
    //     while {
    //         let bytes_read = reader.read_line(&mut buffer).unwrap();
    //         bytes_read > 0
    //     } {
    //         yield_!(buffer.as_bytes());
    //         buffer.clear()
    //     }
    // });

    // for line in file_by_line {
    //     dbg!(&line);
    // }



    Ok(())
}
