# PR #1674 처리 보고서 — HWPX 가운데 정렬 셀 TAC shape 뒤 본문 vpos 정합 (#1673)

- PR: https://github.com/edwardkim/rhwp/pull/1674
- 제목: `Fix HWPX centered cell paragraph vpos after TAC shape`
- 작성자: oleg-sung (Oleg Sungurovsky, location: Russian — 러시아 기여자, **재기여**: #1489 머지 후 2번째 PR)
- 연결: closes #1673
- base ← head: `devel` ← `oleg-sung:fix/hwpx-centered-cell-vpos-after-tac`
- 처리일: 2026-06-30

## 1. 처리 결정 — 시각 판정 통과 + 페이지 회귀 게이트 통과 → admin merge

**admin merge (시각 판정 통과).** 렌더링/레이아웃 수정이나 작업지시자 시각 판정 통과
(2026-06-30) + 페이지/렌더 회귀 게이트 전수 통과로 확인. CLEAN + 충돌 0 + CI 전부 pass.

### 페이지 회귀 게이트 전수 결과 (FAILED 0)

| 게이트 | 결과 |
|---|---|
| `svg_snapshot` (golden SVG 시각 출력) | 8 passed |
| `hwpx_roundtrip_baseline` | 4 passed |
| `hwp5_roundtrip_baseline` | 3 passed |
| `visual_roundtrip_baseline` | 3 passed |
| `opengov_corpus_snapshot` (페이지 스냅샷) | 2 passed |

좁은 가드(`has_initial_tac_shape_host`) 덕분에 TAC 제목 호스트 셀에만 영향, golden SVG 8건
무변동 → 다른 문서 페이지 배치 무회귀 확인.

## 2. base 착시 주의

내 로컬 devel 이 한 단계 뒤(`4481d0bf`)였을 때 `git diff devel pr1674` 가 60 files/+4199 로
보였으나, origin/devel 이 `1d8c2577`(#1670/#1671/#1662 머지분)로 진전돼 있었고 **PR base 가 곧
`1d8c2577`** 이다. origin/devel ff 후 `git diff origin/devel pr1674` = **정확히 3 files +142/-4**.
즉 PR 은 최신 base 위 깨끗한 변경이며, 섞여 보이던 #1658 등은 이미 devel 머지분이다.

## 3. 변경 범위 (3 files +142/-4)

| 파일 | 내용 |
|---|---|
| `src/renderer/layout/table_layout.rs` | +40 — `cell_para_line_anchor_y()` 헬퍼 + `has_initial_tac_shape_host()` 가드 추가. 가운데 정렬 셀이 **첫 문단=빈텍스트+TAC shape 호스트**면 후속 문단도 saved `LINE_SEG.vertical_pos` 앵커 사용 |
| `tests/issue_centered_cell_vpos_after_tac_shape.rs` | +106 — fixture SVG 에서 intro 문단 y 가 title shape 와 겹치지 않음을 검증(2 tests) |
| `samples/hwpx/hwpx-centered-cell-vpos-after-tac-shape.hwpx` | 축소 1페이지 fixture(381KB) |

## 4. 근본 원인 / 수정 평가

원인: 가운데 정렬 셀의 다문단을 누적 레이아웃으로 배치 + 후속 문단의 saved vpos 무시 →
TAC 제목 shape 뒤 본문이 너무 높게 올라와 제목과 겹침.

수정: 정렬 게이트를 `use_top_vpos_anchor || has_initial_tac_shape_host` 로 확장. **좁은 가드** —
첫 문단이 빈 텍스트 + TAC(treat_as_char) shape 호스트일 때만 발동. 기존 top-aligned 셀 경로와
단일 문단 가운데 정렬 셀 동작은 불변(`cell_para_line_anchor_y` 가 `use_top_vpos_anchor` 분기로
기존 앵커 계산 유지). 회귀 안전.

## 5. 검증 (로컬, Linux WSL2)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas) | 전부 pass |
| 충돌 | 0건 (CLEAN, base=origin/devel) |
| 신규 `issue_centered_cell_vpos_after_tac_shape` | 2 passed |
| 전체 `cargo test --tests` | **FAILED 0건** |
| fmt --check / clippy(table_layout.rs) | clean / 무경고 |
| **시각 대조 (PNG)** | 수정 전: 본문이 파란 TAC 제목 박스 위로 겹침 → 수정 후: 제목 박스 아래 적정 간격 분리 |
| 스크린샷 정책 | PNG 미커밋(PR 본문 첨부만) — 룰 준수 |

## 6. 시각 판정 자료

- 픽스처: `samples/hwpx/hwpx-centered-cell-vpos-after-tac-shape.hwpx` (문화체육관광부 공고)
- before/after SVG·PNG: `output/poc/task1673/{before,after}_p1.png`
- 판정 포인트: page1 의 파란 제목 박스("…경력경쟁채용시험 공고")와 본문("문화체육관광부에서는…")이
  겹치지 않고 박스 아래에 배치되는지. 권위는 작업지시자 한컴 환경.

## 7. 기여자 메모

- **러시아 기여자**(location: Russian)의 **재기여** — #1489(rowbreak) 머지 후 2번째 PR.
  (정정: 초기 영어 머지 코멘트에서 "first contributor from Russia"로 잘못 표기 → #1489가 선행이므로 정정. 동일 author 이메일 `andreisungurovsky@yandex.ru` 확인.)
  코멘트는 영어로. fixture·테스트 동반 + 스크린샷 정책 준수로 기여 품질 양호.

## 8. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1674_review.md`
