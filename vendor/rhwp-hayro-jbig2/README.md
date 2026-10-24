# hayro-jbig2

[![Crates.io](https://img.shields.io/crates/v/hayro-jbig2.svg)](https://crates.io/crates/hayro-jbig2)
[![Documentation](https://docs.rs/hayro-jbig2/badge.svg)](https://docs.rs/hayro-jbig2)

A memory-safe, pure-Rust JBIG2 decoder.

`hayro-jbig2` decodes JBIG2 images as specified in ITU-T T.88 (also known as
ISO/IEC 14492). JBIG2 is a bi-level image compression standard commonly used
in PDF documents for compressing scanned text documents.

## Status

This crate is currently under development.

## Safety
By default, the crate has the `simd` feature enabled, which uses the
[`fearless_simd`](https://github.com/linebender/fearless_simd) crate to accelerate decoding of JBIG2 images with halftoning.
If you want to eliminate any usage of unsafe in this crate as well as its dependencies, you can simply disable this
feature and still get very good performance in the vast majority of cases. Unsafe code is forbidden
via a crate-level attribute.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
