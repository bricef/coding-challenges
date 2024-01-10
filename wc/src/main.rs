use clap::{ArgAction, Arg, ArgMatches};
use clap::Command;
use std::fs::File;
use std::io::prelude::*;
use std::panic;

enum ParseState {
    Whitespace,
    Wordspace
}

struct Counts {
    bytes: u64,
    lines: u64,
    words: u64
}

fn count(file: impl Read) -> Counts {
    let mut counts = Counts {
        bytes: 0,
        lines: 0,
        words: 0
    };

    let mut current = ParseState::Whitespace;

    for c in file.bytes(){
        let b = c.unwrap_or_else(|err| {
            panic!("Error reading file {}", err)
        });

        counts.bytes += 1;

        if b == b'\n' {
            counts.lines += 1
        }

        if b.is_ascii_whitespace() {
            match current {
                ParseState::Whitespace => (),
                ParseState::Wordspace => ()
            }
            current = ParseState::Whitespace;
        }else {
            match current {
                ParseState::Whitespace => counts.words += 1,
                ParseState::Wordspace => ()
            }
            current = ParseState::Wordspace
        }
        
    }

    return counts;

    
}

fn display(filename: &str, counts: &Counts, options: &ArgMatches) {
    let mut show_all = false;

    if !options.get_flag("bytes")
        && !options.get_flag("lines")
        && !options.get_flag("words") { show_all = true }

    let mut out: Vec<String> = vec![];

    if  options.get_flag("bytes") || show_all{
        out.push(format!("bytes: {}", counts.bytes));
    }
    if options.get_flag("lines") || show_all{
        out.push(format!("lines: {}", counts.lines));
    }
    if options.get_flag("words") || show_all{
        out.push(format!("words: {}", counts.words));
    }

    print!("{}", out.join(" "));
    
    println!(" for {}", filename);
}

fn main() {
    panic::set_hook(Box::new(|info| {
        eprintln!("{info}");
    }));

    let matches = Command::new("WC")
        .version("1.0")
        .author("Brice Fernandes <brice@fractallambda.com>")
        .about("Counts bytes, lines, and words in its input.")
        .arg(Arg::new("FILE")
            .help("Input file"))  
        .arg(Arg::new("bytes")
                .short('c')
                .long("bytes")
                .help("Count bytes")
                .action(ArgAction::SetTrue))
        .arg(Arg::new("lines")
                .short('l')
                .long("lines")
                .help("Count lines")
                .action(ArgAction::SetTrue))
        .arg(Arg::new("words")
                .short('w')
                .long("words")
                .help("Count words")
                .action(ArgAction::SetTrue))
        .get_matches();

    if atty::is(atty::Stream::Stdin) {
        // Nothing piped in...
        if let Some(filename) = matches.get_one::<String>("FILE") {
            let file = File::open(filename)
                .unwrap_or_else(|_| panic!("Could not open file {}", filename));
            let counts = count(&file);
            display(filename, &counts, &matches);
        }    
    }else{
        let counts = count(std::io::stdin().lock());
        display("stdin", &counts, &matches);
    }


}
