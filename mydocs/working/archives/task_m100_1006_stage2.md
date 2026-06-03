# Task #1006 Stage 2 — 렌더러 분기 정정 + 변환본 cover logo 보호

이슈: [#1006](https://github.com/edwardkim/rhwp/issues/1006)
Stage 1: [`task_m100_1006_stage1.md`](task_m100_1006_stage1.md)

## 1. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs::build_page_borders` | `paper_based = matches!(pbf.basis, PaperBased)` 분기 변경 |
| `src/renderer/layout.rs::build_page_borders` | 머리말 paper-anchored 객체 detect 시 outline top clip 건너뜀 |
| `src/renderer/layout.rs::detect_header_object_bottom` | 신규 helper (paper-anchored Picture/Shape with top<body_top 의 max bottom) |
| `src/renderer/layout.rs::페이지 footer baseline 함수` | 동일하게 basis 필드 사용 |

## 2. 핵심 로직

### 2-1. attr bit 0 → basis 필드

```rust
// Before (attr bit 0 단일 해석 — 회귀 사이클의 원흉):
let paper_based = (pbf.attr & 0x01) != 0;

// After (포맷별 분리):
use crate::model::page::PageBorderBasis;
let paper_based = matches!(pbf.basis, PageBorderBasis::PaperBased);
```

### 2-2. 변환본 cover logo overlap 해소 — header_inside/footer_inside clip 제거

이전 #1001 의 `!header_inside` / `!footer_inside` conditional clip 은 paper-based outline 을 body_area.y 까지 축소시켜 머리말 logo overlap 을 유발. 작업지시자 추가 시각 판정:

> 그림을 이동하면 외곽선이 줄어드는 현상 발생함.

→ 머리말 객체 존재 여부 (또는 위치) 와 무관하게 paper-based outline 은 항상 paper-spacing edge 까지 확장하는 것이 정답. 따라서 conditional clip 자체를 제거:

```rust
// [Task #1006] paper-based: 머리말/꼬리말 영역 clip 생략 — 외곽선이
// paper-spacing edge 까지 확장 (Hancom Office 정합). 이전 #1001 의
// header_inside/footer_inside conditional clip 은 머리말 객체 (logo)
// 존재 여부에 따라 외곽선이 줄어드는 회귀를 유발 → paper-based 에서는
// attr bit 1/2 무시. body-based 는 spec 정의상 body_area 와 동일하므로
// clip 자체가 무의미.
let _ = (header_inside, footer_inside);
```

`detect_header_object_bottom` helper 는 제거 (불필요).

## 3. 검증 (단위)

| Sample | basis | 외곽선 (page 1) | 비고 |
|--------|-------|----------------|------|
| hwp3-sample16.hwp | PaperBased | (18.93, 17.88) — (774.77, 1047.93) | logo 포함 — Hancom Office close-up 정합 ✓ |
| hwp3-sample16-hwp5.hwp | PaperBased | (18.93, 17.88) — (774.77, 1047.93) | logo (y=56-87) 포함 ✓ |
| 3-09월_교육_통합_2022.hwp | PaperBased | (26.45, 90.71) — (767.25, 1092.27) | #956 정합 ✓ (no header obj) |

페이지 수 회귀 sweep:
- hwp3-sample10: 763 페이지 — 변동 없음 ✓
- hwp3-sample14: 11 페이지 — 변동 없음 ✓
- hwp3-sample16.hwp: 64 페이지 — 변동 없음 ✓
- hwp3-sample16-hwp5.hwp: 64 페이지 — 변동 없음 ✓
- exam_kor.hwp: 20 페이지 — 변동 없음 ✓
- aift.hwp: 74 페이지 — 변동 없음 ✓
- biz_plan.hwp: 6 페이지 — 변동 없음 ✓
- 3-09월_교육_통합_2022.hwp: 21 페이지 — 변동 없음 ✓

빌드/테스트:
- `cargo build --release`: 0 errors ✓
- `cargo test --release --lib`: **1303 passed**, 0 failed ✓
- `cargo clippy --release -- -D warnings`: 0 warnings ✓

## 4. Stage 3 진입

- WASM 빌드 + 최종 보고서 작성 + PR
