use chrono::prelude::*;
use itertools::Itertools;
use std::fs::File;
use ::function_name::named;

use super::*;

#[named]
pub(super) fn parse_date(
    word_and_ctx1 : (&str, &Cursor),
    word_and_ctx2 : (&str, &Cursor),
    word_and_ctx3 : (&str, &Cursor),
    word_and_ctx4 : (&str, &Cursor),
    word_and_ctx5 : (&str, &Cursor),
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

        let date : u8 = match word.to_string().parse() {
            Ok(date) => date,
            Err(_) => {return Err("".to_string())}
        };

        if date > 31 {
            let msg  = format!("reached end of file without parser leaving {}\n", function_name!());
            let msg2 = format!("{word} is not a valid date : must be between 0 and 31\n");
            let msg3 = format!("failure location: {cursor:?}");
            return Err(format!("{}{}{}", msg, msg2, msg3))

        }

        date.to_string()
    };

    let (hh, mm, ss) = {
        // get hour
        let (word, cursor) = word_and_ctx4;

        let res = take_until(word, b':');
        res.assert_match()?;
        let hh : u8 = res.matched.to_string()
                        .parse()
                        .map_err(|_| "failed to parse".to_string())?;

        if hh > 23 {
            let msg  = format!("reached end of file without parser leaving {}\n", function_name!());
            let msg2 = format!("{hh} is not a valid hour : must be between 0 and 23\n");
            let msg3 = format!("failure location: {cursor:?}");
            return Err(format!("{}{}{}", msg, msg2, msg3))
        }

        // get minute
        let word = &res.residual[1..]; // chop off colon which is at index 0
        let res = take_until(word, b':');
        res.assert_match()?;
        let mm : u8 = res.matched.to_string()
                        .parse()
                        .map_err(|_| "failed to parse".to_string())?;

        if mm > 60 {
            let msg  = format!("reached end of file without parser leaving {}\n", function_name!());
            let msg2 = format!("{mm} is not a valid minute : must be between 0 and 60\n");
            let msg3 = format!("failure location: {cursor:?}");
            return Err(format!("{}{}{}", msg, msg2, msg3))
        }

        // get second
        // let ss : u8 = remainder.to_string().parse().unwrap();
        res.assert_residual()?;
        let residual = &res.residual[1..]; // chop of colon which is at index 0
        let ss : u8 = residual.to_string()
                        .parse()
                        .map_err(|_| "failed to parse".to_string())?;

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

    // unfortunately, the minutes, seconds, and hour could occur in an 
    // unexpected order
    let full_date = format!("{day} {month} {date} {hh}:{mm}:{ss} {year}");
    let full_date = Utc.datetime_from_str(full_date.as_str(), "%a %b %e %T %Y");
    if full_date.is_ok() {
        return Ok(full_date.unwrap())
    }

    Err("failed to parse date".to_string())

}

#[named]
pub(super) fn parse_version(word_reader : &mut WordReader) -> Result<Version, String> {
    let mut version = String::new();

    loop {
        let word = word_reader.next_word();

        // if there isn't another word left in the file, then we exit
        if word.is_none() {
            return Err(format!("reached end of file without parser leaving {}", function_name!()))
        }

        let (word, cursor) = word.unwrap();

        if word == "$end" {
            // truncate trailing whitespace
            let version = version[0..(version.len() -  1)].to_string();
            return Ok(Version(version))

        }
        else {
            version.push_str(word);
            version.push_str(" ");
        }
    }
}

