# Stage 3 보고서 (증분 5) — Task #1257/#1184: 다줄 prev 갭 보존 타깃 수정

- 이슈: edwardkim/rhwp#1257 · 브랜치: `local/task1257`

## 발견 (집중 디버깅, EN_GAP_TRACE)

render `build_single_column` 에 이미 between-notes 갭 보존 로직 존재
(`should_preserve_endnote_title_gap` + `prev_endnote_title_gap_px`, layout.rs ~2953/~3123).
다줄/forward 케이스가 실패하는 정확한 원인:

| 헤더 | prev last (ls, lh) | 실패 원인 |
|------|--------------------|----------|
| 문26 (pi945) | (1984, **2070**) | `seg.line_height <= 1500` 필터 → gap=0 |
| 문29 (pi995) | (1984, **6897**) | 동일 필터 → gap=0 |
| 문27 (pi956) | (1984, 900) | 필터 통과하나 `y_offset==y_before`(forward-suppress) → 조건 실패 |

## 수정 (이번 증분)

`prev_endnote_title_gap_px` 의 FullParagraph 필터에서 **`&& seg.line_height <= 1500` 제거**.
`line_spacing > 1000` 만으로 주입된 between-notes 갭(1984) 을 식별 → 직전 미주가 tall 줄(수식)
으로 끝나도 갭 보존(문26 lh=2070·문29 lh=6897).

## 검증

- 문26/29: 갭 보존 발동(`preserve=true`, gap_px 0→26.5).
- **페이지수 패리티 유지**: 2022 23, 2023 20, 10월 18, 미주사이20 24, 구분선아래20 23, 3-11 21.
- **`cargo test` 전체 1965 passed, 0 failed**(default 동작 변경인데 회귀 0).
- 오버플로우(가벼운 갭-복원 tradeoff): 10월 max 33.6→54.0px(한 경계 +20px), 2023/미주사이20
  max 불변(+1/+4 건은 기존 max 이하). 모두 기존 다수 오버플로우(미주사이20 31건) 위.

## 한계 (남은 케이스)

- **문5 등 컬럼-하단**: column-bottom cap(별도 경로) — 본 수정 무관, 미해결.
- **문27 등 forward-suppress**: `y_offset==y_before` 조건 미충족 — 본 수정은 필터만 완화(조건은
  유지). 조건 완화는 오버플로우 확대 위험으로 별도 검토.
- 오버플로우 tradeoff 완전 제거 = 순차-flow 재설계(build_single_column 미주 배치 모델 통째 재작성)
  필요(설계서 §2). 본 증분은 그 전 **타깃 개선**.

## 증분 6 — 문27(forward-suppress) 조건 완화 시도 (revert)

`should_preserve_endnote_title_gap` 의 `y_offset > y_before+0.5` 조건 제거 시도(문27 등
forward-suppress 는 vpos_adjust 가 y 를 안 움직여 조건 실패). 결과: **4개 PDF-정합 테스트 실패**
(`issue_1189_oct_page11_..._gaps_match_pdf`, `issue_1189_nov_...`, `issue_1209_nov_page14_q22_
keeps_hancom_endnote_gap`, `issue_1256_..._q12_keeps_between_notes_gap`).

→ **발견: `y_offset>y_before+0.5` 조건은 load-bearing.** forward-suppress 케이스는 한컴이 갭을
풀로 주지 않는(억제 위치가 PDF 정답) 경우를 올바로 제외한다. 즉 문27 의 억제 위치가 PDF 정답일
수 있고, 갭 강제는 PDF 와 괴리. **targeted 완화로는 문27/문5 해결 불가** — PDF 정합 테스트와 충돌.
조건 revert(필터 수정 d3393f4c 유지).

## 결론

집중 디버깅으로 다줄 prev 케이스(문26/29 + 2023/10월 동류)를 **회귀 0·패리티 유지**로 정합 복원
(필터 수정, 명확·확정). 그러나 **문27(forward-suppress)·문5(컬럼-하단)은 targeted 수정 한계**:
완화가 PDF-정합 테스트와 충돌 → 갭을 전역 균일·PDF 동시검증하는 **순차-flow 재작성으로만 해소**.
이 케이스들은 현 절대-vpos 모델에서 부분 수정으로는 PDF 와 양립 불가(설계서 §2 영역).
