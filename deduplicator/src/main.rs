use clap::Command;
use clap::{Arg, ArgAction};
use crc::{Crc, CRC_32_CKSUM};
use dialoguer::Select;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;
use xxhash_rust::xxh3::xxh3_64;
const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);

fn candidates(walker: WalkDir) -> Result<Vec<Vec<PathBuf>>, std::io::Error> {
    let mut result: Vec<PathBuf> = Vec::new();
    for entry in walker {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            result.push(entry.path().to_owned());
        }
    }
    Ok(vec![result])
}

fn size_of_path(c: &PathBuf) -> Result<u64, std::io::Error> {
    let meta = std::fs::metadata(&c)?;
    return Ok(meta.len());
}

fn crc_of_path(c: &PathBuf) -> Result<u32, std::io::Error> {
    let mut file = File::open(&c)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    return Ok(CRC.checksum(&buf));
}

fn hash_of_path(c: &PathBuf) -> Result<u64, std::io::Error> {
    let mut file = File::open(&c)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    return Ok(xxh3_64(&buf));
}

fn matches<T>(
    matcher: Box<dyn Fn(&PathBuf) -> Result<T, std::io::Error>>,
    cs: Vec<Vec<PathBuf>>,
) -> Result<Vec<Vec<PathBuf>>, std::io::Error>
where
    T: Hash + Eq + PartialEq,
{
    let mut output: Vec<Vec<PathBuf>> = Vec::new();
    for size_matches in cs {
        let mut index: HashMap<T, Vec<PathBuf>> = HashMap::new();
        for c in size_matches {
            let hash = matcher(&c)?;
            index
                .entry(hash)
                .and_modify(|v| v.push(c.clone()))
                .or_insert(vec![c]);
        }
        let mut remaining_matches = index.into_values().filter(|cs| cs.len() > 1).collect();
        output.append(&mut remaining_matches);
    }
    Ok(output)
}

fn find_duplicates(walker: WalkDir) -> Result<Vec<Vec<PathBuf>>, std::io::Error> {
    let candidates = candidates(walker)?;
    let s_matches = matches(Box::new(size_of_path), candidates)?;
    let c_matches = matches(Box::new(crc_of_path), s_matches)?;
    let h_matches = matches(Box::new(hash_of_path), c_matches)?;
    Ok(h_matches)
}

fn present_report(dups: &Vec<Vec<PathBuf>>) -> Result<(), std::io::Error> {
    for duplicates in dups {
        println!("Found duplicates:");
        for d in duplicates {
            println!("   {}", d.to_str().unwrap());
        }
    }
    Ok(())
}

fn autodelete(dups: &Vec<Vec<PathBuf>>) -> Result<(), std::io::Error> {
    for duplicates in dups {
        let mut iter = duplicates.iter();
        let _first = iter.next().unwrap();
        for duplicate in iter {
            std::fs::remove_file(&duplicate)?;
        }
    }
    Ok(())
}

#[derive(Clone)]
enum Action {
    Ignore,
    Keep(PathBuf),
    DeleteAll,
}

struct Choice {
    label: String,
    action: Action,
}

fn prompt_choices(choices: Vec<Choice>) -> Result<Action, std::io::Error> {
    let items: Vec<String> = choices.iter().map(|a| a.label.clone()).collect();

    let selection = Select::new()
        .with_prompt("Choose an action")
        .items(&items)
        .interact()
        .unwrap();

    Ok(choices[selection].action.clone())
}

fn handle_duplicates(duplicates: &Vec<PathBuf>) -> Result<(), std::io::Error> {
    println!("Duplicate found!");
    for d in duplicates {
        println!("   {}", d.to_str().unwrap());
    }
    // Set up choices...
    let mut choices: Vec<Choice> = Vec::new();
    choices.push(Choice {
        label: "Ignore".to_string(),
        action: Action::Ignore,
    });
    for c in duplicates {
        choices.push(Choice {
            label: format!("Keep {:#?}", c),
            action: Action::Keep(c.clone()),
        });
    }
    choices.push(Choice {
        label: "Delete all duplicates".to_string(),
        action: Action::DeleteAll,
    });

    // prompt user for choice
    let chosen = prompt_choices(choices)?;

    // Act on user choice
    return match chosen {
        Action::Ignore => Ok(()),
        Action::Keep(path) => {
            for d in duplicates {
                if d != &path {
                    std::fs::remove_file(&d)?;
                }
            }
            return Ok(());
        }
        Action::DeleteAll => {
            for d in duplicates {
                std::fs::remove_file(&d)?;
            }
            return Ok(());
        }
    };
}

fn prompt_delete_all(dups: &Vec<Vec<PathBuf>>) -> Result<(), std::io::Error> {
    for duplicates in dups {
        handle_duplicates(duplicates)?;
        println!("");
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
                .help("The directory to scan for duplicate files."),
        )
        .arg(
            Arg::new("report")
                .short('r')
                .long("report")
                .help("Report on duplicate files.")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("follow-symlinks")
                .short('f')
                .long("follow-symlinks")
                .help("Follow symlinks when scanning (False by default).")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("autodelete")
                .short('d')
                .long("autodelete")
                .help("Automatically delete duplicate files without prompting.")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let dir = matches.get_one::<String>("DIRECTORY").unwrap();

    let walker = match matches.get_flag("follow-symlinks") {
        true => WalkDir::new(dir.clone()),
        false => WalkDir::new(dir.clone()).follow_links(false),
    };

    let duplicates = find_duplicates(walker)?;

    if matches.get_flag("report") {
        present_report(&duplicates)?;
        return Ok(());
    }

    if matches.get_flag("autodelete") {
        autodelete(&duplicates)?;
    } else {
        prompt_delete_all(&duplicates)?;
    }

    return Ok(());
}
