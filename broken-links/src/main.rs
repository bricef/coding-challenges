use core::time;
use std::collections::HashSet;
use std::time::Instant;

use clap::Command;
use clap::{Arg, ArgAction};

use anyhow::Result;
// use fantoccini::{ClientBuilder, Client, Locator};
use headless_chrome::Browser;


#[derive(Debug)]
enum Reason {
    Code(u32),
    Timeout,
    DNS(String),
    SSL(String),
} 

struct Failure {
    url: String,
    reason: Reason,
} 

struct Location {
    url: String,
    line: u32,
    column: u32,
}

struct Href{
    location: Location,
    url: String,
}

struct ScannerOptions {
    follow: bool,
    clickable: bool,
    dynamic: bool,
}

struct Scanner {
    options: ScannerOptions,
    scanned: HashSet<String>
}

impl Scanner {
    async fn new(options: ScannerOptions) -> Result<Self, anyhow::Error> {
        
        Ok(Scanner {
            options,
            scanned: HashSet::new(),
        })
    }

    async fn hrefs(&mut self, url: String) -> Result<Vec<Href>, anyhow::Error> {
        let browser = Browser::default()?;

        let tab = browser.new_tab()?;
        // tab.set_default_timeout(std::time::Duration::from_secs(5));

        // Navigate to wikipedia
        tab.navigate_to("https://stackoverflow.com/questions/14248063/xpath-to-select-element-by-attribute-value")?;
        tab.wait_until_navigated()?;
        println!("Scanning elements...");
        // tab.wait_for_element("div")?;
        let start = Instant::now();
        let es = tab.find_elements("[href]")?;
        let duration = start.elapsed();
        println!("Time elapsed in find_elements() is: {:?}", duration);

        for e in es {
            match e.get_attribute_value("href") {
                Ok(href) => println!("{:?}", href),
                Err(_) => println!("No href"),
            }
        }
        // let client = ClientBuilder::native().connect("http://localhost:9515").await?;
        // client.goto(&url).await?;
        // client
        //     .wait()
        //     .at_most(time::Duration::from_secs(5))
        //     .for_element(Locator::Css(r"[href]")).await?;
        // let elems = client.find_all(Locator::Css(r"[href]")).await?;
        // for elem in elems {
        //     if let Some(href) = elem.attr("href").await? {
        //         self.scanned.insert(href.clone());
                
        //     }
        // }
        // for elem in self.scanned.iter() {
        //     println!("{}", elem);
        // }
        // client.close().await?;
        Ok(vec![])
    }

    async fn scan(&mut self, url: String) -> Result<Vec<Failure>, anyhow::Error> {
        // let tab = self.browser.new_tab()?;
        // tab.navigate_to(&url)?;
        // tab.find_element("a")?;
        Ok(vec![])
    }
}



// fn scan_links(url: String) -> Result<Vec<HRef>, std::io::Error> {

//     let mut links = Vec::new();

//     let client = reqwest::blocking::Client::new();
//     let res = client.get(&url).send()?;

//     let body = res.text()?;

//     let document = Document::from(body.as_str());

//     for node in document.find(Name("a")).filter_map(|n| n.attr("href")) {
//         links.push(HRef::new(node.to_string()));
//     }

//     Ok(links)
// }

fn display_results(results: Vec<Failure>) {
    for result in results {
        println!("{}: {:#?}", result.url, result.reason);
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error>{
    let matches = Command::new("blf")
        .version("0.1.0")
        .author("Brice Fernandes <brice@fractallambda.com>")
        .about("Broken link finder")
        .arg(
            Arg::new("URL")
                .required(true)
                .help("The URL to scan."),
        )
        .arg(
            Arg::new("dynamic")
                .short('d')
                .long("dynamic")
                .help("Run scripts on loaded page (False by default).")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("follow")
                .short('f')
                .long("follow")
                .help("Follow links and perform analysis on all pages in subdomain. (False by default).")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("clickable")
                .short('c')
                .long("clickable")
                .help("Scan for all elements that react to clicks, not juts hyperlinks. (False by default).")
                .action(ArgAction::SetTrue),
        )
        .get_matches();


    let url = matches.get_one::<String>("URL").unwrap();

    let mut scanner = Scanner::new(ScannerOptions {
        follow: matches.get_flag("follow"),
        clickable: matches.get_flag("clickable"),
        dynamic: matches.get_flag("dynamic"),
    }).await?;

    let results = scanner.hrefs(url.clone()).await?; 

    // display_results(results);

    Ok(())

}
