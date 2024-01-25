# `wc` clone

## About

This simple program counts bytes, words and lines in a file.

## learnings

- The [Atty](https://docs.rs/atty/latest/atty/) crate was very useful to determine whether we're in a TTY or not.
- Using a quickly rolled state machine allowed calculating all counts simultaneously without reading the input multiple times.

## Improvements 

- [ ] Wrap the state machine creation and use in a simple library to reuse in future projects.