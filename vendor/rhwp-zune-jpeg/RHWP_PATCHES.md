# rhwp request-bounded JPEG security patch

Upstream: zune-jpeg **0.5.15** (licenses and original manifest retained;
per-file source hashes in `UPSTREAM.json`). Local package: `rhwp-zune-jpeg`
**0.5.15-rhwp.1**, `publish = false`. zune-core remains **0.5.3**.
This is a separate dependency of the optional PDF importer. The ordinary image
crate continues to use registry zune-jpeg. No decoder feature or SIMD algorithm
was removed. Source formatting follows the enclosing rhwp workspace.

## Changed contract

`jpeg_limited::decode` installs a borrowed request budget before parsing any
header. The same decoder carries it through metadata, component storage,
baseline multi-scan / progressive coefficient planes, MCU rows, upsampling
buffers and worker resize. Checked byte counts are charged **before**
`try_reserve_exact`; denial and allocation failure return `AllocationLimit`.
The importer checks its sticky budget before translating other JPEG errors.
A growing vector charges its complete new requested reservation, conservatively
including data that may be reallocated in place. This is cumulative reservation
accounting, not a measurement of RSS or allocator bookkeeping.

Repeated/replaced EXIF, IPTC, XMP, ICC, MPF and gain-map payloads retain their
cumulative cost. Extended XMP assembly still requires matching GUID/declared
size and contiguous, complete offsets. Its sort is allocation-free; length
arithmetic is checked. An incomplete declaration does not reserve its full size.

The existing non-strict decode policy and RGB/Gray color selection are retained.
Output pixels are separately reserved by the importer before `decode_into`.
Its owned JPEG input copy is still charged at `jpeg_input::Input::read_to_end`.

## Scope and remaining boundaries

- This contract applies to the importing `ZCursor<&[u8]>` + `decode_headers` /
  `decode_into` path. `info()` clones and `icc_profile()` assembly are upstream
  APIs outside that path and are **not** covered by this claim.
- Error formatting, fixed-size codec structures, arbitrary user-provided reader
  implementations, allocator overhead, process heap and CPU isolation are not
  proven by this hook.
- CMYK PDF images retain the separate Adobe-aware `jpeg-decoder` 0.3.2 path.
  Its internal metadata/plane/worker allocation audit remains open.
- `parser_allocation_bounds_proven` stays **false**. This patch is not evidence
  that PDF import is ready for untrusted input distribution.

## Validation

See `mydocs/working/task_m100_7911_stage2.md` section 3.19 for exact commands,
Native/fresh Docker WASM results, source hashes and unexecuted boundaries.
Resource regressions live in `tests/cases/issue_7911_pdf_resource_limits.rs`.
Synthetic JPEG fixtures are codec contracts, not Hancom rendering baselines.
