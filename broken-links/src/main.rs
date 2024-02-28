
use core::time;
use std::collections::HashSet;

use std::hash::Hash;
use std::io::{Stdin, Write};
use std::iter::Scan;
use std::result;
use std::time::Duration;
use clap::Command;
use clap::{Arg, ArgAction};

use anyhow::{anyhow, Result};
use failure::Fail;
use fantoccini::{ClientBuilder, Locator};
use crossbeam::channel::{Receiver, Sender, unbounded, RecvTimeoutError};
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
use tokio::runtime;
use tokio::task::JoinHandle;
use url::form_urlencoded::parse;
use url::{Url, ParseError};

#[derive(Debug)]
enum Reason {
    Code(u16),
    Timeout(u32),
    DNS(String),
    SSL(String),
    Other(String)
} 

#[derive(Debug)]
struct LinkFailure {
    link: Link,
    reason: Reason,
} 

#[derive(Debug)]
struct PageFailure {
    page: Url,
    reason: Reason,
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

#[allow(dead_code)]
struct PageScannerOptions {
    scope: HashSet<Url>,
    follow: bool,
    clickable: bool,
    dynamic: bool,
}

#[allow(dead_code)]
struct PageScannerChannels {
    page_input: Receiver<Page>,
    page_output: Sender<Page>,
    link_ouput: Sender<Link>,
    result_output: Sender<ScanResult>,
}

#[allow(dead_code)]
struct PageScanner {
    client: fantoccini::Client,
    options: PageScannerOptions,
    // scanned: HashSet<String>,
    channels: PageScannerChannels,
}

type Page = Url;

fn canonical (base : &Url, url: &str) -> Result<Url, anyhow::Error> {
    match Url::parse(url) {
        Ok(parsed) => {
            let scheme = parsed.scheme();
            if !scheme.is_empty() && !scheme.starts_with("http") {
                // println!("Invalid scheme for {}: {}", parsed.as_str(), parsed.scheme());
                return Err(anyhow!("Invalid scheme for {}", parsed.as_str()));
            }
            let mut parsed = parsed;
            parsed.set_fragment(None);
            Ok(parsed)
        },
        Err(ParseError::RelativeUrlWithoutBase) => {
            let mut parsed = base.join(url)?;
            parsed.set_fragment(None);
            Ok(parsed)
        },
        Err(e) => {
            println!("Invalid URL: {}", e);
            Err(anyhow!("Invalid URL: {}", e))
        }
    }
}

mod test{
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_canonical() {
        let base = Url::parse("http://example.com").unwrap();
        let canon = move | url: &str | canonical(&base, url);

        assert_eq!(canon("http://example.com").unwrap().to_string(), "http://example.com/");
        assert_eq!(canon("https://example.com").unwrap().to_string(), "https://example.com/");
        assert_eq!(canon("ftp://example.com").unwrap_err().to_string(), "Invalid scheme for ftp://example.com/");
        assert_eq!(canon("/foo/bar").unwrap().to_string(), "http://example.com/foo/bar");
    }

}

impl PageScanner {
    // async fn default() -> Result<Self, anyhow::Error> {
    //     Scanner::new(ScannerOptions {
    //         follow: true,
    //         clickable: true,
    //         dynamic: true,
    //     }).await
    // }
    fn new(channels: PageScannerChannels, client: fantoccini::Client, options: PageScannerOptions) -> Result<Self, anyhow::Error> {   
        Ok(PageScanner {
            channels,
            client,
            options,
        })
    }

    async fn start(&mut self) -> Result<(), anyhow::Error> {
        // println!("Starting page scanner");
        let timeout = 500;
        loop {
            match self.channels.page_input.recv_timeout(Duration::from_millis(timeout)){
                Ok(p) => {
                    let hrefs = self.page_to_hrefs(&p).await;
                    match hrefs {
                        Ok(hrefs) => {
                            self.channels.result_output.send(ScanResult::PageSuccess(p.clone()))?;
                            for h in hrefs {
                                // println!("Pushing link: {}", h.url.as_str());
                                self.channels.link_ouput.send(Link { source: p.clone(), link: h.link.clone()})?;
                            }
                            // r_out.send(ScanResult::Failure(Failure{source: p.clone(), link: h.url.clone(), reason: Reason::Timeout(500)}))?;
                        },
                        Err(e) => {
                            self.channels.result_output.send(ScanResult::PageFailure(PageFailure{page:p , reason: Reason::Other(e.to_string())}))?;
                        }
                    }
                },
                Err(RecvTimeoutError::Timeout) => break, //{ println!("Timeout on pages"); break },
                Err(RecvTimeoutError::Disconnected) => break, //{ println!("Disconnect on pages"); break },
            }
        }
        Ok(())
    }

