---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 15 — HWP 그림 51 TAC table 이월과 후속 page shift

## 목적

Stage 14 commit `405b58a24` 이후의 222쪽 native HWP output을 기준으로 그림 51의 첫 잘못된 page break를
다시 확정한다. p77의 `treat_as_char=true` 2×1 `RowBreak` table(문단 876, Picture `bin_id=51` + `그림 51.`
caption)이 PDF처럼 p77의 각주 위에 함께 있어야 하는데, 기존 223쪽 output에서는 p78의 단독 table로 이월됐다.

이 단독 page가 맞다면 p78–p81, p83–p84, p87, p90, p99–p100의 같은-번호 PDF 대조가 차례로 의미가
달라지는 연쇄를 만든다. p83의 `para=897` FullParagraph overflow는 이 page shift와 별개로 남는지 확인한다.

사용자 시각 확인으로 아래 항목도 잔여 결함으로 고정한다. 번호 연쇄로 설명 가능한 항목과 독립 기하 결함을
구분할 때까지 어느 것도 해소로 표시하지 않는다.

- rhwp p31: 각주와 문단 하단 내용의 겹침
- rhwp p37: 그림 2개가 3개로 보이는 중복 렌더
- rhwp p43·p54·p66: 본문/표와 각주 영역의 겹침
- rhwp p76: 표 24가 기준의 5줄 대신 4줄만 표시됨
- rhwp p77: 그림 51이 없어야 할 p78로 단독 이월되고, p79에는 표가 없음
- rhwp p99–p100: 기준 PDF와 같은-번호 화면이 다름

## 출발 증거와 가설

- 기존 p77 render tree: preceding continuation table 866은 `h=243.2px`, 마지막 일반 본문은 `y=752.5px`,
  footnote top은 `y=945.3px`였다. table 876의 계산 높이는 `209.8px`여서 현재 흐름에는 들어가지 않아 p78에
  단독 배치됐다.
- p76→p77의 table 866 행 스캔은 p76에서 rows `0..4`만 완결하고 row 4 전체를 p77에 재배치한다.
  그러나 기준 PDF p77은 row 4의 tail부터 재개한다. 현재 스캔은 row 4 첫 cut을 계산
  (`budget=36.2px`, `content=21.6px`, `end_cut=[1,1,1]`)했지만 일반 25px orphan guard가 이를 기각했다.
  따라서 table 876 자체가 아니라 표 24의 짧은 native-HWP RowBreak tail을 통이월한 것이 그림 51 이월의
  first cause다.
- 기준 PDF p77에는 image와 caption이 각주 103·104 위에 모두 있다. 따라서 단순 TAC table near-fit 완화로
  overflow를 허용하면 안 되며, continuation table 866의 fragment row geometry·host spacing·captured picture
  height 중 어느 것이 p77 usable space를 과대 점유했는지 먼저 입증해야 한다.
- p83에서 기록된 `para=897` max 120.7px overflow는 p77 shift가 해소된 latest output에서도 존재하는지
  확인한다. 존재하면 line/full-fit 분기의 독립 결함이다.

## 작업 순서

1. latest native HWP p77–p84 selected sweep과 export-text로 그림 51 caption의 실제 owner page를 확인한다.
2. table 866/876의 source record, measured rows, render tree geometry를 PDF p77과 대조해 first cause를 문서화한다.
3. focused regression을 먼저 추가하고 최소 구현을 적용한다.
4. p76–p79를 우선 재검증해 row 4 tail·그림 51·후속 본문 owner page를 확인한다.
5. p31·37·43·54·66의 겹침, p83 overflow, p99–p100 차이를 page-shift 해소 뒤 각각 다시 판정한다.
   p66 UI path는 사용자 수동 WASM build와 native evidence의 revision identity를 읽기 전용으로 확인한 뒤
   별도 residual로 남긴다.

## 구현과 결과

### 표 24의 native HWP RowBreak reset tail

표 24의 row 4에는 저장된 `LINE_SEG.vpos`가 `0 → 1620 → 3240 → 0`으로 되감기는 내부 reset이 있다.
기존 일반 orphan guard는 p76에 남은 `21.6px` 조각을 최소 `25px`보다 작다고 기각해 row 전체를 p77로
이월했다. 이 때문에 p77에서 그림 51이 들어갈 높이가 사라졌다.

`typeset.rs`는 native HWP5의 비-TAC `RowBreak` table에서, 실제 저장 reset을 가진 continuation row만
painted height로 최소 보존량을 판단하도록 좁혔다. 이미 page에 각주가 있고 table 자신은 각주를 갖지 않는
`TopAndBottom` table은 실제 각주 경계까지 fragment를 배치한다. 일반 RowBreak, 일반 표, HWPX와 TAC 그림
table에는 이 조건을 열지 않는다.

- p76: row 4의 reset 전 세 줄(`생존 신장 기증자가 …`, `후 2년 …`, `위한 대기자 목록 …`)이 기준 PDF처럼
  남고 각주와 충돌하지 않는다.
- p77: reset 후 tail과 표의 다음 행이 이어지며 그림 51과 caption이 각주 103·104 위에 함께 남는다.
- p78: `3. EU`로 시작하고 그림 51이 남지 않는다.
- p79: 표 내용이 존재하며 그림 51의 단독 이월·빈 표 페이지가 아니다.

focused regression `issue_3738_rowbreak_table_footnote_fragment` 5개와 `cargo fmt --check`,
release-test binary build를 통과했다. native HWP 출력은 222쪽에서 221쪽으로 줄었지만 기준 PDF 215쪽과는
여전히 다르다. 이 단계는 p76–p79의 선택 흐름만 복원한 것이며 전체 pagination 완료를 주장하지 않는다.

## 시각 증적

연속 p76–p79 raster batch는 이 환경의 단일 실행 한도로 p76–p78만 완료됐다. p79는 별도 단일 sweep으로
완료해 선택 범위 네 페이지를 모두 확인했다. p76–p78 batch와 p79 단일 run의 structural flag는 모두 0건이다.
두 run의 pixel/ink 지표와 review PNG는
[Stage 15 visual sweep](task_m100_3738_stage15_visual_sweep.md)에, 보관 사본은
[`mydocs/pr/assets/pr_3740_issue3738_stage15/`](../pr/assets/pr_3740_issue3738_stage15/)에 둔다.

## 다음 Stage

사용자 화면에서 재관측한 p31·p37·p43·p54·p66, 기존 p83 overflow, p87·p90·p99–p100은 아직 해소로
표시하지 않는다. 이 커밋 뒤 Stage 16에서 최신 221쪽 출력과 기준 PDF의 semantic page owner를 다시 맞춘
뒤, 겹침·중복 그림·표 행 누락을 독립 원인으로 분석한다.
