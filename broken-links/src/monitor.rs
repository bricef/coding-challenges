use std::cmp;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::page_scanner::Page;
use crate::Link;
use crate::ScanResult;

pub struct Monitor{
    pages: Arc<Mutex<Vec<Page>>>,
    links: Arc<Mutex<Vec<Link>>>,
    results: Arc<Mutex<Vec<ScanResult>>>,
}

impl Monitor {
    pub fn new(
        pages: Arc<Mutex<Vec<Page>>>,
        links: Arc<Mutex<Vec<Link>>>,
        results: Arc<Mutex<Vec<ScanResult>>>,
    ) -> Self {
        Monitor{ pages, links, results }
    }
    pub async fn start(&mut self){
        let m = MultiProgress::new();
        let sty = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7} {msg}")
            .unwrap()
            .progress_chars("##-");

        let pages_todo = m.add(ProgressBar::new(100));
        pages_todo.set_style(sty.clone());
        pages_todo.set_message("Unscanned Pages");

        let links_todo = m.add(ProgressBar::new(100));
        links_todo.set_style(sty.clone());
        links_todo.set_message("Unscanned Links");
        
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
    }
}
