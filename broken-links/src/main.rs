use std::cmp;
use std::collections::{HashMap, HashSet};

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::Command;
use clap::{Arg, ArgAction};
use anyhow::{anyhow, Result};
use fantoccini::{ClientBuilder, Locator};
use cookie::{SameSite};
use fantoccini::cookies::Cookie;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::runtime;
use tokio::sync::Mutex;
use url::{Url, ParseError};

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
    cookies: Vec<String>,
    // clickable: bool,
    // dynamic: bool,
}

#[allow(dead_code)]


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
struct PageScanner {
    seen: HashSet<Url>,
    pages: Arc<Mutex<Vec<Page>>>,
    links: Arc<Mutex<Vec<Link>>>,
    results: Arc<Mutex<Vec<ScanResult>>>,
    client: Arc<fantoccini::Client>,
    options: PageScannerOptions,
    exit: Arc<AtomicBool>,
    // scanned: HashSet<String>,
    // channels: PageScannerChannels,
}

impl PageScanner {
    // async fn default() -> Result<Self, anyhow::Error> {
    //     Scanner::new(ScannerOptions {
    //         follow: true,
    //         clickable: true,
    //         dynamic: true,
    //     }).await
    // }
    fn new(
        pages: Arc<Mutex<Vec<Page>>>, 
        links: Arc<Mutex<Vec<Link>>>, 
        results: Arc<Mutex<Vec<ScanResult>>>, 
        client: Arc<fantoccini::Client>, 
        options: PageScannerOptions,
        exit: Arc<AtomicBool>
    ) -> Result<Self, anyhow::Error> {   
        Ok(PageScanner {
            seen: HashSet::new(),
            pages,
            links,
            results,
            client,
            options,
            exit
        })
    }

    async fn start(&mut self) -> Result<(), anyhow::Error>{
        // println!("Starting page scanner");
        
        loop {
            let mut pages = self.pages.lock().await;
            let next_page = pages.pop();
            drop(pages);

            match next_page {
                Some(p) => {
                    // Ignore already scanned pages
                    if self.seen.contains(&p) {
                        continue;
                    }
                    // We haven't scanned this one before. 
                    // Add it to the list of seen pages
                    self.seen.insert(p.clone());
                    
                    // Scan the page
                    let hrefs = self.page_to_hrefs(&p).await;
                    
                    // Process the results
                    match hrefs {
                        Ok(hrefs) => {

                            let mut results = self.results.lock().await;
                            results.push(ScanResult::PageSuccess(p.clone()));
                            drop(results);
                            

                            for h in hrefs {
                                let host = host_url_from(&h.link)?;
                                if self.options.scope.contains(&host) && self.options.follow { 
                                    // Same domain
                                    self.pages.lock().await.push(h.link);
                                } else { 
                                    // external link
                                    self.links.lock().await.push(h);
                                }
                            }
                        },
                        Err(e) => {
                            self.results.lock().await.push(ScanResult::PageFailure(PageFailure{page:p , reason: Reason::Other(e.to_string())}));
                        }
                    }
                },
                None => tokio::time::sleep(Duration::from_millis(100)).await,
            }
            if self.exit.load(Ordering::Relaxed) {
                break;
            }
        }   
        // println!("Ending page scanner");
        Ok(())
    }

    async fn page_to_hrefs(&mut self, url: &Page) -> Result<Vec<Link>, anyhow::Error> {
        let mut hrefs: HashSet<Url> = HashSet::new();
        
        self.client.goto(url.as_str()).await?;

        let canonify = | fragment : &str | canonical(url, fragment);
        
        // search for hrefs
        let refs =  self.client.find_all(Locator::Css("[href]")).await?;
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

        // search for sources
        let srcs= self.client.find_all(Locator::Css("[src]")).await?;
        for r in srcs {
            match r.attr("src").await {
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
    input: Arc<Mutex<Vec<Link>>>,
    output: Arc<Mutex<Vec<ScanResult>>>,
    client: reqwest::Client,
    exit: Arc<AtomicBool>,
}
impl LinkScanner {
    fn new(input: Arc<Mutex<Vec<Link>>>, output: Arc<Mutex<Vec<ScanResult>>>, exit: Arc<AtomicBool>) -> Self {
        LinkScanner {
            input,
            output,
            client: reqwest::Client::new(),
            exit,
        }
    }
    async fn start(&mut self ){
        // println!("Starting link scanner");
        loop {
            let mut links = self.input.lock().await;
            let next_link = links.pop();
            drop(links);

            match next_link {
                Some(l) => {
                    // println!("Checking link: {}", l.link.as_str());
                    self.check_link(l).await
                },
                None => tokio::time::sleep(Duration::from_millis(100)).await, 
            }
            if self.exit.load(Ordering::Relaxed) {
                break;
            }
        }
        // println!("Ending link scanner");
    }
    async fn check_link(&mut self, l: Link){
        let res = self.client.head(l.link.as_str()).send().await;
        match res {
            Ok(res) => {
                if res.status().is_success() {
                    self.output.lock().await.push(ScanResult::LinkSuccess(l));
                } else {
                    self.output.lock().await.push(ScanResult::LinkFailure(LinkFailure{link: l, reason: Reason::Code(res.status().as_u16())}));
                }
            },
            Err(e) => {
                self.output.lock().await.push(ScanResult::LinkFailure(LinkFailure{link: l, reason: Reason::Other(e.to_string())}));
            }
        }
    }
}

struct Monitor{
    pages: Arc<Mutex<Vec<Page>>>,
    links: Arc<Mutex<Vec<Link>>>,
    results: Arc<Mutex<Vec<ScanResult>>>,
}

impl Monitor {
    fn new(
        pages: Arc<Mutex<Vec<Page>>>,
        links: Arc<Mutex<Vec<Link>>>,
        results: Arc<Mutex<Vec<ScanResult>>>,
    ) -> Self {
        Monitor{ pages, links, results }
    }
    async fn start(&mut self){
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

        let mut results_max = 0;

        loop {
            let pages = self.pages.lock().await;
            let links = self.links.lock().await;
            let results = self.results.lock().await;

            let pages_todo_count = pages.len();
            let links_todo_count = links.len();
            let results_count = results.len();

            drop(pages);
            drop(links);
            drop(results);

            let total = pages_todo_count + links_todo_count + results_count;

            pages_todo.set_length(total as u64);
            pages_todo.set_position(pages_todo_count as u64);

            links_todo.set_length(total as u64);
            links_todo.set_position(links_todo_count as u64);

            results_done.set_length(total as u64);
            if results_count > results_max {
                results_max = results_count;
            }
            results_done.set_position(cmp::max(results_max, results_count) as u64);
            
            tokio::time::sleep(Duration::from_millis(100)).await;

            if pages_todo_count == 0 && links_todo_count == 0 && results_count != 0 {
                break;
            }
        }

        pages_todo.finish_and_clear();
        links_todo.finish_and_clear();
        results_done.finish_and_clear();
        m.clear().unwrap();
        // println!("Ending updater");
        
    }
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

        for cookie in &cookies {
            let mut c = Cookie::parse(cookie.to_owned()).unwrap();
            let domain = host_url.clone().domain().unwrap().to_string();
            println!("Setting cookie domain to {}", domain);
            c.set_domain(domain);
            c.set_path("/");
            c.set_same_site(Some(SameSite::Lax));

            let _ = match client.add_cookie(c).await {
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
                cookies: cookies,
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
