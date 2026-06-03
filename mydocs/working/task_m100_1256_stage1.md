# Stage 1 보고서 — Task #1256: 진단 고정 + 회귀 베이스라인

- 이슈: edwardkim/rhwp#1256 · 브랜치: `local/task1256` (from `devel`)
- 구현계획서: `task_m100_1256_impl.md`

## 1. 영향 경계 전수 목록 (before, RHWP_VPOS_DEBUG)

문서 전체 1회 패스에서 `prev_ls=1984 && compact_safe_backtrack=true` (= 주입된 7mm 을
backtrack 이 덮어쓰는 버그) 경계 **총 15건**. 모두 `delta = y_in − end_y ≈ 20.42px` 로
**정확히 7mm 분량을 위로 끌어올림** (단일·체계적 원인 입증).

| # | path | pi (제목) | prev_pi | y_in (정답) | end_y (현재,버그) | delta(px) |
|---|------|-----------|---------|-------------|-------------------|-----------|
| 1 | page | 499 | 498 | 233.53 | 213.11 | 20.42 |
| 2 | lazy | 511 | 510 | 500.13 | 479.71 | 20.42 |
| 3 | page | 567 | 566 | 418.69 | 398.27 | 20.42 |
| 4 | page | 696 | 695 | 804.04 | 783.61 | 20.43 |
| 5 | page | 712 | 711 | 193.88 | 173.45 | 20.43 |
| 6 | page | 718 | 717 | 363.56 | 343.13 | 20.43 |
| 7 | lazy | 734 | 733 | 804.00 | 783.57 | 20.43 |
| 8 | lazy | 757 | 756 | 595.03 | 574.60 | 20.43 |
| 9 | page | 854 | 853 | 214.44 | 194.01 | 20.43 |
| 10 | page | 867 | 866 | 569.27 | 548.84 | 20.43 |
| 11 | lazy | 935 | 934 | 428.71 | 408.28 | 20.43 |
| 12 | page | 956 | 955 | 321.40 | 300.97 | 20.43 |
| 13 | page | 1066 | 1065 | 398.83 | 378.40 | 20.43 |
| 14 | page | 1076 | 1075 | 611.81 | 591.39 | 20.42 |
| 15 | lazy | 1106 | 1105 | 608.65 | 588.23 | 20.42 |

→ **수정 후 기대**: 이 15건이 모두 `result = y_in` (delta→0) 으로 렌더, 7mm 복원.

## 2. 무변경 보장 케이스 (회귀 대상)

- `compact_safe_backtrack=true` 인데 `prev_ls < 1984` 인 경계: **0건** (본 문서).
  → 판별자 `injected_between_notes = endnote_between_notes_hu>0 && seg.line_spacing>=1984`
    는 15건만 정확히 타격, 문13(의도적 소간격, prev_ls<1984)·자연 trailing 은 무영향.
- 다른 backtrack 분기 재포획 검토: safe_backtrack 해제 시 deep_backtrack(near-tail
  `y_offset>0.90·h`)·title_tail_backtrack(`follows_endnote_title`)·new_note_jump(forward
  `end_y>y_offset`) 모두 조건 불성립(15건은 mid-column·prev 빈문단·backward) → `result=y_offset`
  로 자연 낙하. **단 후속 항목 desync 방지 base-shift 동반 필요(Stage 2).**

## 3. 회귀 베이스라인 수치 (고정)

| 파일 | 페이지 수 |
|------|----------|
| `3-09월_교육_통합_2022.hwp` | **23** |
| `3-09월_교육_통합_2022.hwpx` | **23** |
| `3-09월_교육_통합_2024-미주사이20.hwp` | **24** (핵심 분기 타깃) |
| `3-09월_교육_통합_2024-구분선아래20.hwp` | **23** |

- before SVG 23p 보관: `output/poc/before/*.svg` (수정 후 시각 diff 용).
- 한컴 정합 기준(96dpi red-header): p9 문6→문7 = 우리 264px vs 한컴 PDF 287px (−23px).
- before VPOS 전체 로그: `output/poc/vpos_all_before.txt` (1118건).

## 4. 테스트 베이스라인

`cargo test` — 전체 green (5개 스위트 ok, 0 failed; 1 ignored).

## 5. 결론

근본원인·타깃 15건·무변경 경계·회귀 기준선 모두 고정 완료. Stage 2(판별자 + backtrack
제외 + base-shift) 착수 준비 완료.

---
승인 요청: Stage 1 결과 확인 후 Stage 2 구현에 착수해도 될까요?
