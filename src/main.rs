use std::fs::File;
use clap::Parser;

pub mod vcd;
use vcd::*;

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let file           = File::open(&args.path)?;
    dbg!(["hello", "goodbye", "myworld"].contains(&"myworlde"));
    // let mut word_gen   = WordReader::new(file);
    // let mut word_count = 0;

    // while word_gen.next_word().is_some() {
    //     word_count += 1;
    // }
    // dbg!(word_count);

    // let word1 = "hello world";
    // let word2 = "hello planet";
    // dbg!(&word1[0..6].len());
    dbg!(take_until("tea time  now: and later", b':'));
    // parse_vcd(file);

    // tag("my oh my");


    // loop {
    //     let word = word_gen.next_word();
    //     if word.is_none() {break};

    //     dbg!(word.unwrap());
    // }


    Ok(())
}