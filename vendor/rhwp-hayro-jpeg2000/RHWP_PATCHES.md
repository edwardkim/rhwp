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

## #8174 PDF 색 공간 우선순위

`DecodeSettings.pdf_colour_components`가 있으면 PDF dictionary의 색 성분 수와 선택적 opacity를
실제 decode 검증에 사용한다. raw J2K에 임의의 RGB/Gray metadata를 합성한 뒤 strict 모드가
정상 index+alpha·CMYK+alpha를 거부하지 않게 한다. PDF 색 공간을 선언했을 때 JP2의 별도
색 변환을 중복 적용하지 않는다. 독립 codestream 사용의 기본값은 `None`으로 유지한다.
기존 요청별 예약·반복·실패 전파는 유지한다.
