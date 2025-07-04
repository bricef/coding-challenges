

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use reqwest::Client;

use tokio::sync::Mutex;

use crate::{Link, ScanResult, LinkFailure, Reason};


pub struct LinkScanner {
    input: Arc<Mutex<Vec<Link>>>,
    output: Arc<Mutex<Vec<ScanResult>>>,
    client: Client,
    exit: Arc<AtomicBool>,
}
impl LinkScanner {
    pub fn new(input: Arc<Mutex<Vec<Link>>>, output: Arc<Mutex<Vec<ScanResult>>>, exit: Arc<AtomicBool>) -> Self {
        LinkScanner {
            input,
            output,
            client: Client::new(),
            exit,
        }
    }
    pub async fn start(&mut self ){
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
    pub async fn check_link(&mut self, l: Link){
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