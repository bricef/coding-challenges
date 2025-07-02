# Large file code challenge

A very large CSV file that can't fit into memory represents 2D data.

Some values are not numbers. They must be replaced by interpolated values (the average of the 4 direct neighbours) in the output file.

Write a CLI tool which takes in two arguments. First the name of the input file, second the name of the output file. Create the output file with the replaced non-numeric values.

## Brice's Assumptions

1. Individual rows are too large to keep in memory. (Extremely large files)
2. If a cell has only NaN neighbours, it's value will be mapped to 0. (I don't actually like this because it's bad semantically for almost all 2D data, but it seems like an easy first option.)
3. If a cell has _some_ NaN neighbours, its value will be the average of the other neighbours only.
4. Lines are terminated with `\n`
6. Input file smaller than 2^64 bytes (18 Exabytes) - (addressable space, possible on ZFS or Btrfs).
7. CSV is properly formatted. For example, all values are delimited, no empty lines between rows, no space between digits in a number, etc... It would pass CSV validation.
8. Valid number formats for the CSV files are integer not separated by spaces ("12345" rather than "12 345") or floating point values ("123.456"). Any number not parseable as such will be considered to be NaN. All input data will be interpreted within the bounds of IEE 754 64 bit floating point numbers.
9. Individual CSV datums are small enough to fit into memory.

## Method

The processing is carried out using similar methods to [how a kernel (or convolution matrix) is used in image processing](https://en.wikipedia.org/wiki/Kernel_(image_processing)). A small sample from the input centered on a particular pixel is used and processed as a sub-image, the output of this processing giving the result for the center pixel in the output image. The benefit of this approach is that the algorithm is embarassingly parallelisable. Unlike convolution matrices, in this case, we only apply the processing step if a value is missing, rather than to all datums.

In practice, because we can't predict the columnar data width in a CSV file (the way we would in an image with fix sized field width for pixels), we must process the image in thin ribbons that correspond to going along the rows of the CSV.

Edge datums are normalised based on their available neighbours (Kernel Crop) rather than using a more complex algorithm

## further work

**Better NaN handling:** If a cell has only NaN neighbours, grow neighbourhood until this isn't true. If you grow out of bounds, stop and error out. Essentially, this would be a dynamically sized convolution matrix.

**Implement as a graphics shader:** if the values are representable as: signed 32 bit integer, unsigned 32 bit integers, single precision IEEE 754 floats or double precision IEEE 754 floats, (which we're assuming) the entire algorithm can be implemented as a shader and executed on a GPU extremely efficiently.

**Specify a numeric type used:** The specification

## Notes

This will perform quite poorly on small files due to the multiplicity of IO calls to read the file. I do not attempt to build a heuristic to determine if a better algorithm could be used given the size of the lines. Given that bad IO characteristics doesn't translate to significant real time on small and medium files, that seems like an acceptable balance. It should be blindingly fast on extremely large files though.

Storage access patterns benefit from random access memory, performance is likely to be significantly worse on sequential access memory.