#[named]
pub(super) fn parse_timescale(word_reader : &mut WordReader) -> Result<(Option<u32>, Timescale), String> {
    let err_msg = format!("failed in {}", function_name!());

    // we might see `scalarunit $end` or `scalar unit $end`

    // first get timescale
    let (word, cursor) = word_reader.next_word().ok_or(&err_msg)?;
    let word = word.to_string();
    let ParseResult{matched, residual} = take_while(word.as_str(), digit);
    let scalar = matched;

    let scalar : u32 = scalar.to_string().parse()
                        .map_err(|_| &err_msg)?;

    let timescale = {
        if residual == "" {
            let (word, cursor) = word_reader.next_word().ok_or(&err_msg)?;
            let unit = match word {
                "fs" => {Ok(Timescale::fs)}
                "ps" => {Ok(Timescale::ps)}
                "ns" => {Ok(Timescale::ns)}
                "us" => {Ok(Timescale::us)}
                "ms" => {Ok(Timescale::ms)}
                "s"  => {Ok(Timescale::s)}
                _    => {Err(err_msg.to_string())}
            }.unwrap();
        
            (Some(scalar), unit)
        }
        else {
            let unit = match residual {
                "ps" => {Ok(Timescale::ps)}
                "ns" => {Ok(Timescale::ns)}
                "us" => {Ok(Timescale::us)}
                "ms" => {Ok(Timescale::ms)}
                "s"  => {Ok(Timescale::s)}
                _    => {Err(err_msg.to_string())}
            }.unwrap();
        
            (Some(scalar), unit)
        }
    };

    // then check for the `$end` keyword
    let (end, cursor) = word_reader.next_word().ok_or(&err_msg)?;
    tag(end, "$end").assert_match()?;

    return Ok(timescale);

    Err("".to_string())
}

#[named]
pub(super) fn parse_metadata(word_reader : &mut WordReader) -> Result<Metadata, String> {
    let err_msg = format!("reached end of file without parser leaving {}", function_name!());

    let mut metadata = Metadata {
        date : None,
        version : None,
        timescale : (None, Timescale::unit)
    };

    loop {
        // check for another word in the file
        let (word, cursor) = word_reader.next_word().ok_or(&err_msg)?;

        let ParseResult{matched, residual} = tag(word, "$");
        match matched {
            // we hope that this word stars with a `$`
            "$" =>  {
                match residual {
                    "date"      => {
                        let err_msg = format!("reached end of file without parser leaving {}", function_name!());
                        // a date is typically composed of the 5 following words which can 
                        // occur in any order: 
                        // {Day, Month, Date(number in month), hh:mm:ss, year}.
                        // Thus, we must lookahead read the 5 next words, and try our date
                        // parser on 5! = 120 permutations of the 5 words.
                        //
                        // It is also possible that within each permutation, the hours,
                        // minutes, and seconds could be in an unusual order, which means
                        // that we may search up to 6 different permutations oh hh::mm:ss,
                        // for an upper bound total of 720 permutations
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

                        let permutations =  lookahead_5_words
                                            .iter()
                                            .permutations(lookahead_5_words.len());
                        
                        // go ahead and search for a match amongst permuted date text
                        for mut permutations in permutations {
                            let (w1, s1) = permutations.pop().unwrap();
                            let arg_1 = (&w1[..], s1);

                            let (w2, s2) = permutations.pop().unwrap();
                            let arg_2 = (&w2[..], s2);

                            let (w3, s3) = permutations.pop().unwrap();
                            let arg_3 = (&w3[..], s3);

                            let (w4, s4) = permutations.pop().unwrap();
                            let arg_4 = (&w4[..], s4);

                            let (w5, s5) = permutations.pop().unwrap();
                            let arg_5 = (&w5[..], s5);

                            let parsed_date = parse_date(arg_1, arg_2, arg_3, arg_4, arg_5);

                            // store date and exit loop if a match is found
                            if parsed_date.is_ok() {
                                metadata.date = Some(parsed_date.unwrap());
                                break
                            }

                        }
                    }
                    "version"   => {
                        let version = parse_version(word_reader);
                        if version.is_ok() {
                            metadata.version = Some(version.unwrap());
                        }
                    }
                    "timescale" => {
                        let timescale = parse_timescale(word_reader);
                        if timescale.is_ok() {
                            metadata.timescale = timescale.unwrap();
                        }
                    }
                    // in VCDs, the scope keyword indicates the end of the metadata section
                    "scope"     => {break}
                    // we keep searching for words until we've found one of the following
                    // keywords, ["version", "timescale", "scope"]
                    _ => {}
                }
            }
            // if word does not start with `$`, then we keep looping
            _ => {}
        }

    }
    return Ok(metadata)
}