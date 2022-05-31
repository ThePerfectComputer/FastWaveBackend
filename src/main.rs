use std::io::prelude::*;
use std::io;
use std::fs::File;

use num::*;
use clap::Parser;
use chrono::prelude::*;
use std::collections::BTreeMap;

use nom_bufreader::bufreader::BufReader;
use nom_bufreader::{Error, Parse};
use nom::{
    branch::alt,
    bytes::streaming::{tag, take_until, take_while1},
    character::is_alphanumeric,
    character::complete::none_of,
    character::streaming::{
        space0, alpha1, multispace0,
        multispace1, digit1, line_ending,
        alphanumeric1},
    combinator::map_res,
    IResult,
    sequence::{tuple, delimited, terminated, preceded}
};

// TODO : Remove
use std::str::from_utf8;

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
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

fn version(i: &[u8]) -> IResult<&[u8], String, ()> {
    let (i, _) = tag("$version")(i)?;
    let (mut i, _) = multispace1(i)?;

    let mut version = "".to_string();
    let mut reached_end = false;

    while (reached_end == false) {
        let (_i, next_word) = take_while1(
            |c| 
                c != 36 && // not $
                c >= 33 && //  between ! and ~
                c <= 126
            )(i)?;
        i = _i;
        let next_word      = from_utf8(next_word).unwrap();

        version = format!("{version} {next_word}");
        let (_i, _) = multispace1(i)?;
        i = _i;

        match tag::<&str, &[u8], ()>("$end")(i) {
            Ok((_i, _)) => {
                i = _i;
                reached_end = true;
            }
            Err(_) => {}
        };

    }

    // strip the initial space
    version = version[1..].to_string();

    Ok((i, version))
        
}

fn timescale(i: &[u8]) -> IResult<&[u8], (Option<u32>, Timescale), ()> {
    let (i, _) = tag("$timescale")(i)?;
    dbg!("here");
    let (i, _) = multispace1(i)?;
    
    let (i, scale) = digit1(i)?;
    let (i, _) = multispace1(i)?;

    let (i, unit)  = alpha1(i)?;
    let (i, _) = multispace1(i)?;

    let (i, _) = tag("$end")(i)?;

    let scale = from_utf8(scale).unwrap().to_string();
    let scale = scale.parse::<u32>().unwrap();

    let unit = match from_utf8(unit).unwrap() {
        "ps" => Ok(Timescale::ps),
        "ns" => Ok(Timescale::ns),
        "us" => Ok(Timescale::us),
        "ms" => Ok(Timescale::ms),
        "s"  => Ok(Timescale::s),
          _  => Err(())
    }.unwrap();


    Ok((i, (Some(scale), unit)))
        
}

fn f_multispace0(i: &[u8]) -> IResult<&[u8], (), ()> {
    let (i, _) = multispace0(i)?;
    Ok((i, ()))
}

fn main() -> Result<(), Error<()>> {
    let args = Cli::parse();

    let file       = File::open(&args.path)?;
    let mut reader = BufReader::new(file);

    reader.parse(f_multispace0).unwrap();
    let date = reader.parse(date).expect("failed to parse date");

    reader.parse(f_multispace0).unwrap();
    let version = reader.parse(version).expect("failed to parse version");

    reader.parse(f_multispace0).unwrap();
    let timescale = reader.parse(timescale).expect("failed to parse timescale");

    dbg!(date.to_rfc2822());
    dbg!(version);
    dbg!(timescale);

    Ok(())
}