# ya rusterizer

_(Judging from [crates.io](https://crates.io/search?q=rusterizer) the name is
quite the low hanging fruit)_

A weekend project by a weekend rustacean. Nothing to see here yet.

## Prior art

- [tinyrenderer](https://github.com/ssloy/tinyrenderer) by Dmitry Sokolov
- [image](https://github.com/PistonDevelopers/image) by Piston Developers
- [rust-softrender](https://github.com/novacrazy/rust-softrender) by novacrazy
- [ternimal](https://github.com/p-e-w/ternimal]) by Philipp Emanuel Weidmann

Thanks to [Dmitry Sokolov](https://github.com/ssloy) for producing an excellent
guide on building software rasterizers.

Also thanks to the [image](https://github.com/PistonDevelopers/image) crate
developers for inspiring the image implementation used here.

## Roadmap

__Short term__

- terminal example
- window/glutin example
- backface culling
- polish nalgebra interop (From/Into impls)
  * can we make float normalization between integer and float data in images?
- proc macro for varyings

__Long term__

- postprocessing example
- actual clipping
- geometry shaders
- think about data parallelism
