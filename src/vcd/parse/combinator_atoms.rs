// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.
use super::super::reader::{next_word, WordReader};
use super::types::ParseResult;

pub(super) fn digit(chr: u8) -> bool {
    let zero = b'0';
    let nine = b'9';

    (chr >= zero) && (nine >= chr)
}

pub(super) fn take_until(word: &str, pattern: u8) -> ParseResult<'_> {
    let mut new_start = 0;

    for chr in word.as_bytes() {
        if *chr == pattern {
            break;
        } else {
            new_start += 1;
        }
    }

    ParseResult {
        matched: &word[0..new_start],
        residual: &word[new_start..],
    }
}

pub(super) fn take_while(word: &str, cond: fn(u8) -> bool) -> ParseResult<'_> {
    let mut new_start = 0;

    for chr in word.as_bytes() {
        if cond(*chr) {
            new_start += 1;
        } else {
            break;
        }
    }

    ParseResult {
        matched: &word[0..new_start],
        residual: &word[new_start..],
    }
}

pub(super) fn tag<'a>(word: &'a str, pattern: &'a str) -> ParseResult<'a> {
    let lhs = word.as_bytes().iter();
    let rhs = pattern.as_bytes();
    let iter = lhs.zip(rhs);
    let mut new_start = 0;

    let mut res = true;
    for (c_lhs, c_rhs) in iter {
        res = res && (c_lhs == c_rhs);
        if !res {
            break;
        }
        new_start += 1;
    }

    ParseResult {
        matched: &word[0..new_start],
        residual: &word[new_start..],
    }
}

pub(super) fn ident<R: std::io::Read>(
    word_reader: &mut WordReader<R>,
    keyword: &str,
) -> Result<(), String> {
    // let keyword = "module";
    let (word, cursor) = next_word!(word_reader)?;

    if word == keyword {
        Ok(())
    } else {
        let err = format!("found keyword `{word}` but expected `{keyword}` on {cursor:?}");
        Err(err)
    }
}
