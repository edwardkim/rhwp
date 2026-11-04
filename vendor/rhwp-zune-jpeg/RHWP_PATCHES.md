# rhwp request-bounded JPEG security patch

Upstream: zune-jpeg **0.5.15** (licenses and original manifest retained;
per-file source hashes in `UPSTREAM.json`). Local package: `rhwp-zune-jpeg`
**0.5.15-rhwp.1**, `publish = ["crates-io"]`. zune-core remains **0.5.3**.
This is a separate dependency of the optional PDF importer and the Print JPEG
transfer. The ordinary image
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

## Frame sampling geometry compatibility (#8174 T06)

Resolve the maximum horizontal and vertical sampling factors across **all** SOF
components before deriving the MCU grid and any component's padded row stride.
A running maximum gave an early component a different grid from the progressive
coefficient planes when a later component had a larger sampling factor. Keep the
progressive bounds check; valid images must use a consistent grid, rather than
bypassing the malformed-image guard. Derive component height from its vertical
factor as well (the field is currently metadata, not the IDCT row consumer).

The same setup serves baseline and progressive decoding. Existing cjpeg/djpeg
controls cover both encodings, small/padded edges and mixed component sampling.
The request allocator and all growth-before-reservation hooks remain in place.
See `mydocs/working/task_m100_8174_t06_completion.md` for source lineage, actual
Native/WASM product and independent-consumer verification, pixel comparisons,
resource failure/recovery and visual evidence. Interpolation/IDCT differences
from djpeg are reported separately from sampling geometry and content retention.
## Print 인스턴스의 선택 변환 (#8153)

기본 decoder의 IDCT·보간 선택은 유지한다. `set_transforms`를 헤더 파싱 전에 호출한
인스턴스만 호출자가 제공한 IDCT와 보간을 사용한다. 같은 IDCT를 DC/저주파 shortcut에도
설치하며, 원래의 metadata/MCU/progressive 할당 예산을 그대로 사용한다.

선택 보간에는 패딩 폭이 아닌 실제 성분 폭을 전달한다. 마지막 실제 성분 행을 MCU 패딩에
복제한 뒤, 기존의 MCU 간 문맥·출력 버퍼를 사용한다. 선택하지 않은 인스턴스는 그 버퍼와
기존 함수만 사용하며 끝 표본 복제를 추가하지 않는다. 기본 PDF importer는 이 API를 호출하지 않는다.

`decoder.rs`의 헤더 정보 타입을 `image_info.rs`로 옮겼지만 기존 타입 경로를 재수출한다.
보간 설정과 선택 호출은 `transforms.rs`로 분리해 수정 파일을 1000줄 이하로 유지했다.
공개 IJG 보간을 Rust로 재표현한 제품 소스는
`src/ijg_upsampler.rs`이며, 원 배포의 README와 고지는
이 디렉터리의 `README.ijg` 및 해당 소스에 보존한다.

This software is based in part on the work of the Independent JPEG Group.

## 최종 Print JPEG 재생 (#8153)

`src/ijg_replay.rs`는 libjpeg-turbo 3.1.2의 `jidctint.c` 8비트 ISLOW IDCT와
`jdcolor.c` 16비트 YCbCr→RGB를 안전한 정수 연산으로 재표현한다. 원 저작권은 해당
파일에, 배포 조건은 `README.ijg`에 보존한다. 기존 IJG 보간과 함께 명시적으로 선택한
최종 Print JPEG 재생 인스턴스에만 적용한다. 기본 decoder와 원본 사진을 압축 전에
변환하는 기존 Print IDCT 선택은 유지한다. IDCT는 MCU shortcut이 일부만 초기화하는
입력 버퍼를 덮어쓰지 않는다. 각 pass의 정규화와 색 변환의 반올림은 공개 원문을 따른다.
