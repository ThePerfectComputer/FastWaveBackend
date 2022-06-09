use super::*;
use chrono::prelude::*;
use itertools::Itertools;
use std::fs::File;
use ::function_name::named;

#[derive(Debug)]
pub struct Residual<'a>(&'a str);

pub fn take_until<'a>(word : &'a str, pattern : u8) -> Option<(&'a str, Residual)> {
    let mut new_start  = 0;

    for chr in word.as_bytes() {
        if (*chr == pattern) {
            return Some((&word[0..new_start], Residual(&word[new_start+1..])));
        } 
        else {
            new_start += 1;
        }
    }

    None
}

fn tag<'a>(word : &'a str, pattern : &'a str) -> Option<&'a str> {
    let lhs           = word.as_bytes().iter();
    let rhs           = pattern.as_bytes();
    let iter          = lhs.zip(rhs);
    let mut new_start = 0;

    let mut res = true;
    for (c_lhs, c_rhs) in iter {
        res = res && (c_lhs == c_rhs);
        if !res {return None}
        new_start += 1;
    }

    Some(&word[new_start..])
}

#[named]
fn parse_date(
    word_and_ctx1 : (&str, Cursor),
    word_and_ctx2 : (&str, Cursor),
    word_and_ctx3 : (&str, Cursor),
    word_and_ctx4 : (&str, Cursor),
    word_and_ctx5 : (&str, Cursor),
) -> Result<DateTime<Utc>, String> {

    let day = {
        // check for another word in the file
        let (word, cursor) = word_and_ctx1;
    
        let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        if !days.contains(&word) {
            let msg  = format!("reached end of file without parser leaving {}\n", function_name!());
            let msg2 = format!("{word} is not a valid weekday : expected one of {days:?}\n");
            let msg3 = format!("failure location: {cursor:?}");
            return Err(format!("{}{}{}", msg, msg2, msg3))
        }

        word.to_string()
    };

    let month = {
        // check for another word in the file
        let (word, cursor) = word_and_ctx2;

        let months = [
            "Jan", "Feb", "Mar", "Apr", 
            "May", "Jun", "Jul", "Aug", 
            "Sept", "Oct", "Nov", "Dec", 
            ];

        if !months.contains(&word) {
            let msg  = format!("reached end of file without parser leaving {}\n", function_name!());
            let msg2 = format!("{word} is not a valid month : expected one of {months:?}\n");
            let msg3 = format!("failure location: {cursor:?}");
            return Err(format!("{}{}{}", msg, msg2, msg3))
        }

        word.to_string()
    };

    let date = {
        // check for another word in the file
        let (word, cursor) = word_and_ctx3;

        let date : u8 = word.to_string().parse().unwrap();

        if date > 31 {
            let msg  = format!("reached end of file without parser leaving {}\n", function_name!());
            let msg2 = format!("{word} is not a valid date : must be between 0 and 31\n");
            let msg3 = format!("failure location: {cursor:?}");
            return Err(format!("{}{}{}", msg, msg2, msg3))

        }

        word.to_string()
    };

    let (hh, mm, ss) = {
        // get hour
        let (word, cursor) = word_and_ctx4;

        let (hh, Residual(remainder)) = take_until(word, b':').unwrap();
        let hh : u8 = hh.to_string().parse().unwrap();

        if hh > 23 {
            let msg  = format!("reached end of file without parser leaving {}\n", function_name!());
            let msg2 = format!("{hh} is not a valid hour : must be between 0 and 23\n");
            let msg3 = format!("failure location: {cursor:?}");
            return Err(format!("{}{}{}", msg, msg2, msg3))
        }

        // get minute
        let (mm, Residual(remainder)) = take_until(remainder, b':').unwrap();
        let mm : u8 = mm.to_string().parse().unwrap();

        if mm > 60 {
            let msg  = format!("reached end of file without parser leaving {}\n", function_name!());
            let msg2 = format!("{mm} is not a valid minute : must be between 0 and 60\n");
            let msg3 = format!("failure location: {cursor:?}");
            return Err(format!("{}{}{}", msg, msg2, msg3))
        }

        // get second
        let ss : u8 = remainder.to_string().parse().unwrap();

        if ss > 60 {
            let msg  = format!("reached end of file without parser leaving {}\n", function_name!());
            let msg2 = format!("{ss} is not a valid second : must be between 0 and 60\n");
            let msg3 = format!("failure location: {cursor:?}");
            return Err(format!("{}{}{}", msg, msg2, msg3))
        }
        (hh.to_string(), mm.to_string(), ss.to_string())
    };

    let year = {
        // check for another word in the file
        let (word, cursor) = word_and_ctx5;
        word.to_string()
    };

    let date = Utc.datetime_from_str(
        format!("{day} {month} {date} {mm}:{hh}:{ss} {year}").as_str(),
        "%a %b %e %T %Y").unwrap();

    Ok(date)
}

#[named]
fn parse_header(word_reader : &mut WordReader) -> Result<Metadata, String> {
    let mut header = Metadata {
        date : None,
        version : None,
        timescale : (None, Timescale::unit)
    };

    loop {
        // check for another word in the file
        let word = word_reader.next_word();

        // if there isn't another word left in the file, then we exit
        if word.is_none() {
            return Err(format!("reached end of file without parser leaving {}", function_name!()))
        }

        // destructure
        let (word, cursor) = word.unwrap();
        let ident = tag(word, "$");

        match tag(word, "$") {
            // we hope that this word stars with a `$`
            Some(ident) =>  {
                match ident {
                    "date"      => {
                        let err_msg = format!("reached end of file without parser leaving {}", function_name!());
                        // a date is typically composed of the 5 following words which can 
                        // occur in any order: 
                        // {Day, Month, Date(number in month), hh:mm:ss, year}.
                        // Thus, we must lookahead read the 5 next words, and try our date
                        // parser on 5! = 120 permutations of the 5 words.
                        //
                        // While looking ahead, if one of the 5 words in `$end`, we have to 
                        // immediately stop trying to get more words.

                        let mut found_end = false;
                        let mut lookahead_5_words : Vec<(String, Cursor)> = Vec::new();

                        for word in 0..5 {
                            let (word, cursor) = word_reader.next_word().expect(err_msg.as_str());
                            let word = word.to_string();
                            match word.as_str() {
                                "$end" => {
                                    found_end = true;
                                    break;
                                }
                                _ => {
                                    lookahead_5_words.push((word, cursor));
                                }
                            };
                        }

                        // we no longer attempt to parse date if we weren't able to lookahead 5
                        // words
                        if found_end {continue}

                        let iter =  lookahead_5_words
                                    .iter()
                                    .permutations(lookahead_5_words.len());
                        // let parsed_date = parse_date(word_reader).unwrap();
                        // header.date     = Some(parsed_date);
                    }
                    "version"   => {println!("got version")}
                    "timescale" => {println!("got timescale")}
                    "scope"     => {break}
                    _ => {}
                }
            }
            // if not, then we keep looping
            None => {}
        }

    }
    return Ok(header)
}

pub fn parse_vcd(file : File) {
    let mut word_gen = WordReader::new(file);

    let header = parse_header(&mut word_gen).unwrap();
    dbg!(header);
}