use core::time;
use std::collections::HashSet;
use std::time::Instant;

use clap::Command;
use clap::{Arg, ArgAction};

use anyhow::{Result, anyhow};
use fantoccini::{ClientBuilder, Client, Locator};
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

struct Href{
    source_page: String,
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

use url::{Url, ParseError};

fn canonical (base : &Url, url: &str) -> Result<String, anyhow::Error> {
    match Url::parse(url) {
        Ok(parsed) => {
            let scheme = parsed.scheme();
            if !scheme.is_empty() && !scheme.starts_with("http") {
                // println!("Invalid scheme for {}: {}", parsed.as_str(), parsed.scheme());
                return Err(anyhow!("Invalid scheme for {}", parsed.as_str()));
            }
            let mut parsed = parsed;
            parsed.set_fragment(None);
            Ok(parsed.as_str().to_string())
        },
        Err(ParseError::RelativeUrlWithoutBase) => {
            let mut parsed = base.join(url)?;
            parsed.set_fragment(None);
            Ok(parsed.as_str().to_string())
        },
        Err(e) => {
            println!("Invalid URL: {}", e);
            Err(anyhow!("Invalid URL: {}", e))
        }
    }
}

mod test{
    use super::*;

    #[test]
    fn test_canonical() {
        let base = Url::parse("http://example.com").unwrap();
        let canon = move | url: &str | canonical(&base, url);

        assert_eq!(canon("http://example.com").unwrap(), "http://example.com/");
        assert_eq!(canon("https://example.com").unwrap(), "https://example.com/");
        assert_eq!(canon("ftp://example.com").unwrap_err().to_string(), "Invalid scheme for ftp://example.com/");
        assert_eq!(canon("/foo/bar").unwrap(), "http://example.com/foo/bar");
    }

}


impl Scanner {
    async fn new(options: ScannerOptions) -> Result<Self, anyhow::Error> {
        
        Ok(Scanner {
            options,
            scanned: HashSet::new(),
        })
    }

    async fn hrefs(&mut self, url: String) -> Result<Vec<Href>, anyhow::Error> {
        let c = ClientBuilder::native().connect("http://localhost:4444").await.expect("failed to connect to WebDriver");
        let mut hrefs: HashSet<String> = HashSet::new();

        c.goto(url.as_str()).await?;
        
        let refs =  c.find_all(Locator::Css("[href]")).await?;

        let here = Url::parse(&url)?;
        let canonify = | url : &str | canonical(&here, url);

        for r in refs {
            match r.attr("href").await {
                Ok(Some(href)) => match canonify(&href) {
                    Ok(canon) => drop(hrefs.insert(canon)),
                    Err(_) => (), // Ignore bad URLs
                },
                Ok(None) => (), // Ignore no href
                Err(_) => (), // Ignore no href
            }
        }

        c.close().await?;
        let out = hrefs.into_iter().map(|h| Href{source_page: url.clone(), url: h}).collect();
        Ok(out)
    }

    async fn scan(&mut self, url: String) -> Result<Vec<Failure>, anyhow::Error> {
        // let tab = self.browser.new_tab()?;
        // tab.navigate_to(&url)?;
        // tab.find_element("a")?;
        Ok(vec![])
    }
}


fn display_results(results: Vec<Href>) {
    for result in results {
        println!("{}: {:#?}", result.source_page, result.url);
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

    display_results(results);

    Ok(())

}
