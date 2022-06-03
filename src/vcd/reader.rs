use super::*;

use std::fs::File;
use std::collections::VecDeque;
use std::slice;
use std::str;
use std::io::prelude::*;
use std::io;

#[derive(Debug)]
struct Line(usize);
#[derive(Debug)]
struct Word(usize);
#[derive(Debug)]
pub struct Cursor(Line, Word);

pub struct WordReader {
    reader       : io::BufReader<File>,
    EOF          : bool,
    buffers      : Vec<String>,
    curr_line    : usize,
    str_slices   : VecDeque<(*const u8, usize, Cursor)>,
}

impl WordReader {
    pub fn new(file : File) -> WordReader {
        let mut reader = io::BufReader::new(file);
        WordReader {
            reader       : reader,
            EOF          : false,
            buffers      : vec![],
            curr_line    : 0,
            str_slices   : VecDeque::new()
        }
    }

    pub fn next_word(&mut self) -> Option<(&str, Cursor)> {
        // if there are no more words, attempt to read more content
        // from the file
        if self.str_slices.is_empty() {
            self.buffers.clear();

            if self.EOF {return None}

            let num_buffers = 10;

            for buf_idx in 0..num_buffers {
                self.buffers.push(String::new());
                self.curr_line += 1;
                let bytes_read = self.reader.read_line(&mut self.buffers[buf_idx]).unwrap();

                // if we've reached the end of the file on the first attempt to read
                // a line in this for loop, no further attempts are necessary and we
                if bytes_read == 0 {
                    self.EOF = true; 
                    break;
                }

                let mut words = self.buffers[buf_idx].split_ascii_whitespace();
                
                for word in words.enumerate() {
                    let (word_idx, word) = word;
                    let position = Cursor(Line(self.curr_line), Word(word_idx + 1));
                    self.str_slices.push_back((word.as_ptr(), word.len(), position))
                }

            }
        }

        // if after we've attempted to read in more content from the file,
        // there are still no words...
        if self.str_slices.is_empty() {
            return None
        }

        // if we make it here, we return the next word
        unsafe {
            let (ptr, len, position) = self.str_slices.pop_front().unwrap();
            let slice = slice::from_raw_parts(ptr, len);
            return Some((str::from_utf8(slice).unwrap(), position));
        };
    }
}