# #5583 본문 수식-only 줄이 문단 정렬을 따르지 않는다

## 목표

가운데 정렬 문단에 놓인 수식이 단 왼쪽 끝에 붙는 증상을 없앤다.

## 원인 — 계측으로 확정

재현: 코퍼스 00192(`국가유산수리 감리대가 기준`) 2·3쪽 수식(폭 254.6px). 본문 좌측 75.6px 에
그려지고, 가운데라면 269.6px 여야 한다.

계측으로 세 가지를 차례로 배제·확정했다.

1. `layout_shape` 의 수식 배치는 정렬을 본다(`Center → 가운데`). 그러나 이 문서는
   `inline=true` 로 조기 반환한다 — 수식이 **글자처럼 취급(treatAsChar=1)** 이라 인라인 경로가
   좌표를 먼저 등록한다.

   ```
   DBG_EQ pi=7  inline=true align=Center colx=75.6 colw=642.5 w=254.6
   DBG_EQ pi=12 inline=true align=Center colx=75.6 colw=642.5 w=254.6
   ```

2. 문단 정렬 해석은 정상이다(`align=Center`).

3. 실제 x 를 정하는 곳은 `place_empty_line_inline_equations` 의 정렬 오프셋이었다.

   ```rust
   let align_offset = if cell_ctx.is_some() {
       match alignment { Center|Distribute => (avail - line_tac_width)/2.0, Right => …, _ => 0.0 }
   } else {
       0.0            // ← 본문이면 정렬을 아예 무시
   };
   ```

   **표 셀 안에서만 정렬을 계산하고 본문에서는 0.0 으로 굳어 있었다.**

## 변경

본문에서도 같은 정렬을 적용하되, 저장 `LINE_SEG.column_start > 0` 인 줄은 한컴이 흐름 x 를 적어 둔
경우이므로 #1256/#1308 계약대로 저장값을 존중한다. 00192 의 해당 줄은 `cs=0 sw=48188`(단 전체)로
위치 정보가 없다.

## 검증

| 항목 | 결과 |
|---|---|
| 재현 문서 00192 3쪽 수식 | x 75.6 → **269.6** (기대 가운데 269.6) |
| 표본에서 위치가 바뀐 수식 | 3개 — **전부 가운데로 이동**, 그 외 0 |
| 쪽수 회귀 (표본 1,000문서) | **변화 0건** |

## 검증 기준

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --test regression_suite_027 -- issue_5583` (계약 테스트 2건)
