# `press`: a Huffman file compressor

## About

`press` is a simple file compression utility which uses [Huffman encodig](https://en.wikipedia.org/wiki/Huffman_coding) to compress and decompress files. It is a learning project and should not be used in production.

The compression ratios are not bad, and compare reasonably to existing file compression (same order of magnitude compression ratios).

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