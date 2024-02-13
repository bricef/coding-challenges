
# DevLog

## 2024-02-13

- `chrome_headless` crate works but suffers from performance issues. Raised a [ticket with the details](https://github.com/rust-headless-chrome/rust-headless-chrome/issues/460), but most of the time seems to be spent in chrome, and the binary I'm currently using doesn't have debug symbols.
- [Chromuimoxide](https://github.com/mattsse/chromiumoxide) may be a viable alternative that talks dircetly to the chrome devtool interface.
