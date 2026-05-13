# Task #877 Stage 1 완료 보고서 — 방어성 가드 (allocation sanity check)

**관련 계획서**: [task_m100_877_impl.md](../plans/task_m100_877_impl.md)
**브랜치**: `local/task877`

## 작업 내용

### 1. 공통 helper 도입 ([src/parser/hwp3/mod.rs:46-76](../../src/parser/hwp3/mod.rs#L46-L76))

```rust
pub(crate) const HWP3_MAX_RECORD_SIZE: usize = 256 * 1024 * 1024; // 256 MB

pub(crate) fn alloc_record_buf(length: usize) -> Result<Vec<u8>, io::Error> { ... }
pub(crate) fn check_record_count(count: usize) -> Result<(), io::Error> { ... }
```

- `alloc_record_buf`: `vec![0u8; length]` 직접 호출 대신 cap 검증 후 할당
- `check_record_count`: `Vec::with_capacity` 인자 검증 (point/cell count 등 비-u8 element 용)
- cap 초과 시 `io::Error(InvalidData, "HWP3 record count overflow: ...")` 반환

### 2. 가드 적용 위치

| 파일 | 위치 | 수정 전 | 수정 후 |
|------|------|---------|---------|
| records.rs:392 | `Hwp3InfoBlock::read` (u16 length, max 64KB but 일관성) | `vec![0u8; length]` | `alloc_record_buf(length)?` |
| records.rs:413 | `Hwp3AdditionalInfoBlock::read` (**원래 panic 지점**, u32 length) | `vec![0u8; length]` | `alloc_record_buf(length)?` |
| mod.rs:694 | 표/textbox cell_buf (`27 × cell_count`) | `vec![0u8; ...]` + `break` | `alloc_record_buf(...)` match err → break |
| mod.rs:932 | picture(ch=11) `ext_buf` (n_ext from info_buf[0..4]) | `vec![0u8; n_ext]` + `break` | `alloc_record_buf(n_ext)` match err → break |
| drawing.rs:372 | `Hwp3DrawingTextBox::read` (info2_len) | `vec![0u8; info2_len]` | `alloc_record_buf(info2_len)?` |
| drawing.rs:346 | `Hwp3DrawingPolygon::read` (point_count u32, `Vec<[i32;2]>::with_capacity`) | `Vec::with_capacity(point_count)` | `check_record_count(point_count)?` 추가 |
| drawing.rs:399 | `Hwp3DrawingCurve::read` (동일) | 동일 | 동일 |
| drawing.rs:451-458 | `Hwp3DrawingExtendedPolygon::read` (point_count + line_attrs) | `vec![0u8; point_count]` + `Vec::with_capacity` | `check_record_count` + `alloc_record_buf` |
| drawing.rs:543-547 | drawing unknown object (info1_len/info2_len u32) | `vec![0u8; ...]` × 2 | `alloc_record_buf(...)?` × 2 |
| ole.rs:38 | `Hwp3OleInfo::read` (total_length - 4) | `vec![0u8; ...]` | `alloc_record_buf(...)?` |

`ch=29` ([mod.rs:1143](../../src/parser/hwp3/mod.rs#L1143)) 의 기존 `< 1000000` 검증은 본 cap (256MB) 보다 엄격하므로 그대로 유지.

### 3. 단위 테스트 추가 ([mod.rs:tests](../../src/parser/hwp3/mod.rs))

- `test_alloc_record_buf_overflow_returns_err`: `HWP3_MAX_RECORD_SIZE + 1`, `0xDC000000` (sample16 실측 garbage 값) → graceful Err
- `test_alloc_record_buf_within_cap_ok`: 정상 범위 (1024) → Ok
- `test_check_record_count_overflow_returns_err`: count 가드 검증
- `test_hwp3_sample16_load_without_panic`: sample16 로드 시 panic 없음 (Ok/Err 무관, panic 검증이 본질)

## 검증 결과

### 빌드
```
$ cargo build --release
    Finished `release` profile [optimized] target(s) in 19.44s
```

### 단위 테스트
```
$ cargo test --release --lib parser::hwp3
running 7 tests
... (7개 전부 ok)
test result: ok. 7 passed; 0 failed; 0 ignored
```

### 전체 cargo test
```
$ cargo test --release
test result: ok. 1234 passed; 0 failed; 2 ignored (lib)
+ integration test 36개 묶음 전부 ok
```

### sample16 panic 사라짐 확인
```
$ cargo run --release --bin rhwp -- dump samples/hwp3-sample16.hwp
... (32093줄 정상 출력)
=== 완료: 1 구역, 77 문단 ===
```

**Stage 1 이전 (panic/error)**:
```
오류: HWP 파싱 실패 - 유효하지 않은 파일: HWP 3.0 오류: 입출력 오류가 발생했습니다: failed to fill whole buffer
```

**Stage 1 이후 (graceful 부분 파싱)**: 1 구역 / 77 문단 인식. panic 없음.

> 단, sample16 의 64쪽 분량은 여전히 정확히 인식되지 않음 (현재 28737 페이지로 인식 — pagination 비정상). 이는 picture(ch=11) byte alignment 문제로 인한 paragraph stream misalign 의 후속 영향이며, **Stage 2 에서 해결할 본 task 의 근본 원인**.

### 다른 HWP3 sample 회귀 없음 (Stage 1 전후 동일)

| 샘플 | 문단 수 |
|------|---------|
| hwp3-sample.hwp | 195 |
| hwp3-sample10.hwp | 26767 |
| hwp3-sample13.hwp | 71 |
| hwp3-sample14.hwp | 256 |
| hwp3-sample4.hwp | 1273 |
| hwp3-sample5.hwp | 1931 |

## 환경별 동작 (예상)

- **네이티브 64-bit**: 기존엔 `vec![0u8; 3.69GB]` 후 read EOF Err. 이제는 더 빠른 시점에 cap 검증 Err. 결과 동일하나 cost 감소.
- **WASM 32-bit**: 기존엔 `RawVec capacity overflow` panic → `unreachable` trap. 이제는 graceful Err.

## 변경 파일

- `src/parser/hwp3/mod.rs` — helper 도입 + ext_buf/cell_buf 가드 + 4개 unit test
- `src/parser/hwp3/records.rs` — InfoBlock/AdditionalInfoBlock 가드
- `src/parser/hwp3/drawing.rs` — TextBox/Polygon/Curve/ExtendedPolygon/Unknown 가드
- `src/parser/hwp3/ole.rs` — OleInfo 가드

## 다음 단계

Stage 2 (picture ch=11 byte alignment 정합) 진행 예정 — sample16 의 단일 paragraph 조기 종료 원인 해결 → 64쪽 전체 정합 파싱.
