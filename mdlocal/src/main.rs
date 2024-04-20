use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
// use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Arg;
use clap::Command;
use markdown::{to_mdast, ParseOptions};
use markdown::mdast;


use url::{Url, ParseError};

// use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum MarkdownParseError {
//     #[error("data store disconnected")]
//     Disconnect(#[from] io::Error),
//     #[error("the data for key `{0}` is not available")]
//     Redaction(String),
//     #[error("invalid header (expected {expected:?}, found {found:?})")]
//     InvalidHeader {
//         expected: String,
//         found: String,
//     },
//     #[error("unknown data store error")]
//     Unknown,
// }

fn main() {
    let matches = Command::new("mdlocal")
        .version("0.1.0")
        .author("Brice Fernandes <brice@fractallambda.com>")
        .about("Markdown Localiser: A tool to localise links to images in Markdown.")
        .arg(Arg::new("FILE").required(true).help("The file to localise"))
        // .arg(
        //     Arg::new("inplace")
        //         .short('i')
        //         .long("inplace")
        //         .help("Modify the file rather than output the new verison.")
        //         .action(ArgAction::SetTrue),
        // )
        .get_matches();

    let filename = matches.get_one::<String>("FILE").unwrap();

    let file = File::open(filename)
        .unwrap_or_else(|_| panic!("Could not open file {}", filename));
    

    let mut buf = BufReader::new(file);
    let mut input_str = String::new();
    let _ = buf.read_to_string(&mut input_str);
    
    let mut tree = to_mdast(&input_str, &ParseOptions::default())
        .unwrap_or_else(|_| panic!("Could not parse file {}", filename));


    let mut ctx = Downloader::new(PathBuf::from("images"));
    let newtree = transform(&mut tree, image_transfomer, &mut ctx);
    println!("{:?}", newtree);

}

struct Downloader{
    target_location: PathBuf
}

impl Downloader {
    fn new(target_location: PathBuf) -> Downloader {
        std::fs::create_dir_all(&target_location).unwrap();
        Downloader {
            target_location
        }
    }
    fn download(&self, url: Url) -> PathBuf {
        let imgfile = self.target_location.join(url.path_segments().unwrap().last().unwrap());
        let bytes = reqwest::blocking::get(url.clone())
            .unwrap_or_else(|_| panic!("Failed to download {}", url))
            .bytes()
            .unwrap_or_else(|_| panic!("Failed to read {}", url));

        let mut file = File::create(&imgfile).unwrap();
        file.write(&bytes).unwrap_or_else(|_| panic!("Failed to write {:?}", imgfile));
        return imgfile;
    }
}

fn image_transfomer(node: &mdast::Node, ctx: &mut Downloader) -> mdast::Node {
    match node{
        mdast::Node::Image(image) => {
            let image_url = Url::parse(&image.url).unwrap();
            let image_path = ctx.download(image_url);
            let mut new_image = image.clone();
            new_image.url = image_path.to_str().unwrap().to_string();
            return mdast::Node::Image(new_image);
        }
        _ => node.clone()
    }
}

fn transform<T>(tree: &mdast::Node, transfomer: fn(&mdast::Node, ctx: &mut T) -> mdast::Node, ctx:&mut T )
 -> mdast::Node {
    match tree.children() {
        None => transfomer(tree, ctx),

        Some(children) => {
            
            let mut newtree = transfomer(tree, ctx);
            let mut newchildren = newtree.children_mut().unwrap();

            let mut index = 0;

            while index < children.len() {
                let child = &children[index];
                let newchild = transform(&child, transfomer, ctx);
                newchildren[index] = newchild;
                index += 1;
            }
            
            return newtree;
        }
       
    }
    
}