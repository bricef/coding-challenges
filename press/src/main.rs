
use clap::{Arg, ArgAction};
use clap::Command;
use huffman::HuffmanEncoding;
use postcard::{from_bytes, to_allocvec};
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter, BufRead};
use bimap::BiMap;
use bitvec::prelude::*;
use serde::{Serialize, Deserialize};

mod huffman;

fn compress(input: &Vec<u8>, output: &mut Vec<u8>) {
    let encoding = HuffmanEncoding::from_data_vec(input);
    let filestream = encoding.encode(input);
    
    let mut code_table: Vec<u8> = encoding.into_vec();
    let mut data = filestream.into_vec();

    // let mut data = to_allocvec(&p).unwrap();
    output.append(&mut data)

    
}

fn _decompress(_input: &Vec<u8>, _output: &mut Vec<u8>) {
    ()
}

fn main() -> Result<(), std::io::Error>  {
    let matches = Command::new("Press")
        .version("1.0")
        .author("Brice Fernandes <brice@fractallambda.com>")
        .about("Basic file compressor.")
        .arg(Arg::new("INPUT")
            .help("Input file"))
        .arg(Arg::new("OUTPUT")
            .help("Output file."))
        .arg(Arg::new("decompress")
            .short('d')
            .long("Decompress input file instead of compressing.")
            .action(ArgAction::SetTrue))
        .get_matches();

    let mut input: Box<dyn BufRead>;
    let mut output: Box<dyn Write>;

    if atty::is(atty::Stream::Stdin) {
        if let Some(filename) = matches.get_one::<String>("INPUT") {
            if filename == "-" {
                input = Box::new(BufReader::new(std::io::stdin()));
            } else {
                let file = File::open(filename)
                    .unwrap_or_else(|_| panic!("Could not open file {}", filename));
                input = Box::new(BufReader::new(file));
            }
        } else {
            panic!("Must specify an input file.");
        }
    } else {
        input = Box::new(BufReader::new(std::io::stdin()));
    }

    if atty::is(atty::Stream::Stdout) {
        if let Some(filename) = matches.get_one::<String>("OUTPUT") {
            if filename == "-" {
                output = Box::new(BufWriter::new(std::io::stdout()));
            } else {
                let file = File::open(filename)
                    .unwrap_or_else(|_| panic!("Could not open file {}", filename));
                output = Box::new(file);
            }
        } else {
            panic!("Must specify an output file.");
        }
    } else {
        output = Box::new(BufWriter::new(std::io::stdout()));
    }

    let mut in_buf: Vec<u8> = Vec::new();
    let mut out_buf: Vec<u8> = Vec::new();
    input.read_to_end(&mut in_buf)?;
   
    compress(&in_buf, &mut out_buf);

    output.write(&out_buf)?;


    return Ok(());
}
