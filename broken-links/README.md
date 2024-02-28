# Broken Link Finder

## About

`blf` scans a url for broken links. by default, it will follow links on the same domain ad perform the same analysis on these as well. It will work with dynamically generated content and Javascript.

## Usage

```shell
```

## Enhancements

## Todo

- [x] Scrape a single static page
- [ ] Transitive scrape of links within domain
- [x] Run JS on page to get link
- [ ] Scan clickable elements, not just hyperlinks.
- [x] Multithread the code


## Design 
```rust
//
// Pseudocode for approach 
//

// Page queue for pages in site that need to be scrapped
let mut pq = UniqQueue<Page>::new();

// Link queue for links that need to be verified 
let mut lq = UniqQueue<Link>::new();

// Output Queue for worker results to be shared into.
let mut oq = SyncSet<ScanResult>::new();

// X page-workers with their own webdriver clients and webdriver instance
let mut page_workers = Vec<PageWorker>::new();
for _ in 0..npworkers {
    page_workers.push(PageWorker::new(pq, lq, oq));
}

// X link-workers checking links for status code.
let mut link_workers = Vec<LinkWorker>::new();
for _ in 0..nlworkers {
    link_worker.push(LinkWorker::new(lq, oq));
}

// Create a progress monitor to update the UI
let progress_monitor = ProgressWorker::new(pg, lq, oq);

// Initial URL(s) to scan are added to the work queue
pq.push(Page{url: "https://some.url"})


spawn!(progress_worker)
spawn_all!(page_workers)
spawn_all!(link_workers)

join_all!(page_workers)
join_all!(link_workers)

//Post processing step to turn finsihed work queue into readable report
display_results(oq)
```

## Learnings

1. Performance analysis with `perf`
2. [Chrome devtools protocol](https://chromedevtools.github.io/devtools-protocol/) vs [webdriver standard](https://www.w3.org/TR/webdriver2/).

## References and resources

- [chrome for testing binary](https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/121.0.6167.85/linux64/chrome-linux64.zip)
- [chromedriver binary](https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/121.0.6167.85/linux64/chromedriver-linux64.zip)
- [chrome-headless-shell binary](https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/121.0.6167.85/linux64/chrome-headless-shell-linux64.zip)
