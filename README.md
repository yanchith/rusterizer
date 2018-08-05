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
- proc macro for varyings
- polish nalgebra interop (From/Into impls)
  * float normalization in From/Into impls?
  * float normalization in image?
- accept `IntoIterator<Item = Attribute>` instead of `&[Attribute]`
- backface culling
- triangle clipping

__Long term__

- postprocessing example
- geometry shaders
- think about data parallelism
