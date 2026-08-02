# RHWP Bindings

This directory separates the shared native ABI from language-specific bindings.

## Native ABI

- `Native/`: Rust `cdylib` crate that exposes the C ABI used by bindings.
- `csharp/`: C# P/Invoke wrapper over the shared native library.
- `swift/`: Swift Package wrapper over the shared native library.

## CLI subprocess bindings

These wrap the `rhwp` CLI `--json` envelopes and the `mcp-serve` session protocol
instead of the C ABI. They are **repackaging, not a new surface**: no document
parsing, coordinate math, or verdict logic lives in the binding, because doing it
twice guarantees the two answers eventually disagree. Rationale for choosing the
subprocess surface is in
[`bindings_foundation.md`](../mydocs/tech/bindings_foundation.md) §2.

- `python/`: package `rhwp` — first binding (M18,
  [#3762](https://github.com/edwardkim/rhwp/issues/3762)). Three layers (stateless
  commands / session / plan) plus an envelope→dataclass generator driven by
  `export-ir-schema`.
  Status: **submitted, not merged** (PR
  [#3775](https://github.com/edwardkim/rhwp/pull/3775)) — the directory and its guide
  (`mydocs/manual/python_binding_guide.md`) both arrive when that PR lands.
- `node/`: package `@rhwp/node` — Node/TypeScript binding plus a browser (WASM)
  adapter behind the same client interface (M19,
  [#3776](https://github.com/edwardkim/rhwp/issues/3776)). Same three layers; the
  types are generated from `export-ir-schema` and `capabilities` so the compiler —
  not a hand-kept mapping table — checks envelope field names.
  Status: **in progress.** Guide:
  [`node_binding_guide.md`](../mydocs/manual/node_binding_guide.md).

`napi` native addons (and a C ABI for these two languages) are deliberately
deferred. They add a Node-ABI × OS × arch release matrix that is only worth paying
for once in-process performance is demonstrated to be the actual bottleneck; paying
first leaves an irreversible distribution surface behind if it never was.

Add new language bindings as sibling folders under `bindings/`.
