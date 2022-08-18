use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::slice;
use std::str;

#[derive(Debug, Clone)]
pub(super) struct Line(pub(super) usize);
#[derive(Debug, Clone)]
pub(super) struct Word(pub(super) usize);
#[derive(Debug, Clone)]
pub(super) struct Cursor(pub(super) Line, pub(super) Word);

pub(super) struct WordReader {
    reader: io::BufReader<File>,
    eof: bool,
    buffers: Vec<String>,
    curr_line: usize,
    str_slices: VecDeque<(*const u8, usize, Cursor)>,
    curr_slice: Option<(*const u8, usize, Cursor)>,
}

impl WordReader {
    pub(super) fn new(file: File) -> WordReader {
        let reader = io::BufReader::new(file);
        WordReader {
            reader: reader,
            eof: false,
            buffers: vec![],
            curr_line: 0,
            str_slices: VecDeque::new(),
            curr_slice: None,
        }
    }

    pub(super) fn next_word(&mut self) -> Option<(&str, Cursor)> {
        // although reaching the eof is not technically an error, in most cases,
        // we treat it like one in the rest of the codebase.

        // if there are no more words in the buffer, attempt to read more content
        // from the file
        if self.str_slices.is_empty() {
            self.buffers.clear();

            if self.eof {
                return None;
            }

            let num_buffers = 10;

            for buf_idx in 0..num_buffers {
                self.buffers.push(String::new());
                self.curr_line += 1;
                let bytes_read = self.reader.read_line(&mut self.buffers[buf_idx]).unwrap();

                // if we've reached the end of the file on the first attempt to read
                // a line in this for loop, no further attempts are necessary and we
                if bytes_read == 0 {
                    self.eof = true;
                    break;
                }

                let words = self.buffers[buf_idx].split_ascii_whitespace();

                for word in words.enumerate() {
                    let (word_idx, word) = word;
                    let position = Cursor(Line(self.curr_line), Word(word_idx + 1));
                    self.str_slices
                        .push_back((word.as_ptr(), word.len(), position))
                }
            }
        }

        // if after we've attempted to read in more content from the file,
        // there are still no words...
        if self.str_slices.is_empty() {
            return None;
        }

        // if we make it here, we return the next word
        unsafe {
            let (ptr, len, position) = self.str_slices.pop_front().unwrap();
            let slice = slice::from_raw_parts(ptr, len);
            self.curr_slice = Some((ptr, len, position.clone()));
            return Some((str::from_utf8(slice).unwrap(), position));
        };
    }

    pub(super) fn curr_word(&mut self) -> Option<(&str, Cursor)> {
        match &self.curr_slice {
            Some(slice) => unsafe {
                let (ptr, len, position) = slice.clone();
                let slice = slice::from_raw_parts(ptr, len);
                return Some((str::from_utf8(slice).unwrap(), position));
            },
            None => None,
        }
    }
}

macro_rules! next_word {
    ($word_reader:ident) => {
        $word_reader.next_word().ok_or(()).map_err(|_| {
            format!(
                "Error near {}:{}. Did not expect to reach end of file here.",
                file!(),
                line!()
            )
        })
    };
}

macro_rules! curr_word {
    ($word_reader:ident) => {
        $word_reader.curr_word().ok_or(()).map_err(|_| {
            format!(
                "Error near {}:{}. A call to curr_word! shouldn't fail unless next_word has not yet been invoked.",
                file!(),
                line!()
            )
        })
    };
}

pub(super) use curr_word;
pub(super) use next_word;
