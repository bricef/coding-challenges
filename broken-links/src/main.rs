use std::collections::{HashMap, HashSet};

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::Command;
use clap::{Arg, ArgAction};
use anyhow::{anyhow, Result};
use fantoccini::ClientBuilder;
use cookie::SameSite;
use fantoccini::cookies::Cookie;
use tokio::runtime;
use tokio::sync::Mutex;
use url::Url;

mod url_utils;
mod link_scanner;
use link_scanner::{LinkScanner, LinkFailure};

mod page_scanner;
use page_scanner::{PageScanner, PageScannerOptions, Page, PageFailure};

mod monitor;
use monitor::Monitor;

#[derive(Debug, Clone, Hash)]
enum Reason {
    Code(u16),
    // Timeout(u32),
    // DNS(String),
    // SSL(String),
    Other(String)
} 

impl Eq for Reason {}

impl PartialEq for Reason {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Reason::Code(a), Reason::Code(b)) => a == b,
            // (Reason::Timeout(a), Reason::Timeout(b)) => true,
            // (Reason::DNS(a), Reason::DNS(b)) => a == b,
            // (Reason::SSL(a), Reason::SSL(b)) => a == b,
            (Reason::Other(a), Reason::Other(b)) => a == b,
            _ => false,
        }
    }

}

#[derive(Debug)]
enum ScanResult {
    LinkFailure(LinkFailure),
    PageFailure(PageFailure),
    PageSuccess(Url),
    LinkSuccess(Link)
}

#[derive(Debug)]
struct Link{
    source: Url,
    link: Url,
}

#[derive(Debug)]
struct Report {
    ok_pages: u32,
    ok_links: u32,
    link_failures: Vec<LinkFailure>,
    page_failures: Vec<PageFailure>,
}

async fn collect_results(results: Arc<Mutex<Vec<ScanResult>>>) -> Result<Report, anyhow::Error> {
    let mut ok_pages = 0;
    let mut ok_links = 0;
    let mut link_failures = Vec::new();
    let mut page_failures = Vec::new();
    let mut results = results.lock().await;
    loop{
        match results.pop(){
            Some(r) => match r {
                ScanResult::LinkSuccess(_l) => ok_links += 1,
                ScanResult::PageSuccess(_p) => ok_pages += 1,
                ScanResult::LinkFailure(f) => link_failures.push(f),
                ScanResult::PageFailure(f) => page_failures.push(f),
            }
            None => break,
        }
    }
    return Ok(Report{ok_pages, ok_links, link_failures, page_failures});
}

fn present_results(report: Report) {
    // println!("Results {:?}", report);
    println!("OK Pages: {}", report.ok_pages);
    println!("OK Links: {}", report.ok_links);

    let mut map: HashMap::<Reason, Vec<LinkFailure>> = HashMap::new();

    for failure in report.link_failures.into_iter(){
        let group = map.entry(failure.reason.clone()).or_insert(Vec::new());
        group.push(failure);
    }
    
    for (key, group) in map.into_iter(){
        println!("Link failures due to {:?}:", key);
        for f in group {
            println!("\t{} -> {} ", f.link.source, f.link.link );
        }
    }

    
    for f in report.page_failures{
        println!("Page failure: {} due to {:?}", f.page, f.reason);
    }
}

async fn show_results(results: Arc<Mutex<Vec<ScanResult>>>) -> Result<(), anyhow::Error>{
    let results = collect_results(results).await?;
    present_results(results);            
    println!("Done");
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    Ok(())

}

fn parse_target(url: &str) -> Result<Url, anyhow::Error> {
    match Url::parse(url) {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err(anyhow!("Invalid URL: {}", e)),
    }
}

fn host_url_from(url: &Url) -> Result<Url, anyhow::Error> {
    let mut host_url = url.clone();
    host_url.set_fragment(None);
    host_url.set_query(None);
    host_url.set_path("");
    Ok(host_url)
}

fn build_browser_capabilities() -> fantoccini::wd::Capabilities{
    let mut capabilities = serde_json::map::Map::new();
    let browser_options = serde_json::json!({ "args": ["--headless"] });
    capabilities.insert("moz:firefoxOptions".to_string(), browser_options.clone());
    // capabilities.insert("browserName".to_string(), serde_json::json!("Firefox"));
    // capabilities.insert("browserVersion".to_string(), serde_json::json!("105"));
    
    return capabilities;
}


