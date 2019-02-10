# ya rusterizer

_I am sure this is neither the first nor last rusterizer you will see_

A weekend project by a weekend rustacean. Nothing to see here yet.

## Building

This is `edition = "2018"`, so you need a rustc version that can compile that.
At the time of writing this `beta` already works.

Run examples with:

- `cargo run --release --example window <model path> <texture path>`
- `cargo run --release --example terminal <model path> <texture path>`

(you need to get the assets yourself, e.g. in the
[tinyrenderer](https://github.com/ssloy/tinyrenderer) repo)

## Roadmap

__Short term__

- `Pixel` as associated type
- nicer program API

``` rust
impl ShaderProgram for SimpleProgram {
    type Attribute = [f32; 3];
    type Varying = ();
    type Pixel = [f32; 4];

    fn vert(&self) -> ([f32; 4], Self::Varying) {
        // ...
    }

    fn frag(&self, varying: &Self::Varying) -> Self::Pixel {
        // ...
    }
}
```

- proc macro derive for Varying
- polish nalgebra interop (From/Into impls)
  * float normalization in From/Into impls?
  * float normalization in image?
- use nalgebra_glm?
- accept `IntoIterator<Item = Attribute>` instead of `&[Attribute]`
- triangle clipping

__Long term__

- postprocessing example
- geometry shaders
- think about data parallelism

## Prior art

- [tinyrenderer](https://github.com/ssloy/tinyrenderer) by Dmitry Sokolov
- [image](https://github.com/PistonDevelopers/image) by Piston Developers
- [rust-softrender](https://github.com/novacrazy/rust-softrender) by novacrazy
- [ternimal](https://github.com/p-e-w/ternimal]) by Philipp Emanuel Weidmann

Thanks to [Dmitry Sokolov](https://github.com/ssloy) for producing an excellent
guide on building software rasterizers.

Also thanks to the [image](https://github.com/PistonDevelopers/image) crate
developers for inspiring the image implementation used here.
