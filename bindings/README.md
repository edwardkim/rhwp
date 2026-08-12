# RHWP Bindings

This directory separates the shared native ABI from language-specific bindings.

- `Native/`: Rust `cdylib` crate that exposes the C ABI used by bindings.
- `csharp/`: C# P/Invoke wrapper over the shared native library.
- `swift/`: Swift Package wrapper over the shared native library.

New language bindings are downstream by default. Adding one here requires explicit maintainer
adoption, a stable compatibility contract, and a tested long-term release path.