fn main() -> Result<(), anyhow::Error>{
    let matches = Command::new("blf")
        .version("0.1.0")
        .author("Brice Fernandes <brice@fractallambda.com>")
        .about("Broken link finder")
        .arg(
            Arg::new("URL")
                .required(true)
                .help("The URL to scan."),
        )
        // .arg(
        //     Arg::new("dynamic")
        //         .short('d')
        //         .long("dynamic")
        //         .help("Run scripts on loaded page (False by default).")
        //         .action(ArgAction::SetTrue),
        // )
        .arg(
            Arg::new("follow")
                .short('f')
                .long("follow")
                .help("Follow links and perform analysis on all pages in subdomain. (False by default).")
                .action(ArgAction::SetTrue),
        )
        // .arg(
        //     Arg::new("clickable")
        //         .short('c')
        //         .long("clickable")
        //         .help("Scan for all elements that react to clicks, not juts hyperlinks. (False by default).")
        //         .action(ArgAction::SetTrue),
        // )
        .arg(
            Arg::new("cookie")
                .long("cookie")
                .help("Set cookie for requests. Can be specified multiple times.")
                .action(ArgAction::Append),
        )
        .get_matches();


    let parsed_url = parse_target(matches.get_one::<String>("URL").unwrap())?;
    let host_url = host_url_from(&parsed_url)?;
    let cookies: Vec<String> = matches.get_many::<String>("cookie").unwrap_or_default().map(|v| String::from(v)).collect::<Vec<_>>();

    // Set permitted scan hosts 
    let mut permitted_hosts = HashSet::new();
    permitted_hosts.insert(host_url.clone());

    let rt = runtime::Runtime::new()?; // multithreaded runtime

    let pages = Arc::new(Mutex::new(Vec::<Page>::new()));
    let links = Arc::new(Mutex::new(Vec::<Link>::new()));
    let results = Arc::new(Mutex::new(Vec::<ScanResult>::new()));

    let exit = Arc::new(AtomicBool::new(false));
    

    rt.block_on(async move {

        let client = Arc::new(
            ClientBuilder::native()
                .capabilities(build_browser_capabilities())
                .connect("http://localhost:4444")
                .await?
        );

        // We need this in order to set the cookies
        // As the webdriver protocol doesn't support setting cookies 
        // on a domain we're not already on. :shrug:
        client.goto(parsed_url.as_str()).await?;

        for cookie in &cookies {
            let mut c = Cookie::parse(cookie.to_owned()).unwrap();
            let domain = host_url.clone().domain().unwrap().to_string();
            c.set_domain(domain);
            c.set_secure(true);
            c.set_path("/");
            c.set_same_site(Some(SameSite::Lax));

            let _ = match client.add_cookie(c.clone()).await {
                Ok(_) => println!("Cookie set"),
                Err(e) => {
                    if let Some(client) = Arc::into_inner(client) {
                        client.close().await?;
                    }else {
                        eprintln!("WARNING: Failed to close client");
                    }
                    panic!("Error setting cookie: {}", e)
                }
            };
        }

        let mut page_scanner = PageScanner::new(
            pages.clone(),
            links.clone(),
            results.clone(),
            client.clone(),
            PageScannerOptions {
                scope: permitted_hosts,
                follow: matches.get_flag("follow"),
                // clickable: matches.get_flag("clickable"),
                // dynamic: matches.get_flag("dynamic"),
            },
            exit.clone()
        )?;

        // hydrate work queue with initial url
        {
            // println!("Seeding page requests");
            pages.lock().await.push(parsed_url.clone());
        }

        let mut set = tokio::task::JoinSet::new();

        let mut monitor = Monitor::new(pages.clone(), links.clone(), results.clone());
        
        let jh = tokio::spawn( async move { 
            monitor.start().await; 
        }) ; 

        // Set up the page scanner worker
        set.spawn(async move {
             let _ = page_scanner.start().await; 
             drop(page_scanner);
        });

        // Set up the link scanner workers
        for _ in 0..8 {
            let mut link_scanner = LinkScanner::new(links.clone(), results.clone(), exit.clone());
            set.spawn(async move { 
                link_scanner.start().await; 
                drop(link_scanner);
            });
        }
        
        let _ = tokio::join!(jh);
        // println!("Closing workers");
        exit.store(true, Ordering::Relaxed);

        // Wait for all workers to finish
        while let Some(_) = set.join_next().await {}
        
        show_results(results.clone()).await?;

        println!("Closing client");
        if let Some(client) = Arc::into_inner(client) {
            client.close().await?;
        }else {
            eprintln!("WARNING: Failed to close client");
        }
        
        Ok(())
    })

}
