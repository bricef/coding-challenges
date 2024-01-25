
use clap::{Arg, ArgAction};
use clap::Command;
use huffman::HuffmanEncoding;

use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter, BufRead};

mod huffman;


const MAGIC: [u8; 5] = [b'P', b'R', b'E', b'S', b'S'];

fn compress(input: &Vec<u8>, output: &mut Vec<u8>) {
    let encoding = HuffmanEncoding::from_data_vec(input);
    let mut filestream = encoding.encode(input);
    let mut code_table = encoding.save();
    let mut tlen = u32_to_u8s(code_table.len() as u32);
    
    output.append(&mut Vec::from(MAGIC));
    output.append(&mut tlen);
    output.append(&mut code_table);
    output.append(&mut filestream);   
}

fn decompress(input: &Vec<u8>, output: &mut Vec<u8>) {
    if input[0..MAGIC.len()] != MAGIC {
        panic!("File is not in press format.");
    }

    let table_length = u8s_to_u32(&Vec::from(&input[MAGIC.len()..MAGIC.len()+4])) as usize;
    let table_raw = &input[MAGIC.len()+4..table_length+MAGIC.len()+4];
    let data_raw = &input[table_length+MAGIC.len()+4..];

    let encoding = HuffmanEncoding::restore_from(&Vec::from(table_raw));

    assert!(encoding.encoding.contains_key(&huffman::Symbol::EOT));
    let mut data = encoding.decode(&Vec::from(data_raw));
    output.append(&mut data);
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
        .arg(Arg::new("compress")
            .short('c')
            .long("Compress input file to output file.")
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
   
    if matches.get_flag("decompress") {
        decompress(&in_buf, &mut out_buf);
    }else if matches.get_flag("compress"){
        compress(&in_buf, &mut out_buf);
    }else{
        panic!("Must specify either compression or decompression. See --help option.");
    }

    output.write(&out_buf)?;

    // eprintln!("in: {}, out: {}, ratio: {}", in_buf.len(), out_buf.len(), out_buf.len() as f64/in_buf.len() as f64);

    return Ok(());
}


fn u32_to_u8s(i: u32) -> Vec<u8> {
    Vec::from([
        (i & 0xff000000 >> 24) as u8,
        (i & 0x00ff0000 >> 16) as u8,
        (i & 0x0000ff00 >> 8) as u8,
        (i & 0x000000ff) as u8,
    ]) 
}

fn u8s_to_u32(us: &Vec<u8>) -> u32 {
    let mut out:u32 = 0;
    out |= (us[0] as u32) & 0x000000ff << 24;
    out |= (us[1] as u32) & 0x000000ff << 16;
    out |= (us[2] as u32) & 0x000000ff << 8;
    out |= us[3] as u32 & 0x000000ff;
    return out;
}

#[cfg(test)]
mod tests {

    use super::*;

    fn _get_tlen(input: &Vec<u8>) -> u32 {
        if input[0..MAGIC.len()] != MAGIC {
            panic!("File is not in press format.");
        }
        u8s_to_u32(&Vec::from(&input[3..7]))
    }

    #[test]
    fn can_encode_decode_u32(){
        for i in 0..256 {
            let xs =u32_to_u8s(i);
            let y = u8s_to_u32(&xs);
            assert_eq!(i,y);
        }
        
    }

    #[test]
    fn can_get_encoding_table_length() {
        let in_buf: Vec<u8> = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi condimentum gravida libero non mollis. Mauris turpis sapien, interdum non tortor id, sollicitudin lobortis mi. Nam sit amet tellus vehicula, condimentum.".to_vec();
        let mut out_buf: Vec<u8> = Vec::new();
        
        let encoding_original = HuffmanEncoding::from_data_vec(&in_buf);
        let saved = encoding_original.save();
        let expected_length = saved.len() as u32;

        compress(&in_buf, &mut out_buf);

        assert_eq!(_get_tlen(&out_buf), expected_length)
    }

    #[test]
    fn encoding_will_always_have_eot(){
        let in_buf: Vec<u8> = b"".to_vec();
        let encoding = HuffmanEncoding::from_data_vec(&in_buf);

        assert!(encoding.encoding.contains_key(&huffman::Symbol::EOT));
    }

    #[test]
    fn decoded_table_has_eot(){
        let in_buf: Vec<u8> = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi condimentum gravida libero non mollis. Mauris turpis sapien, interdum non tortor id, sollicitudin lobortis mi. Nam sit amet tellus vehicula, condimentum.".to_vec();
        
        let encoding = HuffmanEncoding::from_data_vec(&in_buf);
        let saved = encoding.save();
        let restored = HuffmanEncoding::restore_from(&saved);

        assert!(restored.encoding.contains_key(&huffman::Symbol::EOT));
    }

    #[test]
    fn can_compress_decompress(){

        let in_buf: Vec<u8> = b"Hello World".to_vec();
        let mut compressed_buf: Vec<u8> = Vec::new();
        let mut out_buf: Vec<u8> = Vec::new();
        
        compress(&in_buf, &mut compressed_buf);
        println!("COMPRESSED...");
        decompress(&compressed_buf, &mut out_buf);

        assert_eq!(in_buf, out_buf);

    }

    #[test]
    fn test_encoding_serialise_deserialise() {
        let in_buf: Vec<u8> = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi condimentum gravida libero non mollis. Mauris turpis sapien, interdum non tortor id, sollicitudin lobortis mi. Nam sit amet tellus vehicula, condimentum.".to_vec();
        
        let encoding = HuffmanEncoding::from_data_vec(&in_buf);
        let saved = encoding.save();
        let restored = HuffmanEncoding::restore_from(&saved);

        assert_eq!(encoding, restored);
    }

    #[test]
    fn sanity_check() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}