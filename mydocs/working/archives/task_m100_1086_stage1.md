# Task #1086 Stage 1 완료 보고서 — 원점 재검토

- 이슈: [edwardkim/rhwp#1086](https://github.com/edwardkim/rhwp/issues/1086)
- 브랜치: `local/task1086`
- 기준: `upstream/devel` `dae5c94f`
- 승인: 작업지시자 “승인” 후 Stage 1 진단 진행

## 1. 동기화와 기준 확인

`upstream/devel` 동기화 후 `local/task1086` 을 최신 기준으로 초기화했다.

```text
HEAD = dae5c94f fix: materialize header footer tables for hwpx export
```

최신 기준에서도 재현된다.

```bash
cargo build --release
target/release/rhwp dump-pages samples/k-water-rfp.hwp | rg -c '^=== 페이지'
```

```text
29
```

한컴 정답지:

```text
pdf/k-water-rfp-2022.pdf
Creator: Hwp 2022 12.0.0.4426
Pages: 27
```

## 2. PR #1085 기준 확인

Issue #1086 본문은 “devel HEAD + PR #1085” 기준이라고 되어 있다. 현재 `upstream/devel` 에는 PR #1085 (`local/task1042`) 이 아직 들어와 있지 않다.

임시 worktree에서 `upstream/devel + local/task1042` 를 merge 확인했다.

- 코드 충돌 없음
- 문서 `mydocs/orders/20260523.md` add/add 충돌만 존재
- 임시 merge 기준에서도 `samples/k-water-rfp.hwp` 는 29 pages
- 주요 `PartialTable` 분포는 순수 `upstream/devel` 과 동일

따라서 이번 Stage 1 결론은 PR #1085 포함 여부와 무관하게 유지된다.

## 3. Page-by-page 매칭

PDF 텍스트는 `pdftotext -layout` 로 추출했고, rhwp 쪽은 `dump-pages` 로 비교했다.

| 한컴 PDF | rhwp | 판정 |
|----------|------|------|
| p1 | p1 | 표지 정합 |
| p2 | p2 | 목차 정합 |
| p3~p5 | p3~p5 | 본문 시작 정합 |
| p6 | p6~p7 일부 | `pi=52` 이후 rhwp 가 뒤로 밀리기 시작 |
| p7~p8 | p7~p9 일부 | `pi=52` 여파로 +1 유지 |
| p9 | p9~p10 | PDF p9 의 `1.4. 제안서 작성요령` 이하가 rhwp p10 으로 밀림 |
| p13 | p14 | 첫 번째 +1 확정 |
| p14~p15 | p15~p17 | `pi=180` 이 3쪽으로 분할되어 두 번째 +1 발생 |
| p16~p22 | p18~p24 | +2 유지 |
| p23~p24 | p25~p26 | `pi=255` 는 2쪽 구조로 PDF 와 대응 |
| p25~p27 | p27~p29 | +2 유지 |

결론:

- `pi=52` 가 첫 번째 +1 을 만든다.
- `pi=180` 이 두 번째 +1 을 만든다.
- `pi=255` 는 직접 +1 원인으로 보이지 않는다.

## 4. 후보별 세부 진단

### 4.1 `pi=52` — 첫 번째 +1 원인

현재 rhwp 분할:

```text
page 5: PartialTable pi=52 rows=0..3 cont=false
page 6: PartialTable pi=52 rows=3..4 cont=true
```

한컴 PDF:

- PDF p5 는 row3 의 앞부분까지 표시한다.
- PDF p6 은 row3 의 나머지와 후속 본문 `(1)`~`질의사항...` 까지 포함한다.

rhwp 문제:

- row3 이 셀 내부로 분할되지 않고 통째로 p6 으로 넘어간다.
- p6 에 row3 전체가 올라가면서 후속 본문이 p7 로 밀린다.

구조상 핵심:

- `s1:pi=52` 는 4x4 RowBreak 표
- row2~row3 에 걸친 `cell[8]` 이 `row_span=2`
- row3 의 `cell[14]` 는 24 paragraphs 를 포함
- `cell[14]` 내부 paragraph p17 에서 `vpos=0` 으로 reset 된다.

현재 코드 경로에서 의심되는 지점:

```text
src/renderer/typeset.rs::typeset_block_table
```

`rowspan_touched[r]` 가 true 인 행은 컷 분할 불가로 처리된다.

```text
// rowspan 셀이 걸친 행 — 컷 분할 불가, MeasuredTable 높이로 통째 배치
if rowspan_touched[r] { ... }
```

`pi=52` row3 은 row2 에서 시작한 rowspan label 셀 때문에 `rowspan_touched=true` 가 되고, 실제로는 row3 의 큰 cell[14] 를 내부 분할해야 하는데 행 전체가 atomic 으로 취급된다.

### 4.2 `pi=180` — 두 번째 +1 원인

현재 rhwp 분할:

```text
page 15: PartialTable pi=180 rows=0..14
page 16: PartialTable pi=180 rows=14..31
page 17: PartialTable pi=180 rows=31..32
```

한컴 PDF:

- PDF p14 는 row14 (`SWNM`) 까지 포함한다.
- PDF p15 는 row15 (`수질예측`) 부터 row31 (`AI수력진단`) 및 주석까지 포함한다.

rhwp 문제:

- row14 가 page15 에 들어가지 못하고 page16 으로 밀린다.
- row31 이 page16 에 들어가지 못하고 page17 단독 fragment 로 떨어진다.

즉 한컴 기준으로는:

```text
page 15 equivalent: rows 0..15
page 16 equivalent: rows 15..32 + pi181 note
```

rhwp 는:

```text
page 15: rows 0..14
page 16: rows 14..31
page 17: rows 31..32 + pi181 note
```

분석 관점:

- `pi=180` 은 rowspan 원인이 아니다.
- 핵심은 row height / fit budget 이 한컴보다 보수적으로 계산되는 지점이다.
- 특히 row14 와 row31 이 각각 한 행씩 orphan 으로 밀리는 작은 차이다.

우선 확인할 코드:

```text
src/renderer/typeset.rs::typeset_block_table
src/renderer/layout/table_layout.rs::row_cut_content_height
src/renderer/layout/table_layout.rs::advance_row_cut
```

### 4.3 `pi=255` — 직접 원인 아님

현재 rhwp 분할:

```text
page 25: PartialTable pi=255 rows=0..7 start_cut=[] end_cut=[1,21]
page 26: PartialTable pi=255 rows=6..8 start_cut=[1,21] end_cut=[]
```

PDF 매칭:

- PDF p23: `[첨부2]` 시작과 표 앞부분
- PDF p24: 표 나머지
- rhwp p25~p26 이 같은 역할을 한다.

따라서 `pi=255` 는 현재 Stage 2 구현 대상에서 제외한다.

## 5. Stage 2 제안

Stage 2 는 두 개 변경을 분리해서 진행한다.

### Stage 2a — `pi=52` row3 셀 내부 split 복구

목표:

- RowBreak 표에서 `rowspan_touched` 행이라도, row_span label 때문에 불필요하게 atomic 처리하지 않는다.
- `cell[14]` 의 `vpos=0` reset 지점을 따라 row3 이 page5/page6 으로 나뉘도록 한다.
- 기존 rowspan / nested table split 회귀를 막기 위해 조건을 좁힌다.

1차 구현 후보:

- RowBreak 표 한정
- `rowspan_touched[r]` 라도 해당 행에 `row_span==1` 이고 splittable 한 큰 셀이 있으면 `advance_row_block_cut` 경로 사용
- 기존 `rowspan_touched` atomic path 는 비-RowBreak 또는 분할 불가 행에 유지

### Stage 2b — `pi=180` row fit 보수성 정정

목표:

- row14 가 첫 fragment 에 포함되도록 한다.
- row31 이 두 번째 fragment 에 포함되도록 한다.
- `pi=180` 을 3 fragment 에서 2 fragment 로 줄인다.

1차 구현 후보:

- `row_cut_content_height` 와 실제 `cell.height`/content height 차이를 row 단위로 계량
- row14/row31 이 탈락하는 정확한 budget 차이를 확인
- 필요한 경우 RowBreak 일반 행의 fit 판정에서 한컴 기준 높이 우선순위를 정정

## 6. 승인 요청

Stage 2 구현 방향 승인 요청:

1. Stage 2a 로 `pi=52` 의 rowspan-touched RowBreak 행 내부 split 을 먼저 복구한다.
2. 검증 후 Stage 2b 로 `pi=180` orphan row fragment 를 정정한다.
3. 각 단계마다 `dump-pages samples/k-water-rfp.hwp` 페이지 수와 후보 표 분포를 보고한다.
