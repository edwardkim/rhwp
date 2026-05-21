# Task #1052 Stage 3 보고서 — 회귀 가드 + sweep + WASM + 시각 판정 게이트

- 이슈: [#1052](https://github.com/edwardkim/rhwp/issues/1052)
- 단계: Stage 3
- 일시: 2026-05-21

## 1. 결과 요약

회귀 가드 4 추가 (4/4 통과) + 9 fixture 광범위 sweep (BEFORE/AFTER diff=2 = 본 결함 정정만, 회귀 부재) + WASM Docker 빌드 + rhwp-studio 동기화 완료.

## 2. 회귀 가드 추가

`tests/issue_1052_footnote_in_textbox.rs` (88 lines, 4 tests):

| 테스트 | 검증 항목 |
|--------|----------|
| `issue_1052_textbox_footnote_appears_in_footer_area_hwpx` | HWPX 글상자 안 각주 본문 "글상자 내부 각주" 페이지 하단 표시 |
| `issue_1052_textbox_footnote_appears_in_footer_area_hwp` | HWP variant 동일 정합 |
| `issue_1052_body_footnote_no_regression_hwpx` | 본문 직속 각주 "일반 문단내 각주" 회귀 부재 |
| `issue_1052_textbox_footnote_marker_present` | 각주 마크 "1)" + "2)" 본문 위치 유지 |

검증 방식: `svg_text_sequence()` 헬퍼로 SVG 의 모든 `<text>...</text>` 내용을 순서대로 이어붙인 문자열에서 sub-string 등장 단언 (SVG 가 글자 단위 분리되어 있으므로).

결과: **4/4 passed** (`cargo test --release --test issue_1052_footnote_in_textbox`).

## 3. 광범위 sweep (9 fixtures, 143 SVG each)

| Fixture | 페이지 수 | BEFORE/AFTER diff |
|---------|----------|-------------------|
| **samples/hwpx/footnote-tbox-01.hwpx** | 1 | **1** (의도된 본질 정정) |
| **samples/footnote-tbox-01.hwp** | 1 | **1** (의도된 본질 정정) |
| samples/footnote-01.hwp | 6 | 0 |
| samples/2010-01-06.hwp | 6 | 0 |
| samples/table-in-tbox.hwp | 2 | 0 |
| samples/aift.hwp | 74 | 0 |
| samples/KTX.hwp | 27 | 0 |
| samples/biz_plan.hwp | 6 | 0 |
| samples/exam_kor.hwp | 20 | 0 |

```
diff -rq output/poc/issue_1052/before/ output/poc/issue_1052/after/ = 2
```

→ **footnote-tbox-01 만 변동** (본질 정정), **나머지 7 fixture 회귀 부재** 정량 입증.

산출물: `output/poc/issue_1052/{before,after}/` (총 286 SVGs).

## 4. CI 패턴 검증 (Stage 2 완료 후 재확인)

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | **1319 passed** |
| cargo test --release --tests | FAILED 0 (전체 통합) |
| cargo test --release --test issue_1052_footnote_in_textbox | **4/4 passed** |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |

## 5. WASM 빌드 + 동기화

| 항목 | 결과 |
|------|------|
| Docker WASM 빌드 | OK (`pkg/rhwp_bg.wasm` 4.90 MB) |
| rhwp-studio 동기화 | OK (`public/rhwp_bg.wasm` + `rhwp.js` 247 KB) |

본 PR 코드 변경은 typeset.rs 의 글상자 안 각주 수집만이라 WASM 크기 변동 미미.

## 6. 작업지시자 시각 판정 게이트 (Stage 4 전제)

판정 항목:
1. **HWPX** `samples/hwpx/footnote-tbox-01.hwpx` 로드 → SVG 페이지 하단 각주 영역에 "1) 글상자 내부 각주" + "2) 일반 문단내 각주" 표시 정합
2. **HWP** `samples/footnote-tbox-01.hwp` 로드 → 동일 정합
3. 한컴 PDF (`pdf-large/hwpx/footnote-tbox-01.pdf`) 정합 비교
4. 일반 fixture (aift / KTX / biz_plan / table-in-tbox) 회귀 부재 시각 확인

판정 통과 후 Stage 4 (no-ff merge + close + archives + orders) 진행.

## 7. 잔여 / 후속

- 머리말/꼬리말 안 각주 (본 sample 범위 외)
- 미주 (Endnote) 의 Shape 내부 처리 (별도 검토)
- table-in-tbox 안 표 셀 안 각주 (별도 가드 필요 시 후속)
