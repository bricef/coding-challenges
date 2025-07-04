use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashSet;
use fantoccini::Locator;
use url::Url;

use tokio::sync::Mutex;

use crate::{Link, ScanResult, Reason, host_url_from};
use crate::url_utils::canonical;

#[derive(Debug)]
pub struct PageFailure {
    pub page: Url,
    pub reason: Reason,
}

pub type Page = Url;

#[allow(dead_code)]
pub struct PageScannerOptions {
    pub scope: HashSet<Url>,
    pub follow: bool,
    // clickable: bool,
    // dynamic: bool,
}

#[allow(dead_code)]
pub struct PageScanner {
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
    pub fn new(
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

    pub async fn start(&mut self) -> Result<(), anyhow::Error>{
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

    pub async fn page_to_hrefs(&mut self, url: &Page) -> Result<Vec<Link>, anyhow::Error> {
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