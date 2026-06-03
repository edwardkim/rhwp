---
issue: 630
milestone: m100
branch: local/task630
created: 2026-05-06
status: 완료 — merge 대기
---

# Task #630 최종 결과 보고서

`aift.hwp` 4페이지 목차에서 `·` (U+00B7 MIDDLE DOT) 포함 라인의 `(페이지 표기)` 가 정확히 8.67px (반각 1자) 좌측으로 이탈하는 결함 정정.

## 1. 본질 결함 및 정정

### 1-1. 본질 결함

`is_halfwidth_punct` (`text_measurement.rs:859-862`) 가 `U+00B7` 을 강제 반각 (em/2) 처리. 한컴 저장 시점의 측정값은 전각 (em_size) 기반이므로 `tab_extended[0]` 가 전각 기준으로 산출됨. 본 환경의 반각 측정과 8.67px 차이 → right-tab 정렬 시 `·` 포함 라인이 8.67px 좌측으로 이탈.

### 1-2. 정정 (단일 룰)

**Diff** (`src/renderer/layout/text_measurement.rs:859-862`):

```rust
// Before
let is_halfwidth_punct = matches!(c,
    '\u{2018}'..='\u{2027}' |
    '\u{00B7}'                 // · MIDDLE DOT
);

// After (Issue #630)
// U+00B7 (가운뎃점) 은 본 분기에서 제외 — 한컴 저장본의 tab_extended 가
// 전각 측정 기반으로 산출되므로 반각 강제 시 right-tab 정렬이 8.67px
// 좌측 이탈. 폰트 메트릭 그대로 사용 (전각).
let is_halfwidth_punct = matches!(c,
    '\u{2018}'..='\u{2027}'
);
```

**케이스별 명시** (`feedback_hancom_compat_specific_over_general` 정합):
- `'\u{2018}'..='\u{2027}'` 범위 (스마트 따옴표 등) 는 보존 — 본 정정 영향 없음
- `U+00B7` 만 제외
- 폰트 메트릭이 narrow 로 저장한 폰트는 그 폭 그대로 사용

## 2. 정정 결과 (정량)

### 2-1. aift p4 권위 케이스

| 메트릭 | Before (Stage 1) | After (Stage 5) |
|--------|------------------|-----------------|
| 정렬 그룹 수 | 4 (592 / 600 / 601 / 293) | **1 (≈600~601)** |
| 8.67px 이탈 라인 | **6** (1-1, 3-1, 3-4, 4-1, 7-2, 8-1) | **0** |
| spread | 9.08px | **1.13px** ≤ 1.5 |

**`·` 포함 6 라인 모두 정확히 +8.67px 우측 이동 → 정합 회복** (1-1: 592.36 → 601.03, 3-1: 592.33 → 601.00, 3-4: 592.41 → 601.08, 4-1: 592.36 → 601.03, 7-2: 592.31 → 601.00, 8-1: 592.31 → 601.00).

`·` 미포함 라인은 영향 없음 (정정의 케이스별 명시 정합 입증).

### 2-2. 광범위 회귀 검증 (Stage 5)

| 항목 | 결과 |
|------|------|
| 164 fixture / 1614 페이지 회귀 | **0** |
| `cargo test --lib --release` | **1135 passed / 0 failed** (베이스라인 1134 → +1 GREEN) |
| 통합 테스트 16 파일 | **87 passed / 0 failed** (issue_630 추가) |
| `svg_snapshot` | **6 passed** (issue_147 골든 갱신, issue_267 KTX byte-identical) |
| Task #546 / #554 회귀 가드 | 통과 |
| issue_267 (KTX TOC, `·` 미출현) byte 차이 | **0** — 영향 없음 정합 입증 |

## 3. 단계별 진행 (5 stages)

| Stage | 내용 | 결과 |
|-------|------|------|
| 1 | 회귀 베이스라인 측정 | 1614 페이지 / 1134 cargo test / `·` 포함 6 라인 8.67px 이탈 정량 |
| 2 | 단위 테스트 작성 (RED) | 3 테스트 모두 RED 확인 |
| 3 | 정정 1 적용 (`·` 측정 통일) | spread 9.08 → 1.13 px / cargo test 1135 / svg_snapshot 5/6 |
| 4 | **정정 2 시도 → 회귀 발견 → 철회** | RIGHT 탭 정확 매치 시 23/24 라인 113px 이탈 → HWP5 의 `tab_extended[0]` 가 이미 right-tab 결과 위치로 저장됨을 발견. LEFT fallback 이 인코딩 의도와 정합. 코드 코멘트 240-243 의 진단이 정확. 정정 2 철회. |
| 5 | 광범위 회귀 검증 + 골든 갱신 | 페이지 수 회귀 0 / 1135+87 passed / issue_147 갱신 |

## 4. 메타 학습

### 4-1. 본질 가설 검증의 중요성

