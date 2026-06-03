---
issue: 630
milestone: m100
branch: local/task630
stage: 5 — 광범위 회귀 검증 + 시각 판정 자료 정비
created: 2026-05-06
status: 완료 — 승인 대기
---

# Task #630 Stage 5 완료 보고서 — 광범위 회귀 검증

## 1. 광범위 페이지네이션 sweep (164 fixture / 1614 페이지)

```
fixtures: 164  total_pages: 1614
diff Before/After: 0 lines (페이지 수 회귀 0)
```

`samples/` 폴더 전체 164 fixture (158 hwp + 6 hwpx) 1614 페이지 모두 페이지 수 동일. 본 정정이 페이지 break 위치 변경하지 않음.

## 2. cargo test 결과

### 2-1. Unit tests

```
$ cargo test --lib --release
test result: ok. 1135 passed; 0 failed; 2 ignored; 0 measured
```

베이스라인 1134 → **1135 passed** (`test_630_middle_dot_full_width_in_registered_font` 추가 GREEN). 회귀 0.

### 2-2. Integration tests (16 파일)

| 테스트 파일 | passed |
|------------|--------|
| exam_eng_multicolumn | 1 |
| hwpx_roundtrip_integration | 14 |
| hwpx_to_hwp_adapter | 25 |
| issue_301/418/501/505/514/516/530/546/554 | 1+1+1+9+3+8+1+1+12 = 37 |
| **issue_630 (신규)** | **1** ✓ |
| page_number_propagation | 2 |
| svg_snapshot (issue_147 골든 갱신 후) | 6 |
| tab_cross_run | 1 |
| **합계** | **87 passed / 0 failed** |

