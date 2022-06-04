use super::*;
use std::fs::File;
use ::function_name::named;

#[derive(Debug)]
pub struct Residual<'a>(&'a str);

pub fn take_until<'a>(word : &'a str, pattern : u8) -> Option<(&'a str, Residual)> {
    let mut new_start  = 0;

    for chr in word.as_bytes() {
        if (*chr == pattern) {
            return Some((&word[0..new_start], Residual(&word[new_start..])));
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
fn parse_date(word_reader : &mut WordReader) -> Result<(), String> {
    let mut parsed_day   = false;
    let mut parsed_month = false;
    let mut parsed_date  = false;
    let mut parsed_hh    = false;
    let mut parsed_mm    = false;
    let mut parsed_ss    = false;
    let mut parsed_year  = false;
    let mut parsed_end   = false;

    let day = {
        // check for another word in the file
        let (word, cursor) = word_reader.next_word().expect(
            format!("reached end of file without parser leaving {}", function_name!()).as_str()
        );
    
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
        let (word, cursor) = word_reader.next_word().expect(
            format!("reached end of file without parser leaving {}", function_name!()).as_str()
        );

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
        let (word, cursor) = word_reader.next_word().expect(
            format!("reached end of file without parser leaving {}", function_name!()).as_str()
        );

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
        // check for another word in the file
        let (word, cursor) = word_reader.next_word().expect(
            format!("reached end of file without parser leaving {}", function_name!()).as_str()
        );

        let date : u8 = word.to_string().parse().unwrap();
        // let hh = take_until(word, b':').unwrap();

        if date > 31 {
            let msg  = format!("reached end of file without parser leaving {}\n", function_name!());
            let msg2 = format!("{word} is not a valid date : must be between 0 and 31\n");
            let msg3 = format!("failure location: {cursor:?}");
            return Err(format!("{}{}{}", msg, msg2, msg3))

        }
        ("", "", "")
    };

    // else if !parsed_date {

    // }
    // else if !parsed_hh {

    // }
    // else if !parsed_mm {

    // }
    // else if !parsed_ss {

    // }
    // else if !parsed_year {

    // }
    // else if !parsed_end {

    // }

    Ok(())
}

#[named]
fn parse_header(word_reader : &mut WordReader) -> Result<(), String> {
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
                    "date"      => {println!("got date")}
                    "version"   => {println!("got version")}
                    "timescale" => {println!("got timescale")}
                    "scope"     => {return Ok(())}
                    _ => {}
                }
            }
            // if not, then we keep looping
            None => {}
        }


    }
    // Ok()
}

pub fn parse_vcd(file : File) {
    let mut word_gen = WordReader::new(file);

    parse_header(&mut word_gen);
}