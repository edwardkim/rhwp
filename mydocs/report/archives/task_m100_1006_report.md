# Task #1006 최종 결과 보고서 — 쪽 테두리 포맷별 분리 + 변환본 cover logo overlap 해소

이슈: [#1006](https://github.com/edwardkim/rhwp/issues/1006)
배경: PR [#956](https://github.com/edwardkim/rhwp/pull/956) 회귀

## 1. 목표

`samples/hwp3-sample16-hwp5.hwp` cover page (page 1) 의 머리말 logo 가 외곽선과 시각 겹침 해소. 동시에 HWP3 #987 정합과 HWP5/HWPX #956 정합을 모두 유지.

## 2. 결과

| 항목 | Before | After |
|------|--------|-------|
| HWP3 sample16 외곽선 | (37.76, 55.64)~(755.95, 1047.93) body-based — logo top 과 outline top 거의 일치 | paper-based (18.93, 17.88)~(774.77, 1047.93) — logo 내부 포함 (Hancom 정합) ✓ |
| HWP5 변환본 cover 외곽선 | (56.69, 75.62)~ — body_area.y clip → logo overlap | paper-based (18.93, 17.88)~(774.77, 1047.93) — logo 포함 ✓ |
| HWP5 시험지 외곽선 | (26.45, 90.71)~ paper-based | 동일 ✓ (#956 정합 유지) |
| 전체 sample 페이지 수 | — | 모두 보존 (회귀 없음) |
| `cargo test` | 1303 passed | 1303 passed ✓ |
| `cargo clippy` | 0 warnings | 0 warnings ✓ |

## 3. 핵심 fix — 포맷별 contract 분리

### 3-1. 모델 (`src/model/page.rs`)

```rust
pub struct PageBorderFill {
    // ... 기존 필드
    pub basis: PageBorderBasis, // [Task #1006] 신규
}

pub enum PageBorderBasis {
    #[default]
    BodyBased,  // HWP3
    PaperBased, // HWP5/HWPX
}
```

### 3-2. 파서 (HWP3/HWP5/HWPX 명시적 contract)

- HWP3 (`src/parser/hwp3/mod.rs`): `basis: PaperBased` 명시 (작업지시자 Hancom Office close-up 시각 판정 — #987 의 body-based 재판정 정정)
- HWP5 (`src/parser/body_text.rs`): `pbf.basis = PaperBased` 명시 (PR #956 spec)
- HWPX (`src/parser/hwpx/section.rs`): `pbf.basis = PaperBased` 명시 (PR #956 spec)

세 포맷 모두 동일하게 PaperBased 로 통합. 향후 신규 포맷 추가 시 명시적으로 `basis` 를 결정하도록 강제.

### 3-3. 렌더러 (`src/renderer/layout.rs`)

```rust
// Before (attr bit 0 단일 해석 — 회귀 사이클의 원흉):
let paper_based = (pbf.attr & 0x01) != 0;

// After:
use crate::model::page::PageBorderBasis;
let paper_based = matches!(pbf.basis, PageBorderBasis::PaperBased);
```

추가로 이전 #1001 의 `!header_inside` / `!footer_inside` conditional clip 을 paper-based 에서 제거 → outline 이 항상 paper-spacing edge 까지 확장 (Hancom Office 정합). 작업지시자 시각 판정 ("그림을 이동하면 외곽선이 줄어드는 현상") 으로 머리말 객체 존재 여부에 따른 conditional clip 의 회귀 발견 → 무조건 paper-edge 확장 으로 통합.

## 4. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/model/page.rs` | `PageBorderFill::basis` + `PageBorderBasis` enum |
| `src/parser/hwp3/mod.rs` | `basis: BodyBased` 주입 |
| `src/parser/body_text.rs` | `basis = PaperBased` 주입 |
| `src/parser/hwpx/section.rs` | `basis = PaperBased` 주입 |
| `src/renderer/layout.rs::build_page_borders` | basis 필드 사용 + 변환본 cover header logo 보호 |
| `src/renderer/layout.rs::detect_header_object_bottom` | 신규 helper |

## 5. 회귀 sweep

| 파일 | 페이지 수 | 회귀 |
|------|----------|------|
| hwp3-sample10.hwp | 763 | 없음 |
| hwp3-sample14.hwp | 11 | 없음 |
| hwp3-sample16.hwp | 64 | 없음 (#987 baseline 유지) |
| hwp3-sample16-hwp5.hwp | 64 | 없음 |
| exam_kor.hwp | 20 | 없음 |
| aift.hwp | 74 | 없음 |
| biz_plan.hwp | 6 | 없음 |
| 3-09월_교육_통합_2022.hwp (HWP5) | 21 | 없음 (#956 baseline 유지) |

## 6. 회귀 사이클 종결

| Task/PR | 접근 | sample16 | 시험지 | 변환본 logo |
|---------|------|----------|--------|------------|
| task877 | `paper_based = (attr & 0x01) != 0` | ✓ | 회귀 | 회귀 |
| #920 | `paper_based = (attr & 0x01) == 0` | 회귀 | ✓ | 회귀 |
| #956 | `paper_based = true` 전역 | 회귀(잘못된 재판정) | ✓ | overlap (header 영향 X) |
| #987 | bfid 정정 + attr 존중 복원 | body 재판정 (오판단) | 일부 ✓ | 회귀 (#1006) |
| **#1006** | **포맷별 basis 분리 + 모두 PaperBased + cover logo 보호** | **✓** | **✓** | **✓** |

attr bit 0 단일 해석으로 인한 회귀 사이클을 명시적 contract 분리 + 작업지시자 Hancom Office close-up 시각 판정으로 종결.

## 7. 검증 trail

- Stage 1: 모델/파서 contract 분리 ([`stage1`](../working/task_m100_1006_stage1.md))
- Stage 2: 렌더러 분기 정정 + 변환본 cover logo 보호 ([`stage2`](../working/task_m100_1006_stage2.md))

## 8. 결론

본 task 로 attr bit 0 단일 해석에 따른 outline 회귀 사이클 종결. HWP3/HWP5/HWPX 가 각자 자신의 spec 에 맞는 contract 를 명시하고, 렌더러는 공통 분기만 유지. 변환본 cover page header logo overlap 도 해소.
