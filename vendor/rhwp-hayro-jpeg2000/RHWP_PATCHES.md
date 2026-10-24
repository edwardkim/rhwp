# rhwp-hayro-jpeg2000

rhwp internal resource-bounded fork of hayro-jpeg2000 0.4.0.
Not an official hayro release.
Upstream: https://github.com/LaurenzV/hayro
Original authors: Laurenz Stampfl <laurenz.stampfl@gmail.com>
License: Apache-2.0 OR MIT
Original license texts and source hashes are preserved.

Request-scoped allocation and work checks for the rhwp PDF import path.
The `rhwp-bounded` feature enables synchronous request scopes. Data-sized Vec
reservations use checked arithmetic and try_reserve_exact before growth. Copies,
shared dictionaries, image planes, scratch buffers, and sorting consume the same
request. Bulk pixel transforms precharge work; entropy/packet loops also check
work while decoding. Work units are logical counters, not time or instructions.
The scope is synchronous and thread-local, restores nested callers, and is not
Send/Sync. Internal fallible constructors and helper signatures have changed.
Public image parsing/decode entry points retain their signatures. The JPEG 2000
fork adds try_data_u8() for bounded output and num_components() for pre-allocation
validation; RHWP never calls the infallible data_u8() convenience method.

The RHWP graph disables the upstream default image integration and SIMD features.
Those optional integration paths and legacy calls without a request scope are
outside the request-bound contract. Fixed-size Rc/Arc/iterator metadata still uses
normal Rust allocation; this does not make allocator exhaustion recoverable.
Only input-sized codec storage and accounted work are controlled here. Existing
whole-parser and process memory limitations remain.
This work is tracked by rhwp issue #8183 and does not prove process-wide memory or CPU bounds.
