# `press`: a Huffman file compressor

See [the original challenge](https://codingchallenges.fyi/challenges/challenge-huffman).

## About

`press` is a simple file compression utility which uses [Huffman encodig](https://en.wikipedia.org/wiki/Huffman_coding) to compress and decompress files, built in [Rust](https://www.rust-lang.org/). It is a learning project and should not be used in production.

The compression ratios are not bad, and compare reasonably to existing file compression utilities (same order of magnitude compression ratios).

## Build, Install, Run

To build, navigate to the challenge directory and run 

```sh
$ cargo build --release
```

You'll then find the executale in `target/release/press`. You can also install the tool to uyour home directory using cargo as well:

```sh
$ cargo install --path .
```

## Limitations

1. Currently _very_ slow, especially at decompressing due to naive implementation.
2. All work is carried out in-memory, and so files larger than RAM cannot be compressed.
3. Has no run-length encoding so does not compress repeated symbols well.
4. Forces symbols to be 8-bit or EOF. Could be acheive better compression ratios with longer or tunable-length symbols.


## Improvements

- [ ] Optimise decoding logic for speed (Maybe use raw bytes as stored patterns instead of BitVecs to allow use of binary operations?).
- [ ] Refactor to a streaming or buffered implementation to allow files that are larger than memory.
- [ ] Enable logging with levels to improve debugging.
- [ ] Instrument code for timing compression and decompression benchmarks.
- [ ] Create simple test wrapper to benchmark against different inputs.
- [ ] Integrate with [Serde](https://serde.rs/) crate for (de)serialisation.
- [ ] Define a standard good-enough encoding using a corpus and include this as a default (compare sizes with default using distance metric on freq table vs custom and create a heuristic to decide whether to use custom or default encoding)

## Key learnings

- **Huffman Encoding**
- **Huffman encoding must also include EOT symbol** (or the size of the symbol stream) or the decoding logic won't be able to determine when to terminate and will decode uninitialised memory within the last word boudnary.
- **Chained constructors** (see `HuffmanEncoding` implementation) are useful to bootstrap objects and progressively simplify APIs
- **Protocol grammars are easily encoded** for example, protocol start and end symbols can be included when building the frequency table and we could artifically modify the Huffman tree to include protocol symbols with arbitrary size.
- The Huffman trees of two encoding can be easily merged to encode both in a single tree
