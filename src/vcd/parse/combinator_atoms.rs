use super::types::ParseResult;
use super::reader::WordReader;

pub(super) fn digit(chr : u8) -> bool {
    let zero = b'0' as u8;
    let nine = b'9' as u8;

    let between_zero_and_nine = (chr >= zero) && (nine >= chr);

    return between_zero_and_nine
}

pub(super) fn take_until<'a>(word : &'a str, pattern : u8) -> ParseResult<'a> {
    let mut new_start  = 0;

    for chr in word.as_bytes() {
        if *chr == pattern {
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

pub(super) fn take_while<'a>(word : &'a str, cond : fn(u8) -> bool) -> ParseResult<'a> {
    let mut new_start  = 0;

    for chr in word.as_bytes() {
        if cond(*chr) {
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

pub(super) fn tag<'a>(word : &'a str, pattern : &'a str) -> ParseResult<'a> {
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

pub(super) fn ident(
    word_reader : &mut WordReader,
    keyword : &str, 
) -> Result<(), String> {
    // let keyword = "module";

    let err : Result<(), String> = Err(format!("reached end of file without parser leaving ident"));
    let word = word_reader.next_word();
    let (word, cursor) = word.ok_or(err).unwrap();

    if word == keyword {
        return Ok(())
    }
    else {
        let err = format!("found keyword `{word}` but expected `{keyword}` on {cursor:?}");
        return Err(err)
    }
}