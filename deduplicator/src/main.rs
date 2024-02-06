use clap::Command;
use clap::{Arg, ArgAction};
use walkdir::WalkDir;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use crc::{Crc, CRC_32_CKSUM};
use xxhash_rust::xxh3::xxh3_64;
use dialoguer::Select;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Candidate{
    path: PathBuf,
    size: u64,
    crc: u32,
    hash: u64,
}

const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);


fn candidates(directory: String) -> Result<Vec<Candidate>, std::io::Error>{
    let mut result: Vec<Candidate> = vec![];
    let walker = WalkDir::new(directory).into_iter();
    for entry in walker {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let meta = entry.metadata()?;
            let c = Candidate {
                size: meta.len(),
                path: entry.path().to_owned(),
                crc: 0,
                hash: 0,
            };
            result.push(c);
        }
    }
    Ok(result)
}

fn size_matches(cs: Vec<Candidate>) -> Result<Vec<Vec<Candidate>>, std::io::Error> {
    let mut index: HashMap<u64, Vec<Candidate>> = HashMap::new();
    
    // make index
    for c in cs.into_iter() {
        index
            .entry(c.size)
            .and_modify(|v| v.push(c.clone()))
            .or_insert(vec![c]);
    }

    let output = index.into_values().collect();

    Ok(output)
}

fn crc_of_path(path: &PathBuf) -> Result<u32, std::io::Error> {
    let mut file = File::open(path)?;
    let mut buf =Vec::new();
    file.read_to_end(&mut buf)?;
    return Ok(CRC.checksum(&buf));
}

fn crc_matches<'a>(cs: Vec<Vec<Candidate>>) -> Result<Vec<Vec<Candidate>>, std::io::Error> {
    let mut output: Vec<Vec<Candidate>> = Vec::new();
    for size_matches in cs{
        let mut index: HashMap<u32, Vec<Candidate>> = HashMap::new();
        for mut c in size_matches {
            let crc = crc_of_path(&c.path)?;
            c.crc = crc;
            index
                .entry(crc)
                .and_modify(|v|{ v.push(c.clone())})
                .or_insert(vec![c]);
        }
        let mut remaining_matches: Vec<Vec<Candidate>> = index
            .into_values()
            .filter(|cs| cs.len() > 1)
            .collect();
        output.append(&mut remaining_matches);
    }
    Ok(output)
}

fn hash_of_path(path: &PathBuf) -> Result<u64, std::io::Error> {
    let mut file = File::open(path)?;
    let mut buf =Vec::new();
    file.read_to_end(&mut buf)?;
    return Ok(xxh3_64(&buf));
}

fn hash_matches<'a>(cs: Vec<Vec<Candidate>>) -> Result<Vec<Vec<Candidate>>, std::io::Error>{
    let mut output: Vec<Vec<Candidate>> = Vec::new();
    for size_matches in cs{
        let mut index: HashMap<u64, Vec<Candidate>> = HashMap::new();
        for mut c in size_matches {
            let hash = hash_of_path(&c.path)?;
            c.hash = hash;
            index
                .entry(hash)
                .and_modify(|v|{ v.push(c.clone()) })
                .or_insert(vec![c]);
        }
        let mut remaining_matches = index
            .into_values()
            .filter(|cs| cs.len() > 1)
            .collect();
        output.append(&mut remaining_matches);
    }
    Ok(output)
}


fn find_duplicates(directory: String) -> Result<Vec<Vec<Candidate>>, std::io::Error> {
    let candidates = candidates(directory)?;
    let s_matches = size_matches(candidates)?;
    let c_matches= crc_matches(s_matches)?;
    let h_matches = hash_matches(c_matches)?;
    Ok(h_matches)
}

fn present_report(dups: &Vec<Vec<Candidate>>) -> Result<(), std::io::Error>{
    for duplicates in dups {
        println!("Found duplicates:");
        for d in duplicates {
            println!("   {}", d.path.to_str().unwrap());
        }
    }
    Ok(())
}

fn autodelete(dups: &Vec<Vec<Candidate>>) -> Result<(), std::io::Error> {
    for duplicates in dups {
        let mut iter = duplicates.iter();
        let _first = iter.next().unwrap();
        for duplicate in iter {
            std::fs::remove_file(&duplicate.path)?;
        }
    }
    Ok(())
}

struct Action {
    label: String,
    action: Box<dyn FnOnce() -> Result<(), std::io::Error>>,
}


fn candidates_to_delete_choices(duplicates: Vec<Candidate>) -> Vec<Action> {
    duplicates
        .iter()
        .map(|c| Action {
            label: c.path.to_str().unwrap().to_string(),
            action: Box::new(|| Ok(()) ),
        })
        .collect()
}

fn prompt_choices(_choices: Vec<Action>) -> Result<(), std::io::Error> {
    let items = vec!["foo", "bar", "baz"];

    let selection = Select::new()
        .with_prompt("What do you choose?")
        .items(&items)
        .interact()
        .unwrap();

    println!("You chose: {}", items[selection]);
    Ok(())
}

fn prompt_delete(dups: &Vec<Vec<Candidate>>) -> Result<(), std::io::Error> {
    for duplicates in dups {
        let local_dups = duplicates.clone();
        let choices = vec![
            Action {
                label: "Ignore".to_string(),
                action: Box::new(|| Ok(())),
            },
            Action {
                label: "Keep one".to_string(),
                action: Box::new(|| {
                    let del_choices = candidates_to_delete_choices(local_dups);
                    prompt_choices(del_choices)
                }),
            },
        ];
        let _action = prompt_choices(choices);
    }
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let matches = Command::new("dedup")
        .version("1.0")
        .author("Brice Fernandes <brice@fractallambda.com>")
        .about("Interactive file deduplicator")
        .arg(
            Arg::new("DIRECTORY")
                .required(true)
                .help("The directory to scan for duplicate files."))
        .arg(
            Arg::new("report")
                .short('r')
                .long("report")
                .help("Report on duplicate files.")
                .action(ArgAction::SetTrue))
        .arg(
            Arg::new("follow-symlinks")
                .short('f')
                .help("Follow symlinks when scanning (False by default).")
                .action(ArgAction::SetTrue))
        .arg(
            Arg::new("autodelete")
                .short('d')
                .long("autodelete")
                .help("Automatically delete duplicate files without prompting.")
                .action(ArgAction::SetTrue))
        .get_matches();

    let dir = matches.get_one::<String>("DIRECTORY").unwrap();
    let duplicates = find_duplicates(dir.clone())?;


    if matches.get_flag("report") {
        present_report(&duplicates)?;
    }
    if matches.get_flag("autodelete") {
        autodelete(&duplicates)?;
    } else {
        prompt_delete(&duplicates)?;
    }
    if matches.get_flag("follow-symlinks") {
        todo!()
    }

    return Ok(());
}