Stage 1 보고서의 가설:
- 원인 A: `·` 측정 불일치 (text_measurement.rs:863)
- 원인 B: native `tab_type = ext[2]` raw u16 → LEFT fallback (text_measurement.rs:247, 361)

**Stage 4 검증 단계에서 원인 B 가설 기각**. HWP5 의 `tab_extended[0]` 가 이미 right-tab 결과 위치 (= 우측 끝 - 한컴_seg_w) 로 저장되어 있어 LEFT fallback 이 정합. RIGHT 정확 매치 시 seg_w 이중 차감 → 113 px 좌측 이탈 회귀.

코드 코멘트 (line 240-243) 의 "기존 golden SVG 가 LEFT fallback 의존" 명시는 정확한 진단이었음. 의심하기 전에 검증 우선.

### 4-2. 합성 데이터 단위 테스트의 함정

`test_630_native_inline_tab_right_align` 은 합성 ext 데이터 (UNREGISTERED_FONT, 직접 인코딩) 로 RIGHT 매치 검증 → Stage 4 GREEN. 그러나 실제 HWP5 파일의 `ext[0]` 인코딩 의도와 다른 가정 → 통합 테스트에서 회귀 발견.

**TDD 흐름 + 통합 테스트 분리** 덕분에 회귀를 빠르게 발견하고 철회. 5 stage 분리 가치 입증 (`feedback_essential_fix_regression_risk` 정합).

### 4-3. 본 task 정정 후 잔여 결함 (별도 본질)

PDF (한컴 2022 출력) 와 정밀 비교 시 본 정정 후에도 모든 라인이 PDF 대비 약 1.05 mm 좌측에 위치 (우측 마진: PDF 0.11 mm vs SVG 1.16 mm). 이는 Stage 1 베이스라인부터 동일한 결함 — 본 task 정정 영향 없음.

근본 원인: `compute_char_positions` 의 in-run RIGHT 탭이 본문 우측 끝까지 클램프하지 않음 (`paragraph_layout.rs:1402` 의 cross-run handler 클램프 로직이 in-run 에 미적용). HWP TabDef.position = 95662 HU `/ 2.0 = 168.75 mm` 까지만 정렬, 본문 우측 끝 (170 mm) 까지 클램프 누락.

**별도 본질 영역으로 후속 task 등록**: **Issue #635** (아래 §6).

## 5. 산출물

### 5-1. 소스 변경

- `src/renderer/layout/text_measurement.rs`:
  - line 859-862: `is_halfwidth_punct` 매칭에서 `'\u{00B7}'` 제거 + 코멘트 갱신
  - line 240-243 + 354-356: Stage 4 검증 결과 코멘트 추가 (RIGHT 정확 매치 회귀 재발 방지)
  - tests mod: `test_630_middle_dot_full_width_in_registered_font` 추가
- `tests/issue_630.rs` (신규): `test_630_aift_p4_toc_paren_alignment` 통합 테스트
- `tests/golden_svg/issue-147/aift-page3.svg` 갱신 (+48 bytes)

### 5-2. 문서

- `mydocs/plans/task_m100_630.md` (수행계획서)
- `mydocs/plans/task_m100_630_impl.md` (구현계획서)
- `mydocs/working/task_m100_630_stage{1,2,3,4,5}.md` (5 단계별 보고서)
- `mydocs/report/task_m100_630_report.md` (본 최종 보고서)

### 5-3. 시각 판정 자료

- `output/svg/task630_before/aift_004.svg` (Stage 1 baseline)
- `output/svg/task630_before/KTX_002.svg` (영향 없음 검증)
- `output/svg/task630_stage3/aift_004.svg` (정정 1 단독 결과)
- `output/svg/task630_stage4/aift_004.svg` (정정 2 회귀 검증)
- `output/svg/task630_after/{aift,KTX,exam_kor,exam_math,k-water-rfp,kps-ai}/` (251 페이지)

## 6. 후속 태스크 등록

**Issue #635**: aift p4 right-tab 1.05 mm 잔여 마진 — TabDef.position in-run RIGHT 처리에서 본문 우측 끝 클램프 누락

본 Task #630 의 본질 결함 (8.67 px 이탈) 정정 후에도 잔존하는 1.05 mm 마진. 별도 본질 영역 (광범위 영향) 으로 등록. M100 후속.

## 7. 작업지시자 결정

작업지시자 안내: **(a) 옵션 — 본 Task #630 마무리** + 1 mm 마진 별도 후속 task 등록 (Issue #635).

본 task 의 본질 결함 (`·` 포함 라인 8.67 px 이탈) 정정 완료. 회귀 0. 광범위 sweep 통과.

## 8. Merge 절차

1. local/task630 → local/devel merge
2. local/devel → devel merge + push
3. Issue #630 close
4. orders/20260506.md 갱신