    async fn page_to_hrefs(&mut self, url: &Page) -> Result<Vec<Link>, anyhow::Error> {
        let mut hrefs: HashSet<Url> = HashSet::new();

        self.client.goto(url.as_str()).await?;
        
        let refs =  self.client.find_all(Locator::Css("[href]")).await?;

        let canonify = | fragment : &str | canonical(url, fragment);

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

        let out = hrefs.into_iter().map(|h| Link{source: url.clone(), link: h}).collect();
        Ok(out)
    }
}

struct LinkScanner {
    input: Receiver<Link>,
    output: Sender<ScanResult>,
    timeout: u64,
    client: reqwest::Client,
}
impl LinkScanner {
    fn new(input: Receiver<Link>, output: Sender<ScanResult>) -> Self {
        LinkScanner {
            input,
            output,
            timeout: 2000,
            client: reqwest::Client::new(),
        }
    }
    async fn start(&mut self, ) -> Result<(), anyhow::Error>{
        // println!("Starting link scanner");
        loop {
            match self.input.recv_timeout(Duration::from_millis(self.timeout)) {
                Ok(l) => {
                    // println!("Checking link: {}", l.link.as_str());
                    self.check_link(l).await?
                },
                Err(RecvTimeoutError::Timeout) => break, //{ println!("Timeout on links"); break },
                Err(RecvTimeoutError::Disconnected) => break, //{ println!("Disconnect on links"); break },
            
            }
        }
        Ok(())
    }
    async fn check_link(&mut self, l: Link) -> Result<(), anyhow::Error> {
        let res = self.client.head(l.link.as_str()).send().await;
        match res {
            Ok(res) => {
                if res.status().is_success() {
                    self.output.send(ScanResult::LinkSuccess(l))?;
                } else {
                    self.output.send(ScanResult::LinkFailure(LinkFailure{link: l, reason: Reason::Code(res.status().as_u16())}))?;
                }
            },
            Err(e) => {
                self.output.send(ScanResult::LinkFailure(LinkFailure{link: l, reason: Reason::Other(e.to_string())}))?;
            }
        }
        Ok(())
    }
}

struct ProgressUpdater{
    page_send: Sender<Page>,
    page_recv: Receiver<Page>,
    link_send: Sender<Link>,
    link_recv: Receiver<Link>,
    results: Receiver<ScanResult>,
}

impl ProgressUpdater {
    fn new(
        page_send: Sender<Page>,
        page_recv: Receiver<Page>,
        link_send: Sender<Link>,
        link_recv: Receiver<Link>,
        results: Receiver<ScanResult>,
    ) -> Self {
        ProgressUpdater{
            page_send,
            page_recv,
            link_send,
            link_recv,
            results,
        }
    }
    async fn start(&mut self) {
        // println!("Starting updater");
        let m = MultiProgress::new();
        let sty = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7} {msg}")
            .unwrap()
            .progress_chars("##-");

        let pages_todo = m.add(ProgressBar::new(100));
        pages_todo.set_style(sty.clone());
        pages_todo.set_message("Unscanned Pages");

        // let pages_done = m.add(ProgressBar::new(100));
        // pages_done.set_style(sty.clone());
        // pages_done.set_message("Scanned Pages");
        
        let links_todo = m.add(ProgressBar::new(100));
        links_todo.set_style(sty.clone());
        links_todo.set_message("Unscanned Links");
        
        // let links_done = m.add(ProgressBar::new(100));
        // links_done.set_style(sty.clone());
        // links_done.set_message("Scanned Links");
        
        let results_done = m.add(ProgressBar::new(100));
        results_done.set_position(0);
        results_done.set_style(sty.clone());
        results_done.set_message("Results");
        results_done.enable_steady_tick(Duration::from_millis(100));

