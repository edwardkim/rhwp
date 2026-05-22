# Stage 1 보고서 — Task #1068: treat_as_char 표 새 페이지 미이월 (조사)

- 브랜치: `local/task1068` (stream/devel 기준, 소스 무변경)
- 재현 문서: 비공개 RFP (`samples/2. 인공지능...제안요청서.hwpx`, 분석만·커밋 금지)

## 확정 사실

- 문제 항목: para 567 = 제목줄(th=2200≈29px) + **treat_as_char 표**(48081×63234HU, 641×**843px**).
- 앞 문단 pi566 "[붙임 4]"(17px), 뒤 pi568 "[붙임 5]" — para 567 은 독립 배치 의도.
- `LAYOUT_OVERFLOW_DRAW: pi=567 line=1 y=1886 overflow≈839px` — 표 줄이 본문 하단 아래로 통째 렌더.
- **dump-pages 페이지 92: items=3, used=1797.9px** (본문 941px의 ~2배!):
  - `FullParagraph pi=566 h=17.3`
  - `Table pi=567 ... 843px`
  - `PartialParagraph pi=567 lines=0..2`

## 핵심 가설 — treat_as_char 표 높이 이중 계상

`used=1797 ≈ pi566(17) + 표(843) + 문단(제목29+표줄843=872)` → **표 높이가 두 번 계상**:
1. 별도 `Table` PageItem (843px)
2. PartialParagraph 의 표 줄 line_height (843px, line_segs ls[1] lh=63234)

→ 페이지 used 가 과충전되고, 분할 줄 배치에서 표 줄이 페이지 하단에 얹혀 본문 밖으로 렌더.
(분할 fit 산식은 제목29+표843=872 < 잔여~924 로 "들어감" 판정하나, 실제 누적/렌더는 이중
계상·vpos 매핑으로 어긋남.)

- `is_atomic_tac_singleton`(typeset.rs:1851) 은 Picture/Shape 만 → TAC 표 이월 미처리 (관련).
- 비-TAC 표 wrap-around 처리(1096-1122)는 TAC 표 제외.

## 코드 경로 확정 (정정)

para 567(표 포함)은 일반 줄 분할(1909)이 **아니라 표 배치 경로**(`typeset.rs` ~2520-2620)로
처리된다. 이 경로:
- `PageItem::Table` push (2584) + pre-text `PartialParagraph` push (2573).
- **`tac_wrap_split`(2597)**: `treat_as_char && pre_table_end_line>0 && < total_lines` — "전폭 TAC
  표가 자기 줄(line index=pre_table_end_line)에 놓인 split 케이스" 를 **이미 인지**(주석 2593-2596,
  이중 계산 방지 의도, Task #853).
- 그러나 para 567 에서 used=1797px 로 과충전 → 이 경로의 **페이지 fit/break(이월) 처리가 near-
  full-page TAC 표에 대해 불완전**.

→ 수정 영역은 **표 배치 경로의 TAC 표 fit/이월 로직**(2520-2620, tac_wrap_split 인근).
`is_atomic_tac_singleton`(1851, Picture/Shape) 와는 별개 경로.

## 공개 픽스처

- `gen-table` CLI 는 일반 표만 생성(treat_as_char·페이지 크기·전후 쪽나누기 미지원).
- near-full-page treat_as_char 표 + 전후 쪽나누기 픽스처는 **별도 생성기/수작업 구축 필요** →
  Stage 2 초입에서 구축(공개 커밋 가능).

## 잠정 결론

- 수정 영역 localize 완료: **표 배치 경로(2520-2620)의 TAC 표 페이지 fit/이월**.
- used=1797px(본문 2배)는 표 줄 + Table item 의 누적 정책(tac_wrap_split)이 이 케이스를 완전히
  커버하지 못함을 시사. Stage 2에서 누적·fit/break 산식을 정밀 분해 + 공개 픽스처 재현 후 설계.
