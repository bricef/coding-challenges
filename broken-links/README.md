# Broken Link Finder

## About

`blf` scans a url for broken links. by default, it will follow links on the same domain ad perform the same analysis on these as well. It will work with dynamically generated content and Javascript.

## Usage

```shell
```

## Enhancements

## Todo

- [ ] Scrape a single static page
- [ ] Transitive scrape of links within domain
- [ ] Run JS on page to get link
- [ ] Scan clickable elements, not just hyperlinks.
- [ ] Multithread the code

## Learnings

1. Performance analysis with `perf`
2. [Chrome devtools protocol](https://chromedevtools.github.io/devtools-protocol/) vs [webdriver standard](https://www.w3.org/TR/webdriver2/).

## References and resources

- [chrome for testing binary](https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/121.0.6167.85/linux64/chrome-linux64.zip)
- [chromedriver binary](https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/121.0.6167.85/linux64/chromedriver-linux64.zip)
- [chrome-headless-shell binary](https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/121.0.6167.85/linux64/chrome-headless-shell-linux64.zip)
