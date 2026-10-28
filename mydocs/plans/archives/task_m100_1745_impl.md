# Task #1745 구현계획서 — 텍스트-anchor 어울림 표 wrap 흡수 (4단계)

수행계획서: `mydocs/plans/task_m100_1745.md`

## Stage 1 — 재현 샘플 동결 + 실패 재현
- `samples/task1745/table_text_anchor_wrap.hwp` 로 재현 파일 복사 + README
  (출처·구조·기대 동작 기록, PR 재현용 포함 규칙).
- `rhwp dump-pages` 로 실패 확인(pi1 이 3쪽 FullParagraph 로 배치됨) 기록.
- 산출: 샘플 + Stage 1 보고서.

## Stage 2 — wrap strip cs/sw 표 geometry 도출 (`typeset.rs`)
- wrap zone 활성화 지점(2827-2837): anchor 문단의 첫 LINE_SEG 가 wrap 띠가 아닌 경우
  (텍스트 혼합 anchor — cs=0 이고 sw 가 표 우측 잔여폭과 불일치) Square wrap 표 컨트롤의
  geometry 로 expected_cs = horz_offset + width + outer_margin(좌+우), expected_sw =
  body_width − expected_cs 를 계산해 `wrap_around_cs/sw` 에 저장.
- 가드: 비-TAC + wrap=Square 표, expected_sw > 0, 기존 케이스(표 단독 문단, 첫 seg 가
  이미 wrap 띠)는 무변경 경로 유지.
- 단위테스트: 활성화 값 도출 함수화 + 테스트 2개(표 단독 anchor 무변경 / 텍스트 혼합 anchor 도출).

## Stage 3 — 다쪽 분할 표의 WrapAroundPara 첫 fragment column 소급 기록
- 흡수 지점(2628-2635): `st.wrap_around_table_para` 의 첫 fragment(PartialTable cont=false
  또는 Table)가 이미 flush 된 `st.pages` 의 column 에 있으면 그 column 의
  `wrap_around_paras` 에 push, 아니면 현행(현재 column) 유지.
- layout 은 column 별 `wrap_around_paras` 를 해당 column 의 anchor 표 fragment 옆에 렌더하므로
  추가 변경 없음(확인 포함).
- 재현 파일 pi1 → 1쪽 확인 (`dump-pages` + `verify_pi_page_vs_hangul.py --files` MATCH).

## Stage 4 — 회귀 검증 + 보고
- `cargo test --lib` 전체, wrap 계열(#362/#604/#724) 관련 테스트.
- byeolpyo1(4쪽)·byeolpyo4(26쪽) 무회귀 (#1658 게이트).
- 이번 조사 mismatch 39건 `verify_pi_page_vs_hangul.py --files` 재검증 — 개선/무회귀 집계.
- 최종보고서 `mydocs/report/task_m100_1745_report.md`.