        loop {
            

            let pages_todo_count = self.page_send.len();
            let links_todo_count = self.link_send.len();
            let results_count = self.results.len();
            let total = pages_todo_count + links_todo_count + results_count;

            pages_todo.set_length(total as u64);
            pages_todo.set_position(pages_todo_count as u64);

            links_todo.set_length(total as u64);
            links_todo.set_position(links_todo_count as u64);

            results_done.set_length(total as u64);
            results_done.set_position(results_count as u64);
            
            // if self.page_send.is_empty() && self.link_send.is_empty() && self.results.is_empty() {
            //     break;
            // } else {
                tokio::time::sleep(Duration::from_millis(100)).await;
            // }
        }
        println!("Ending updater");

        m.clear().unwrap();
    }
}

struct Report {
    ok_pages: u32,
    ok_links: u32,
    link_failures: Vec<LinkFailure>,
    page_failures: Vec<PageFailure>,
}

async fn collect_results(results: Receiver<ScanResult>) -> Result<Report, anyhow::Error> {
    let mut ok_pages = 0;
    let mut ok_links = 0;
    let mut link_failures = Vec::new();
    let mut page_failures = Vec::new();
    loop{
        match results.recv_timeout(Duration::from_millis(50)) {
            Ok(r) => match r {
                ScanResult::LinkSuccess(l) => ok_links += 1,
                ScanResult::PageSuccess(p) => ok_pages += 1,
                ScanResult::LinkFailure(f) => link_failures.push(f),
                ScanResult::PageFailure(f) => page_failures.push(f),
            }
            Err(_) => break,
        }
    }
    return Ok(Report{ok_pages, ok_links, link_failures, page_failures});
}

fn present_results(report: Report) {
    println!("OK Pages: {}", report.ok_pages);
    println!("OK Links: {}", report.ok_links);
    for f in report.link_failures {
        println!("Link failure: {} -> {} due to {:?}", f.link.source, f.link.link, f.reason);
    }
    for f in report.page_failures{
        println!("Page failure: {} due to {:?}", f.page, f.reason);
    }
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
    let parsed_url = Url::parse(url.as_str())?;

    

    let rt = runtime::Runtime::new()?; // multithreaded runtime

    let (p_s,p_r) = unbounded::<Page>();
    let (l_s,l_r) = unbounded::<Link>();
    let (r_s,r_r) = unbounded::<ScanResult>();

    

    rt.block_on(async move {

        let mut capabilities = serde_json::map::Map::new();
        let browser_options = serde_json::json!({ "args": ["--headless"] });
        capabilities.insert("moz:firefoxOptions".to_string(), browser_options.clone());

        let web_driver_client = ClientBuilder::native()
            .capabilities(capabilities)
            .connect("http://localhost:4444")
            .await
            .expect("failed to connect to WebDriver"); 
        
        let mut host_url = parsed_url.clone();
        host_url.set_fragment(None);
        host_url.set_query(None);
        host_url.set_path("");

        let mut permitted_hosts = HashSet::new();
        permitted_hosts.insert(host_url);

        let mut page_scanner = PageScanner::new(
            PageScannerChannels{
                page_input: p_r.clone(),
                page_output: p_s.clone(), 
                link_ouput: l_s.clone(),
                result_output: r_s.clone(),
            },
            web_driver_client,
            PageScannerOptions {
                scope: permitted_hosts,
                follow: matches.get_flag("follow"),
                clickable: matches.get_flag("clickable"),
                dynamic: matches.get_flag("dynamic"),
            },
        )?;

        // hydrate work queue with initial url
        p_s.send(parsed_url.clone())?;

        let mut set = tokio::task::JoinSet::new();

        // let mut progress_updater = ProgressUpdater::new(p_s.clone(), p_r.clone(), l_s.clone(), l_r.clone(), r_r.clone());
        // let ph = tokio::spawn( async move { progress_updater.start().await; }) ; 

        // Set up the page scanner worker
        set.spawn(async move {
             let _ = page_scanner.start().await; 
             drop(page_scanner);
        });

        // Set up the link scanner workers
        for _ in 0..8 {
            let mut link_scanner = LinkScanner::new(l_r.clone(), r_s.clone());
            set.spawn(async move { 
                let _ = link_scanner.start().await; 
            });
        }

        // Wait for all tasks
        while let Some(_) = set.join_next().await {
            ()
        }

        let results = collect_results(r_r).await?;
        present_results(results);            
        println!("Done");
        // let _ = std::io::stdout().flush();
        // let _ = std::io::stderr().flush();
        // tokio::join!(ph);
        // ph.abort();
        Ok(())
    })

}
