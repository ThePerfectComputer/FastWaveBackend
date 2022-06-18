use super::*;
use chrono::prelude::*;
use itertools::Itertools;
use std::fs::File;
use ::function_name::named;

#[derive(Debug)]
pub struct Residual<'a>(&'a str);
#[derive(Debug)]
pub struct ParseResult<'a> {matched : &'a str, residual : &'a str}

pub fn digit(chr : u8) -> bool {
    let zero = b'0' as u8;
    let nine = b'9' as u8;

    let between_zero_and_nine = (chr >= zero) && (nine >= chr);

    return between_zero_and_nine
}

pub fn take_until<'a>(word : &'a str, pattern : u8) -> ParseResult<'a> {
    let mut new_start  = 0;

    for chr in word.as_bytes() {
        if (*chr == pattern) {
            break
        } 
        else {
            new_start += 1;
        }
    }

    return 
        ParseResult{
            matched  : &word[0..new_start],
            residual : &word[new_start..]
        };

}

pub fn take_while<'a>(word : &'a str, cond : fn(u8) -> bool) -> ParseResult<'a> {
    let mut new_start  = 0;

    for chr in word.as_bytes() {
        if (cond(*chr)) {
            new_start += 1;
        } 
        else {
            break
        }
    }

    return 
        ParseResult{
            matched  : &word[0..new_start],
            residual : &word[new_start..]
        };

}

fn tag<'a>(word : &'a str, pattern : &'a str) -> ParseResult<'a> {
    let lhs           = word.as_bytes().iter();
    let rhs           = pattern.as_bytes();
    let iter          = lhs.zip(rhs);
    let mut new_start = 0;

    let mut res = true;
    for (c_lhs, c_rhs) in iter {
        res = res && (c_lhs == c_rhs);
        if !res {break}
        new_start += 1;
    }

    return 
        ParseResult{
            matched  : &word[0..new_start],
            residual : &word[new_start..]
        };
}

impl<'a> ParseResult<'a> {
    fn match_not_empty(& self) -> Result<(), String> {
        if self.matched == "" {
            return Err("failed".to_string())
        }
        else {
            return Ok(())
        }
    }

    fn assert_match(& self) -> Result<&str, String> {
        if self.matched == "" {
            return Err("no match".to_string())
        }
        else {
            return Ok(self.matched)
        }
    }

    fn assert_residual(& self) -> Result<&str, String> {
        if self.residual == "" {
            return Err("no residual".to_string())
        }
        else {
            return Ok(self.residual)
        }
    }
}

#[named]
fn parse_date(
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
fn parse_version(word_reader : &mut WordReader) -> Result<Version, String> {
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
fn parse_timescale(word_reader : &mut WordReader) -> Result<(Option<u32>, Timescale), String> {
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
    tag(end, "$end").match_not_empty()?;

    return Ok(timescale);

    Err("".to_string())
}

#[named]
fn parse_metadata(word_reader : &mut WordReader) -> Result<Metadata, String> {
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

pub fn parse_vcd(file : File) {
    let mut word_gen = WordReader::new(file);

    let header = parse_metadata(&mut word_gen).unwrap();
    dbg!(header);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    #[test]
    fn headers() {
        let files = vec![
            "./test-vcd-files/aldec/SPI_Write.vcd",
            "./test-vcd-files/ghdl/alu.vcd",
            "./test-vcd-files/ghdl/idea.vcd",
            "./test-vcd-files/ghdl/pcpu.vcd",
            "./test-vcd-files/gtkwave-analyzer/perm_current.vcd",
            "./test-vcd-files/icarus/CPU.vcd",
            "./test-vcd-files/icarus/rv32_soc_TB.vcd",
            "./test-vcd-files/icarus/test1.vcd",
            "./test-vcd-files/model-sim/CPU_Design.msim.vcd",
            "./test-vcd-files/model-sim/clkdiv2n_tb.vcd",
            "./test-vcd-files/my-hdl/Simple_Memory.vcd",
            "./test-vcd-files/my-hdl/sigmoid_tb.vcd",
            "./test-vcd-files/my-hdl/top.vcd",
            // "./test-vcd-files/ncsim/ffdiv_32bit_tb.vcd",
            // "./test-vcd-files/quartus/mipsHardware.vcd",
            // "./test-vcd-files/quartus/wave_registradores.vcd",
            "./test-vcd-files/questa-sim/dump.vcd",
            "./test-vcd-files/questa-sim/test.vcd",
            "./test-vcd-files/riviera-pro/dump.vcd",
            // "./test-vcd-files/systemc/waveform.vcd",
            // "./test-vcd-files/treadle/GCD.vcd",
            "./test-vcd-files/vcs/Apb_slave_uvm_new.vcd",
            "./test-vcd-files/vcs/datapath_log.vcd",
            "./test-vcd-files/vcs/processor.vcd",
            "./test-vcd-files/verilator/swerv1.vcd",
            "./test-vcd-files/verilator/vlt_dump.vcd",
            // "./test-vcd-files/vivado/iladata.vcd",
            "./test-vcd-files/xilinx_isim/test.vcd",
            "./test-vcd-files/xilinx_isim/test1.vcd",
            "./test-vcd-files/xilinx_isim/test2x2_regex22_string1.vcd"
        ];

        for file in files {
            let metadata = parse_metadata(
                &mut WordReader::new(
                    File::open(file)
                    .unwrap()
                )
            );
            assert!(metadata.is_ok());
            assert!(metadata.unwrap().date.is_some());
        }

    }
}