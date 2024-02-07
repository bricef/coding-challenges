# Deduplicator

## About

A file deduplicator in rust.

```
Interactive file deduplicator

Usage: dedup [OPTIONS] <DIRECTORY>

Arguments:                                                                                              <DIRECTORY>  The directory to scan for duplicate files.

Options:
    -r, --report      Report on duplicate files.
    -f                Follow symlinks when scanning (False by default).
    -d, --autodelete  Automatically delete duplicate files without prompting.
    -h, --help        Print help
    -V, --version     Print version
```

## Enhancements

- [ ] Fuzzy matching mode using edit distance between files.
- [ ] Prompt with numbers to speed up option selection
- [ ] Check on delete
- [x] Deduplicate code for match checking
- [x] Implement symlink following

## Notes

Libraries for prompting:
- https://docs.rs/inquire/latest/inquire/
- https://docs.rs/prompts/latest/prompts/index.html
- https://lib.rs/crates/promptuity
- https://crates.io/crates/promptly
- https://github.com/console-rs/indicatif
- https://github.com/console-rs/console
- https://docs.rs/dialoguer/latest/dialoguer/struct.Select.html#method.interact
- https://crates.io/crates/requestty