issue_546 (Task #546 회귀 가드) 1 passed — Task #546 회귀 0.
issue_554 (Task #554) 12 passed — 회귀 0.
svg_snapshot 6 passed — issue_267 (KTX TOC) 영향 없음 + issue_147 (aift p3) 골든 정정 정합.

## 3. 골든 SVG 갱신

`tests/golden_svg/issue-147/aift-page3.svg`:
- Before: 210127 bytes
- After: **210175 bytes** (+48 bytes — `·` 측정 정정 효과)
- byte 차이 분포: 188447 / ~210000 byte 변경 (≈90%) — `·` 가 다수 출현하는 페이지의 paren_x / circle cx 좌표 분포 이동 정합

`tests/golden_svg/issue-267/ktx-toc-page.svg`:
- Before/After byte-identical — `·` 미출현 fixture 영향 없음 ✓

## 4. aift p4 권위 케이스 — paren_x 정량 비교

| 라인 | Before paren_x | After paren_x | 변화 |
|------|---------------|----------------|-----|
| 1-1 (`·` 포함) | 592.36 | **601.03** | +8.67 ✓ |
| 1-2 | 601.03 | 601.03 | 0 |
| 3-1 (`·` 포함) | 592.33 | **601.00** | +8.67 ✓ |
| 3-4 (`·` 포함) | 592.41 | **601.08** | +8.67 ✓ |
| 4-1 (`·` 포함) | 592.36 | **601.03** | +8.67 ✓ |
| 6-1 | 600.25 | 600.25 | 0 |
| 6-2 | 601.39 | 601.39 | 0 |
| 7-2 (`·` 포함) | 592.31 | **601.00** | +8.67 ✓ |
| 8-1 (`·` 포함) | 592.31 | **601.00** | +8.67 ✓ |

**`·` 포함 6 라인 모두 정확히 +8.67px 우측 이동 → 정합 회복**. `·` 미포함 라인은 영향 없음 (정정의 케이스별 명시 정합 입증).

전체 spread: **9.08px → 1.13px** (8.67px 이탈 0).

## 5. 시각 판정 자료

`output/svg/task630_before/aift_004.svg` ↔ `output/svg/task630_after/aift/aift_004.svg`:
- 권위 케이스 (목차 페이지) before/after 비교 가능
- 두 파일 byte 차이 188,447 / 210,127 — `·` 측정 변경이 다수 페이지에 영향

대표 fixture After SVG 정비 (Stage 5):
- `output/svg/task630_after/aift/` (77 페이지)
- `output/svg/task630_after/KTX/` (27 페이지)
- `output/svg/task630_after/exam_kor/` (20 페이지)
- `output/svg/task630_after/exam_math/` (20 페이지)
- `output/svg/task630_after/k-water-rfp/` (27 페이지)
- `output/svg/task630_after/kps-ai/` (80 페이지)
- 합계 251 페이지 시각 판정 자료

## 6. clippy / WASM

### 6-1. clippy

본 task 변경 영역 (`text_measurement.rs`, `tests/issue_630.rs`) **clippy 통과**.

기존 코드 `src/document_core/commands/table_ops.rs:1007` + `src/document_core/commands/object_ops.rs:298` 에서 `panicking_unwrap` 에러 2건 — **devel 베이스의 기존 문제 (본 task 무관)**, 별도 task 영역.

### 6-2. WASM

Docker daemon 미실행으로 본 환경에서 WASM 빌드 측정 불가. 본 task 변경은 다음과 같이 미세:
- `text_measurement.rs`: 1 라인 제거 + 코멘트 갱신 + 단위 테스트 1개
- `tests/issue_630.rs`: 신규 105 라인 (테스트 only)

기존 WASM 사이즈 (`pkg/rhwp_bg.wasm`): 4,588,989 bytes. 본 정정으로 사이즈 유의미한 변화 없음 예상 (LOC 단순 변동).

## 7. 산출물

- `tests/golden_svg/issue-147/aift-page3.svg` (갱신, +48 bytes)
- `output/svg/task630_after/{aift,KTX,exam_kor,exam_math,k-water-rfp,kps-ai}/` (시각 판정 자료)
- `output/svg/task630_before/aift_004.svg` (Stage 1 baseline 보존)
- `output/svg/task630_before/KTX_002.svg`
- `output/svg/task630_stage3/aift_004.svg` (정정 1 단독 결과)
- `output/svg/task630_stage4/aift_004.svg` (정정 2 회귀 검증)

## 8. 핵심 메트릭 최종

| 메트릭 | Stage 1 (Before) | Stage 5 (After) |
|--------|------------------|-----------------|
| aift p4 정렬 그룹 수 | 4 | **1** |
| 8.67px 이탈 라인 | 6 | **0** |
| spread | 9.08px (본 라인) | **1.13px** |
| cargo test --lib --release | 1134 passed | **1135 passed / 0 failed** |
| 통합 테스트 (16 파일) | 86 passed | **87 passed / 0 failed** (+1) |
| svg_snapshot | 6 passed | **6 passed** (issue_147 갱신, issue_267 영향 0) |
| 164 fixture 페이지 수 회귀 | 0 (baseline) | **0** |
| Task #546 / #554 회귀 가드 | passing | **passing** (회귀 0) |
| 본 task 변경 LOC | - | **+105 (테스트) / +6 -2 (소스)** |

## 9. 시각 판정 요청

작업지시자 시각 판정 요청 — `output/svg/task630_before/aift_004.svg` vs `output/svg/task630_after/aift/aift_004.svg` 비교:
- `·` 포함 라인의 `(페이지 표기)` 위치가 한컴 PDF 정답지 (`samples/aift.pdf` 페이지 2 우측) 와 정합 회복되었는지 확인
- `·` 미포함 라인의 정렬에 영향 없는지 확인
- 다른 페이지 (aift 전체 + KTX + exam_kor 등) 의 부수 변경이 PDF 정합 향상 또는 회귀 0 인지 확인

## 10. 다음 단계

작업지시자 시각 판정 ★ 통과 후:
- 최종 결과 보고서 작성 (`mydocs/report/task_m100_630_report.md`)
- `mydocs/orders/20260506.md` 갱신
- local/task630 → local/devel merge → devel push
- Issue #630 close

## 11. 승인 요청

Stage 5 광범위 회귀 검증 완료. 페이지 수 회귀 0 / cargo test 1135+87 passed / 0 failed / svg_snapshot 6/6 / Task #546/#554 회귀 가드 통과.

작업지시자 시각 판정 요청드립니다.
