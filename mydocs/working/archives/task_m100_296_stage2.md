# Task #296 Stage 2 보고서 — 구현

## 목표

Stage 1 조사로 확정된 수정 방향 (옵션 A: WASM 측정기에 inline_tabs 분기 추가) 구현.

## 구현 절차

### (1) 헬퍼 함수 추가

`src/renderer/layout/text_measurement.rs` 상단 (`find_next_tab_stop` 앞):

```rust
/// inline_tabs ext[2] 에서 탭 종류를 추출.
///
/// HWP `tab_extended` 포맷 (PR #292 / Task #290 실증):
/// - high byte = 탭 종류 enum+1 (1=LEFT, 2=RIGHT, 3=CENTER, 4=DECIMAL)
/// - low  byte = fill_type
#[inline]
pub(super) fn inline_tab_type(ext: &[u16; 7]) -> u8 {
    ((ext[2] >> 8) & 0xFF) as u8
}
```

### (2) `WasmTextMeasurer::estimate_text_width` 에 분기 추가

`EmbeddedTextMeasurer` 의 `tab_char_idx` 관리 방식을 복사하되, 탭 종류 판정은 `inline_tab_type(ext)` 로 고바이트 추출. match arm 은 새 포맷에 맞게 재작성 (`2 => RIGHT, 3 => CENTER, _ => LEFT/DECIMAL`).

### (3) `WasmTextMeasurer::compute_char_positions` 에 동일 분기 추가

`estimate_text_width` 와 동일 구조. `tab_char_idx` 변수 신규 도입.

### (4) 네이티브 측정기는 건드리지 않음

`EmbeddedTextMeasurer` 의 `tab_type = ext[2]` 는 유지. 기존 golden SVG (issue-147 "(페이지표기)", issue-267 KTX 목차) 가 "우연한 LEFT 폴백" 동작에 의존하고 있어, 네이티브 측 수정 시 2건 회귀 발생 확인. 주석으로 Task #296 범위 외임을 명시.

**범위 축소 결정 근거**: Stage 2 중간에 네이티브도 `inline_tab_type` 으로 수정했을 때 svg_snapshot 2건 FAIL 확인 → 범위 축소로 전환. 이번 PR 의 목적 (브라우저 #18 좌측 정렬) 은 WASM 수정만으로 달성 가능.

### (5) 단위 테스트 4건 추가

`src/renderer/layout/tests.rs` 하단:
- `task296_inline_tab_type_left` — ext[2]=0x0100 (실측 exam_math #18 케이스) → 1
- `task296_inline_tab_type_right` — ext[2]=0x0203 (PR #292 실측 저작권\t1 케이스) → 2
- `task296_inline_tab_type_center` — ext[2]=0x0300 → 3
- `task296_inline_tab_type_decimal` — ext[2]=0x0400 → 4

## 변경 파일 (Stage 2 종료 시점)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/text_measurement.rs` | 헬퍼 `inline_tab_type` 추가 + WasmTextMeasurer 두 함수에 inline_tabs 분기 신규 + 진단 로그 (Stage 3에서 제거) |
| `src/renderer/layout/tests.rs` | 단위 테스트 4건 |

## 로컬 검증 (Stage 2 종료 시점)

| 항목 | 결과 |
|------|------|
| `cargo test --lib task296` | ✅ 4 passed |
| `cargo test --lib task290` | ✅ 5 passed (#290 회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6 passed (기존 골든 유지) |
| `cargo test --test tab_cross_run` | ✅ 1 passed |
| `cargo test --lib` 전체 | ✅ 992 passed / 0 failed / 1 ignored |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ clean |

## Stage 2 중간 이슈 (기록용)

초기 설계 (수행계획서 옵션 A 원안) 는 네이티브 측정기도 함께 수정하는 것이었음. 이를 Stage 2-3 에서 구현했으나 `svg_snapshot` 2건 FAIL 발생:
- `issue_147_aift_page3`: "(페이지표기)" 우측 정렬 → 좌측으로 이동
- `issue_267_ktx_toc_page`: LAYOUT_OVERFLOW 16.8px 발생

이는 기존 골든이 `tab_type = ext[2]` 전체 u16 해석의 "우연한 LEFT 폴백" 동작에 의존 중임을 시사. 한컴 PDF 대조로 올바른 동작 확정이 필요한 사안 → Task #296 범위 축소하고 별도 후속 이슈 등록 예정.

## 다음 단계

- Stage 3: WASM Docker 빌드 → 브라우저 시각 검증 → 진단 로그 제거 → 회귀 재확인
