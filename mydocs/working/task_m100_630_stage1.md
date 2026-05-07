---
issue: 630
milestone: m100
branch: local/task630
stage: 1 — 회귀 베이스라인 측정
created: 2026-05-06
status: 완료 — 승인 대기
---

# Task #630 Stage 1 완료 보고서 — 회귀 베이스라인 측정

## 1. 베이스라인 정량

### 1-1. aift.hwp 4페이지 `(페이지 표기)` 위치 분포 (Before)

`output/svg/task630_before/aift_004.svg` 에서 `(페이지 표기)` 22 라인 위치 추출:

| 그룹 | `(` x 좌표 | 라인 수 | 특징 |
|------|----------|---------|------|
| 좌측 이탈 (`·` 포함) | **592.31~592.41** | **6 lines** | 1-1 / 3-1 / 3-4 / 4-1 / 7-2 / 8-1 |
| 정상 정렬 | 600.95~601.08 | 16 lines | `·` 미포함 |
| 6-1 (혼재) | 600.25 | 1 line | `·` 미포함 (수치 미세 변동) |
| 6-2 (wrap 별도 라인) | 292.59 | 1 line | 두 줄 wrap 의 끝 라인 |

**결론**: `·` 포함 라인 6 개가 정확히 8.67px 좌측 이탈. `·` 반각(8.67px) vs 전각(17.34px) 차이 = 8.67px 와 정확히 일치.

### 1-2. KTX.hwp 1페이지 (TOC) 위치 분포

`output/svg/task630_before/KTX_002.svg`:
- 17 라인 모두 right-edge ≈ 690.1~690.8 px 에 정합 정렬
- `·` 출현 0 건 (`<circle>` count = 0)
- **본 정정의 KTX 목차 영향 가능성 낮음** (KTX TOC 는 `·` 가 leader/contentent 모두 미사용)

### 1-3. 광범위 sweep 베이스라인

| 항목 | 값 |
|------|-----|
| 샘플 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 | **1614** |
| 단위 테스트 | **1134 passed** / 0 failed / 2 ignored |
| svg_snapshot | **6 passed** (form-002, table-text, issue_157, **issue_267 (KTX TOC)**, **issue_147 (aift p3=p4)**, render_is_deterministic) |

상위 5 fixture (페이지 수): hwpspec.hwp 175 / hwp-3.0-HWPML.hwp 122 / hwpctl_API_v2.4.hwp 105 / kps-ai.hwp 80 / **aift.hwp 77**.

### 1-4. `·` 출현 분포 (aift.hwp 전체)

`output/aift_*.svg` 의 `<circle>` 출현 카운트 (Task #257 의 `·` → `<circle>` 매핑) — 본 정정의 광범위 영향 영역:

- aift 전 77 페이지 중 50+ 페이지에 `·` 출현 (페이지당 1–16 회)
- 측정폭 변경 (반각 → 전각) 이 모든 출현 위치에 영향
- 시각 영향: `·` 좌우 advance 의 중심에 `<circle>` 그려지므로 글자 간격이 한컴 PDF 정합으로 자연 이동

## 2. 회귀 위험 영역 사전 식별

### 2-1. 즉시 영향 (확실)

1. **`tests/golden_svg/issue-147/aift-page3.svg`** (`samples/aift.hwp` page 3 = 페이지 4)
   - 본 정정의 권위 fixture — 골든 갱신 필요 (시각 판정 후 `UPDATE_GOLDEN=1`)

2. **`tests/golden_svg/issue-267/ktx-toc-page.svg`** (`samples/KTX.hwp` page 1)
   - `·` 미출현이지만 native tab_type 정정 (수정 2) 으로 인라인 탭 RIGHT/CENTER 처리 변경 가능
   - SVG byte diff 분석 → 변경 없으면 안전 / 변경 있으면 시각 판정

### 2-2. 잠재 영향 (sweep 으로 검증)

3. **`·` 가 leader 로 사용되는 fixture** — `·` 의 advance 변경이 dot leader 길이에 직접 영향. KTX 목차는 leader 미사용 확인됨. 다른 fixture sweep 필요.

4. **Justify slack 분배 + `·` 인접** — `·` 측정폭 증가가 자연 폭 → slack 재분배 영향. align=Justify fixture sweep.

5. **인라인 탭 RIGHT/CENTER 사용 케이스** — 코드 코멘트가 명시한 "기존 golden SVG 가 LEFT fallback 의존" 영역. issue_267 외에도 다른 fixture 가 이 동작에 의존할 수 있음.

### 2-3. 영향 없음 (확인됨)

- WASM `WasmTextMeasurer::compute_char_positions` 경로 — 이미 `inline_tab_type(ext)` + `2/3` 정합. 변경 없음.
- TabDef 기반 `has_custom_tabs` 분기 — 별도 코드 경로. 변경 없음.
- 다른 narrow punctuation (`'` `"` 등 `'\u{2018}'..='\u{2027}'`) — `is_halfwidth_punct` 매칭 보존.

## 3. 산출물

- `output/svg/task630_before/aift_004.svg` (베이스라인)
- `output/svg/task630_before/KTX_002.svg` (베이스라인)
- `/tmp/task630_baseline_pages.txt` (164 fixture 페이지 수)
- 단위 테스트 1134 passed 베이스라인

## 4. 단계별 정량 측정 키 메트릭 (Stage 5 까지 추적)

| 메트릭 | Stage 1 (Before) | 목표 (After) |
|--------|------------------|-------------|
| aift p4 `(페이지 표기)` 정렬 그룹 수 | 4 (592 / 600 / 601 / 293) | 1 (≈601 ±0.5) (wrap 별도 라인 제외) |
| 8.67px 이탈 라인 수 | 6 | 0 |
| 단위 테스트 | 1134 passed | 1134+N passed (Stage 2 RED → Stage 4 GREEN) |
| svg_snapshot | 6 passed | 6 passed (issue-147/267 골든 갱신 후) |
| 164 fixture 페이지 수 회귀 | 0 (baseline) | 0 |
| Docker WASM 사이즈 | (Stage 5 측정) | (Stage 5 비교) |
| clippy | 0 | 0 |

## 5. 다음 단계 (Stage 2)

단위 테스트 3 건 작성 (RED 확인):
1. `test_630_middle_dot_full_width` — 텍스트 측정 단위
2. `test_630_native_inline_tab_right_align` — 텍스트 측정 단위
3. `test_630_aift_p4_toc_alignment` — 통합 (aift p4 SVG 위치 검증)

## 6. 승인 요청

Stage 1 베이스라인 측정 완료. Stage 2 (단위 테스트 작성, RED 확인) 진행 승인 부탁드립니다.
