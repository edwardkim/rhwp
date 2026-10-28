# hayro-jpeg2000

[![Crates.io](https://img.shields.io/crates/v/hayro-jpeg2000.svg)](https://crates.io/crates/hayro-jpeg2000)
[![Documentation](https://docs.rs/hayro-jpeg2000/badge.svg)](https://docs.rs/hayro-jpeg2000)

<!-- cargo-rdme start -->

A memory-safe, pure-Rust JPEG 2000 decoder.

`hayro-jpeg2000` can decode both raw JPEG 2000 codestreams (`.j2c`) and images wrapped
inside the JP2 container format. The decoder supports the vast majority of features
defined in the JPEG2000 core coding system (ISO/IEC 15444-1) as well as some color
spaces from the extensions (ISO/IEC 15444-2). There are still some missing pieces
for some "obscure" features(like for example support for progression order
changes in tile-parts), but all features that actually commonly appear in real-life
images should be supported (if not, please open an issue!).

The decoder abstracts away most of the internal complexity of JPEG2000
and yields a simple 8-bit image with either greyscale, RGB, CMYK or an ICC-based
color space, which can then be processed further according to your needs.

## Example
```rust
use hayro_jpeg2000::{DecodeSettings, DecoderContext, Image};

let data = std::fs::read("image.jp2").unwrap();
let image = Image::new(&data, &DecodeSettings::default()).unwrap();

println!(
    "{}x{} image in {:?} with alpha={}",
    image.width(),
    image.height(),
    image.color_space(),
    image.has_alpha(),
);

let mut ctx = DecoderContext::default();
let decoded = image.decode(&mut ctx).unwrap();
let bitmap = decoded.data_u8();
```

If you want to see a more comprehensive example, please take a look
at the example in [GitHub](https://github.com/LaurenzV/hayro/blob/main/hayro-jpeg2000/examples/png.rs),
which shows you the main steps needed to convert a JPEG2000 image into PNG for example.

## Testing
The decoder has been tested against 20.000+ images scraped from random PDFs
on the internet and also passes a large part of the `OpenJPEG` test suite. So you
can expect the crate to perform decently in terms of decoding correctness.

## Performance
A decent amount of effort has already been put into optimizing this crate
(both in terms of raw performance but also memory allocations). However, there
are some more important optimizations that have not been implemented yet, so
there is definitely still room for improvement (and I am planning on implementing
them eventually).

Overall, you should expect this crate to have worse performance than `OpenJPEG`,
but the difference gap should not be too large.

## Safety
By default, the crate has the `simd` feature enabled, which uses the
[`fearless_simd`](https://github.com/linebender/fearless_simd) crate to accelerate
important parts of the pipeline. If you want to eliminate any usage of unsafe
in this crate as well as its dependencies, you can simply disable this
feature, at the cost of worse decoding performance. Unsafe code is forbidden
via a crate-level attribute.

## License
Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

<!-- cargo-rdme end -->
