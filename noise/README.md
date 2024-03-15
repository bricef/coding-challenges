# `noise` Rust noise generator

## About

The noise utility generates coloured random noise in arbitrary dimensions. 

## Enhancements

## TODO

- [ ] Parameterised distribution functions
- [ ] 1D white noise
- [ ] FFT of white noise
- [ ] Spectral graph & Spectogram
- [ ] Coloured noise functions
- [ ] 2D Noise
- [ ] nD Noise

## Learnings

## Notes

1) Noise signals consist of a distribution of values over a range, 
which could be a gaussian, uniform or some other probability distribution.
This implies that our noise algorithms should have a configurable 
distribution function as input. 

2) The range and domains of a noise function should be configurable. For 
example, in audio noise, the range is typically over audible frequencies 
(20Hz to 20kHz) and the domain is in time. Whereas for 2 dimensional 
visual noise, the range is typically 0-255 and the domain a given 
rectangle of 2D coordinates.

3) For utility, the output format of our noise function should be 
configurable. For example, 2D noise could be output as an image, 1D noise
as an audio file with particular sampling frequency (typically 44100Hz). 
Internally, this could be set up as filters on the raw noise values.

4) Analysing the noise output is important to ensure the library works as 
expected. A tool to analyise the output should be created as an early step 
in the development process, and probably precede the actual data generation.

## References

- [Compute::distributions](https://docs.rs/compute/latest/compute/distributions/) for probabilty distribitions 
- [rustfft crate](https://docs.rs/rustfft/latest/rustfft/) for transforming colors of noise.
- [Spectral Density](https://en.wikipedia.org/wiki/Spectral_density)
- [Colors of Noise](https://en.wikipedia.org/wiki/Colors_of_noise)
- [Spectogram](https://en.wikipedia.org/wiki/Spectrogram)
- [Plotters plotting library](https://github.com/plotters-rs/plotters)