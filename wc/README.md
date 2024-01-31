# `wc` clone

See [the original challenge](https://codingchallenges.fyi/challenges/challenge-wc).

## About

This simple program counts bytes, words and lines in a file, built in [Rust](https://www.rust-lang.org/).

## Build, Install, Run

To build, navigate to the challenge directory and run 

```sh
$ cargo build --release
```

You'll then find the executale in `target/release/wc`. You can also install the tool to uyour home directory using cargo as well:

```sh
$ cargo install --path .
```

## learnings

- The [Atty](https://docs.rs/atty/latest/atty/) crate was very useful to determine whether we're in a TTY or not.
- Using a quickly rolled state machine allowed calculating all counts simultaneously without reading the input multiple times.

## Improvements 

- [ ] Wrap the state machine creation and use in a simple library to reuse in future projects.