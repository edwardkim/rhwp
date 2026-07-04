# task 1293 stage86 - stage85 full sweep 잔여 후보 원인 분석

## 목적

stage85 커밋 이후 전체 sweep에서 남은 후보를 원인별로 분류하고, 실제 한컴/PDF 흐름과 다른
항목을 수정한다. WIP/직전 커밋 비교만 믿지 않고 `output/task1293_stage85_full_sweep_after_commit`
의 sweep 산출물과 시각 판단을 기준으로 진행한다.

PR CI 전체 테스트는 사용자 별도 승인 전에는 수행하지 않는다.

## 기준

- 기준 커밋: `145da0bf task 1293: 구분선 없음 20mm 미주 흐름 보정`
- sweep 결과: `output/task1293_stage85_full_sweep_after_commit/summary.json`
- 모든 대상의 SVG/PDF/render tree 페이지 수는 1:1로 맞는다.
- stage84 핵심 회귀 대상 `2024-11-practice-above0-between0-below0`은 `flagged=0/21`이다.

## 우선 대상

stage85 직접 대상인 아래 샘플을 먼저 본다.

- HWP: `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp`
- PDF: `pdf/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.pdf`
- sweep key: `2024-11-practice-no-separator-above20-between20-below20`

stage85에서 page10, page11, page12의 직접 문제는 해결됐다. 남은 후보는 page13, 15, 16,
17, 18, 19, 20, 21, 22다.

## 초기 관찰

- page13은 문16~문19 marker가 모두 PDF보다 약 54px 아래에 있다.
- page17도 문23~문25 marker가 PDF보다 약 54px 아래에 있다.
- page18은 문26이 PDF 기준 page17 오른쪽 단 하단에 있어야 하는데 RHWP에서는 다음 page로
  넘어간 후보가 잡힌다.
- page20~22는 drift가 더 커지므로 앞쪽 page13/page17의 반복적인 54px 밀림을 먼저 해결해야 한다.

## 가설

반복적인 약 54px drift는 특정 문항 하나의 우발적 overwrap이라기보다, 구분선이 없는 20/20/20
미주 block에서 단/페이지 경계 tail을 판단할 때 한 줄 또는 gap을 과도하게 넘기는 공통 흐름 문제일
가능성이 높다.

특히 stage85에서 다룬 `line advance`와 실제 visible bbox의 차이가 page13 이후에도 반복되는지
확인한다. 단순 수치 보정이 아니라 다음 조건을 render tree 기준으로 검증한다.

- 현재 단 하단에 실제 visible bbox가 frame 안쪽으로 들어가는 다줄 문단을 통째로 넘기고 있는지
- 새 문항 제목과 첫 본문이 함께 이동해야 하는데 제목/본문 일부만 tail로 남는지
- 구분선 없음 상태에서 `구분선 위/아래` 여백을 선 자체와 혼동해 불필요한 block 높이를 예약하는지

## 분석 계획

1. page13 compare/render tree에서 문16~문19의 공통 y drift 시작점을 찾는다.
2. page12/page13 경계의 마지막 미주 tail을 `dump-pages`와 `export-render-tree`로 확인한다.
3. page17/page18 경계에서 문26이 다음 page로 밀리는 조건을 같은 방식으로 확인한다.
4. 공통 조건이 확인되면 `src/renderer/typeset.rs`의 tail fit 판단을 좁게 수정하고,
   `tests/issue_1139_inline_picture_duplicate.rs`에 해당 page 흐름 회귀 테스트를 추가한다.
5. focused test와 targeted sweep을 먼저 수행한다.

## 검증 대기

- `cargo build --bin rhwp`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_... -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 ...`

## 진행 기록

### page12/page13 경계 원인

stage85 full sweep의 page13 공통 54px 하방 drift는 page12 오른쪽 단 하단의 문15 마지막 다줄
tail(`pi=571`)이 4줄/2줄로 분할되면서 발생했다.

- stage85: page13 문16~문19가 모두 PDF보다 약 53~54px 아래에 표시됐다.
- 원인: 구분선 없는 큰 block에서 마지막 단 다줄 tail의 실제 visible bbox는 frame 안에 남는데,
  pagination은 마지막 단에서는 bleed fit을 허용하지 않아 다음 쪽으로 분할했다.
- 수정: 구분선 없는 큰 block의 다줄 visible tail fit을 마지막 단에도 적용하되, 마지막 단은 bleed
  extra를 주지 않고 높이 비율만 낮춰 `pi=571` 전체가 page12 오른쪽 단에 남게 했다.
- 회귀 테스트:
  `issue_1293_2024_no_separator_20mm_page12_question15_tail_keeps_page13_aligned`
  를 추가했다.

### page13 문19 후속 원인

`pi=571` 분할을 막은 뒤에도 page13에서 문19 본문/표(`pi=589`, `pi=590`, `pi=591`)가
오른쪽 단으로 너무 빨리 넘어가는 문제가 남았다.

- 원인: `large_between_tail_render_overflows`와
  `large_between_tail_before_rewind_picture`가 보이는 구분선이 있는 큰 미주 보정인데,
  구분선 없는 큰 block에도 적용되어 문19 제목 직후 본문을 다음 단으로 밀었다.
- 수정: 두 조건을 `has_visible_endnote_separator || !large_separator_block`로 좁혀,
  구분선 없는 큰 block에는 해당 보정을 적용하지 않게 했다.
- 결과: 문19 제목/본문은 PDF처럼 page13 왼쪽 단 하단에 남고, 되감김이 시작되는 후속
  문단/그림은 오른쪽 단으로 넘어간다.

이후 compare에서 문19 도표가 왼쪽 단 하단에 남아 문20이 PDF보다 위쪽으로 시작하는 후보가
남았다. 한컴/PDF는 문19 본문 끝까지는 왼쪽 단에 두고, 도표부터 오른쪽 단 상단으로 넘긴다.

- 원인: 구분선 없는 큰 block에서 현재 단 하단에 표-only 미주 문단이 오고 다음 문단 vpos가
  되감기는 경우도 표를 현재 단 tail로 유지했다.
- 수정: `no_separator_tail_table_starts_next_column` 조건을 추가해, 표-only tail이 현재 단
  하단에 있고 다음 문단이 되감기면 표부터 다음 단으로 넘긴다.
- 결과: 문19 도표(`pi=591`)는 page13 오른쪽 단 상단으로 이동하고, 문20 제목(`pi=594`)은
  PDF 위치 근처에서 시작한다.

### 검증 결과

- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_2024_no_separator_20mm -- --nocapture`
  - 통과: 3 passed
- targeted sweep:
  `output/task1293_stage86_no_separator_tail_fit_table_final/summary.json`
  - page count: SVG 23 / PDF 23
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=8/23`
  - stage85의 page13 `render_tree_frame_tail_overflow`, `column_line_band_drift`는 해소됐다.
  - 문19 도표 이동 후 page13은 sweep 후보 목록에서 제외됐다.

### 남은 후보와 판단

- page15: 문27 마지막 줄(`pi=676`)이 frame 아래로 약 11px 내려가는 후보가 남는다.
  원인을 좁히는 과정에서 q26 제목 뒤 같은 미주 본문 앞의 gap을 줄이는 가설을 시험했지만,
  전체 page count가 23에서 21~22로 줄어드는 과도한 압축이 발생해 폐기했다.
- 따라서 page15 이후는 단순히 `미주 사이` gap을 줄이는 방식으로 해결하면 안 된다.
  다음 단계에서는 q18~q20 또는 q25~q27의 우측 단 분배가 한컴/PDF와 다른 원인을
  render tree와 sweep annotation 기준으로 별도 분석해야 한다.

### page15 문26/문27 원인과 보정

page15 후보는 pagination 누적 높이가 아니라 렌더 단계의 vpos 보정 문제였다.

- `ENDNOTE_ADV` 기준으로 문26 제목/본문은 `cur=33.63 -> 45.63`으로 순차 배치된다.
- 하지만 `VPOS_CORR`에서 page-path 저장 vpos를 적용하면서 문26 본문(`pi=659`)이
  `y_in=211.92`에서 `result=297.49`로 약 85px 내려갔다.
- 기존 `compact_endnote_title_body_stale_forward`는 lazy-path 전용이라, page-path에서
  제목 바로 다음 본문에 남은 stale-forward vpos를 접지 못했다.

수정:

- `HeightCursor`에 `compact_endnote_page_title_body_stale_forward`를 추가했다.
- 조건은 `suppress_large_forward_jump`, page-path, 큰 `미주 사이`, 직전 문단이 문항 제목,
  현재 문단이 visible 본문이고 저장 vpos가 순차 y보다 32~120px 앞서는 경우로 제한했다.
- 이때 본문은 제목 뒤 순차 gap(`prev_line_spacing` 기반 10~18px)으로 배치하고,
  후속 문단도 같은 기준을 따르도록 vpos base를 같이 이동한다.

결과:

- 문26 본문(`pi=659`) 첫 줄: `y=297.5` → `y=221.9`
- 문27 제목(`pi=669`): `y=714.0` → `y=638.5`
- 문27 마지막 줄(`pi=676`): `y=1093.3`, bottom `1107.1` → `y=1029.8`, bottom `1043.6`
- targeted sweep:
  `output/task1293_stage86_no_separator_tail_fit_q27_final/summary.json`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=7/23`
  - page15는 후보 목록에서 제외됐다.
- 회귀 테스트 `issue_1293_2024_no_separator_20mm_page12_question15_tail_keeps_page13_aligned`에
  문26 본문, 문27 제목, 문27 마지막 줄 bbox 검증을 추가했다.

### page17~page22 문26~문30 연쇄 보정

page15 이후 남은 후보는 같은 설정의 page17~22에서 반복되는 단/쪽 경계 tail 판정이었다.
WIP/직전 커밋 비교가 아니라 최신 targeted sweep과 compare PNG를 기준으로 확인했다.

- page17 문26: 직전 미주의 마지막 line spacing이 이미 다음 번호와의 시각 gap을 만들고 있는데,
  새 문항 제목 앞에 `미주 사이 20mm`를 다시 예약해 문26 제목/첫 수식이 page18로 밀렸다.
  `no_separator_last_column_new_note_head_without_gap_fits`와 `HeightCursor`의
  `compact_no_separator_large_title_tail_gap`으로 제목과 첫 본문 줄이 frame 안에 들어가면
  page17 오른쪽 단 하단에 남기도록 했다.
- page18 문29: 문28 마지막 tail 뒤 문29 제목/첫 풀이 일부가 PDF처럼 page18 오른쪽 단 하단에
  남아야 한다. 구분선 없는 큰 block의 마지막 단에서는 title-tail gap을 다시 보존하지 않도록
  `layout.rs`에서 이미 cursor가 접은 큰 gap을 중복 보존하지 않게 했다.
- page19 문29/문30: 문29 마지막 수식(`pi=848`)은 page19 왼쪽 단 끝에 남고, 오른쪽 단은
  문30 제목(`pi=849`)부터 시작해야 한다. `no_separator_boundary_tail_without_gap_fits`를
  첫 단 하단에도 적용해 줄 자체가 frame 안에 들어가는 final tail에는 `구분선 아래/미주 사이`
  gap을 붙이지 않게 했다.
- page20 문24/문25: 같은 boundary-tail 규칙으로 문24 마지막 줄(`pi=902`)을 page20 왼쪽 단에
  남기고, 오른쪽 단은 문25(`pi=903`)부터 시작하도록 맞췄다.
- page21/page22 문29/문30: 큰 TAC 그림 뒤 follow-up tail을 page21 하단에 너무 많이 넣어
  page22 문30이 PDF보다 위로 당겨졌다. 구분선 없는 큰 block의 마지막 단에서 큰 TAC 그림 뒤
  텍스트 tail 한 줄을 이미 남긴 경우, 다음 tail부터 새 쪽으로 넘기는
  `no_separator_tail_after_picture_starts_next_page`를 추가했다. 결과적으로 page21은 `pi=965`
  까지만 남고, page22는 `pi=966..967`로 시작해 문30(`pi=976`)이 PDF bbox와 같은
  `y≈488px`에서 시작한다.

회귀 테스트:

- `issue_1293_2024_no_separator_20mm_page12_question15_tail_keeps_page13_aligned`
  - page17 문26 제목/첫 수식
  - page18 문29 제목/첫 tail
  - page19 문29 마지막 수식과 문30 시작
  - page20 문24 마지막 tail과 문25 시작
  - page21 그림 뒤 tail, page22 문30 위치

검증:

- `cargo build --bin rhwp`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_2024_no_separator_20mm -- --nocapture`
  - 통과: 3 passed
- targeted sweep:
  `output/task1293_stage86_no_separator_tail_after_picture_final/summary.json`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`
  - SVG/PDF/render tree: `23/23/23`

### 보이는 구분선 page10 문6 제목/본문 경계 보정

전체 sweep에서 `2024-11-practice-above20-between7-below2`가 `flagged=11/21`로 남았다.
compare page10을 기준으로 보면 PDF/한컴은 문6 제목만 왼쪽 단 하단에 남기고, 문6 본문/수식은
오른쪽 단 상단부터 이어진다. RHWP는 문6 본문을 왼쪽 단 하단에 계속 넣어 render tree 기준
frame 밖으로 최대 134px까지 내려갔고, 문7이 오른쪽 단 상단으로 너무 빨리 올라왔다.

원인:

- 이 샘플은 구분선이 보이고 `구분선 위 20mm`, `미주 사이 7mm`, `구분선 아래 2mm`다.
- 문6 제목 다음 첫 본문/수식은 저장 vpos가 제목 위치로 되감기는 패턴이다.
- 기존 local rewind advance는 pagination 누적 높이가 `available * 0.85`를 넘을 때만 단을 넘겼다.
- 그러나 `구분선 위 20mm`가 있는 구획에서는 렌더 vpos상 이미 단 하단인데, pagination 누적 높이는
  약 81%라서 단 이동이 발동하지 않았다.

수정:

- `visible_separator_title_body_rewind_starts_next_column` 조건을 추가했다.
- 보이는 구분선, 기본 `미주 사이`, 큰 `구분선 위`, 제목 바로 다음 본문, 첫 본문 vpos rewind,
  현재 단 80% 이상인 경우에만 제목은 현재 단에 남기고 본문부터 다음 단으로 넘긴다.
- 구분선 없는 20/20/20 대상에는 `has_visible_endnote_separator` 조건 때문에 적용되지 않는다.

검증:

- `cargo build --bin rhwp`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_2024_visible_separator_above20_between7_question6_body_starts_right_column -- --nocapture`
  - 통과: 1 passed
- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_2024_no_separator_20mm -- --nocapture`
  - 통과: 3 passed
- targeted sweep:
  `output/task1293_stage86_visible_separator_title_body_rewind/summary.json`
  - `2024-11-practice-above20-between7-below2`: `flagged=11/21` → `flagged=3/21`
  - page10 후보는 해소됐다.
- no-separator 회귀 targeted sweep:
  `output/task1293_stage86_after_visible_separator_guard_no_separator_regression/summary.json`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`
- `2024-11-practice-above20-between0-below20` targeted sweep:
  `output/task1293_stage86_visible_separator_above20_between0_check/summary.json`
  - 여전히 `flagged=9/21`
  - `미주 사이 0`은 제목/본문 rewind와 다른 원인으로 남았다.

## 전체 sweep 결과

page22 문30 보정 후 전체 sweep을 다시 실행했다. 이후
`2024-11-practice-above20-between7-below2`만 추가 보정해 targeted sweep을 한 번 더 수행했다.

- 산출물: `output/task1293_stage86_full_sweep_after_page22_final/summary.json`
- 직접 대상 `2024-11-practice-no-separator-above20-between20-below20`은 targeted sweep과
  full sweep 모두 `flagged=0/23`이다.

| target | SVG/PDF/render tree | flagged pages |
|---|---:|---|
| `2022-09` | 23/23/23 | 10, 16, 20 |
| `2023-09` | 20/20/20 | 11, 14, 19 |
| `2024-09-below20` | 23/23/23 | 10, 16, 20 |
| `2024-09-between20` | 24/24/24 | 11 |
| `2024-09-below20-above20` | 23/23/23 | 10, 13, 14, 19, 22 |
| `2022-10` | 18/18/18 | 10, 12, 14 |
| `2022-11-practice` | 21/21/21 | 11, 16 |
| `2024-09-below20above20` | 23/23/23 | 10, 13, 14, 19, 22 |
| `2024-11-practice-shape987` | 21/21/21 | 11, 12, 14, 16, 17, 18, 19, 20 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 없음 |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | 11, 16 |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | 11, 12, 14, 16, 19, 20 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 11, 13, 14, 15, 19, 20, 21, 22 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 10, 11, 13, 15, 16, 17, 18, 19, 20 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | full: 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20 / 최신 targeted: 11, 12, 16 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 없음 |

핵심 회귀 대상인 `2024-11-practice-above0-between0-below0`은 계속 `flagged=0/21`이다.
stage86 직접 대상인 `2024-11-practice-no-separator-above20-between20-below20`은 stage85의
`flagged=9/23`에서 최종 `flagged=0/23`까지 정리됐다.

남은 후보는 대부분 보이는 구분선이 있는 2024-11 실전 샘플에서 발생한다. 특히
`above20-between7-below2`, `above20-between0-below20`,
`above0-between20-below2`, `shape987`은 여러 페이지에 걸쳐 `column_line_band_drift`,
`tail`, `question` 후보가 동시에 잡힌다. 직접 대상의 구분선 없는 20/20/20 보정이 완료됐으므로,
이제는 보이는 구분선이 있는 미주 구획의 `구분선 위/미주 사이/구분선 아래` 적용 순서와
page/column 경계 tail 정책을 별도로 좁혀야 한다.

## 공통 미주 모델 전환

작업지시자 피드백에 따라 page별 증상 보정을 중단하고, 전체 문제집에 반복되는 공통 미주 처리
오류를 먼저 분리한다. 기준은 한컴 미주 모양 UI의 의미다.

- `구분선 위`: 본문 끝과 미주 구분선 사이 여백
- `구분선 아래`: 구분선과 첫 미주 내용 사이 여백
- `미주 사이`: 번호가 있는 이전 미주 내용 끝과 다음 번호 미주 내용 시작 사이 여백

따라서 `미주 사이`는 같은 미주의 제목과 본문 사이 gap으로 쓰면 안 되고, page/column 경계에서
직전 미주 tail의 line spacing으로 이미 보이는 경우 `vpos_offset`에 한 번 더 더하면 다음 번호가
한컴보다 약 `미주 사이`만큼 아래로 내려간다.

### matrix 재검증

다음 matrix를 같은 sweep 스크립트로 비교했다.

- `2024-11-practice-above0-between0-below0`: `flagged=0/21`
- `2024-11-practice-above0-between7-below2`: `flagged=2/21`
- `2024-11-practice-above0-between7-below20`: `flagged=6/21`
- `2024-11-practice-above0-between20-below2`: `SVG/PDF=23/22`, `flagged=10/22`
- `2024-11-practice-above20-between0-below20`: `flagged=5/21`
- `2024-11-practice-above20-between7-below2`: `flagged=3/21`
- `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`

0/0/0과 구분선 없음 20/20/20은 안정적이다. 남은 문제는 보이는 구분선에서 큰 `미주 사이` 또는
큰 구분선 주변 여백을 page/column 경계에 중복 소비하는 축으로 모인다.

### 공통 수정 1: 새 미주 제목 뒤 첫 본문 tail

`above0-between20-below2`의 page11/page12 경계에서 문13 제목은 현재 단 하단에 남지만,
같은 미주의 첫 본문(`ep=1`) 2줄은 `split=None` 상태로 통째로 다음 쪽으로 밀렸다.

- 원인: 남은 높이에 bottom bleed를 포함하면 문단 전체가 들어가는데, 기존 split 로직은
  `split == 전체 줄 수`일 때 split 후보를 버리고 이후 `advance_for_fit`로 빠졌다.
- 수정: 보이는 구분선, 큰 `미주 사이`, 마지막 단, 직전 배치가 같은 미주의 제목이고 현재 문단
  전체 line advance가 bleed 한도 안에 들어갈 때는 `large_between_last_column_title_body_tail_fits`
  로 advance를 막아 같은 단 tail에 남긴다.
- focused 결과: 문13 `ep=1`은 `advance_fit=false`가 되고, 다음 문단(`ep=2`)부터 새 쪽으로 넘어간다.

이 수정만으로 page count mismatch는 해소되지 않았지만, 제목만 남고 본문이 통째로 밀리는 공통
회귀는 줄었다.

### 공통 수정 2: 직전 미주 tail 뒤 `미주 사이` vpos 중복 소비

같은 대상의 page12에서 문14가 PDF보다 약 72px 아래에 있었고, 이 값은 `미주 사이 20mm`
환산값과 거의 같다. 문13 tail이 page12 왼쪽 단 상단에 이미 이어진 상태에서 문14 시작 전
`vpos_offset`에 `미주 사이` 초과분을 다시 더해 이중 gap이 생겼다.

- 1차 시도: `vpos_offset`과 새 미주 advance gap을 모두 끄자 문13/문14가 전체적으로 과도하게
  압축되어 폐기했다.
- 수정: 새 미주 advance gap은 유지하고, 보이는 구분선 + 큰 `미주 사이` + 직전 미주 tail이 같은
  단/쪽 상단(`current_height < available * 0.25`)에 이어진 경우에만 `vpos_offset` 추가를 막는다.
- 결과:
  - `above0-between20-below2` targeted sweep: `flagged=10/22` -> `flagged=9/22`
  - page12 문15는 PDF처럼 왼쪽 단 하단에 남고, 문16도 오른쪽 단 하단으로 내려왔다.
  - page13 문17~문19 marker는 PDF와 거의 같은 y로 맞았다.

### 남은 공통 원인

page13의 문20은 RHWP에서 왼쪽 단 하단에 제목만 남지만, PDF/한컴은 오른쪽 단 상단에서 시작한다.
디버그상 문20 자체의 제목/본문 fit 문제가 아니라, 직전 문19의 큰 TAC 그림을 pagination이 실제
시각 높이보다 낮게 소비해 `current_height`가 약 806px에 머문다. 즉 다음 축은 특정 문항 보정이
아니라 미주 안의 큰 TAC 그림/도형 문단에서 lineSeg vpos와 실제 object bbox 높이를 함께 반영하는
공통 높이 모델이다.

다음 작업은 page별 숫자 보정이 아니라 아래 순서로 진행한다.

1. treat-as-char TAC 그림/수식/도형 문단의 lineSeg 높이와 실제 bbox 높이 차이를 render tree에서
   공통 지표로 뽑는다.
2. lineSeg가 실제 object bottom을 이미 포함하는 경우와, formatter가 object 높이를 별도로 소비해야
   하는 경우를 분리한다.
3. 문19처럼 큰 TAC 그림 뒤 새 문항 제목이 잘못 tail로 남는 케이스를 `current_height`가 아니라
   render-vpos/visible-bottom 기준으로 판단한다.
4. 수정 후 0/0/0, 구분선 없음 20/20/20, 보이는 구분선 between20/between0/between7 matrix를 다시
   sweep한다.

### 공통 수정 3: render-y 기반 새 미주 시작 판단

page13 문20은 `current_height` 기준으로는 왼쪽 단에 제목과 일부 본문이 들어가는 것처럼 보였지만,
render tree에서는 문20의 세 번째/네 번째 문단(`pi=596`, `pi=597`)이 frame 아래로 overflow됐다.
원인은 새 미주 제목 시작 위치를 `first_vpos - base_vpos` 단순 차분으로 판단해, 직전 문19의 비TAC
표와 TAC 그림이 렌더러에서 실제로 소비한 y 진행량을 반영하지 못한 것이다.

수정은 문항 번호가 아니라 현재 단의 기존 `PageItem`을 `HeightCursor`로 재생하는 공통 helper를
추가하는 방식으로 했다.

- `predict_current_column_para_y`: 현재 단의 `FullParagraph`, `PartialParagraph`, `Table`,
  `PartialTable`을 렌더러와 같은 vpos cursor로 재생해 다음 문단의 render-y를 예측한다.
- `Table`/`PartialTable`은 `measured_tables`의 실제 높이를 반영한다. 기존 중복 예측 코드처럼 표를
  0px로 보면 문19 표 뒤의 문20 위치가 낮게 산정된다.
- 보이는 구분선 + 큰 `미주 사이` + 첫 단에서 새 문항 제목이 시작될 때, 제목 한 줄이 아니라 같은
  미주의 선두 풀이 그룹(앞 4개 미주 문단)이 render-y 기준으로 frame을 넘으면 제목만 남기지 않고
  다음 단으로 advance한다.
- 단, `endnote_has_vpos_rewind=true`인 경우는 한컴이 제목 tail을 허용하는 패턴이 있어 제외했다.
  실제로 문5는 `render_y=975.49`, `has_rewind=true`라 왼쪽 단 하단에 남아야 하고, 문20은
  `render_y=1035.21`, `has_rewind=false`라 오른쪽 단으로 넘어가야 한다.

검증 결과:

- 문20: `lead_group_outside=true`, `advance_new=true`로 전환.
- render tree: page13 `pi=596/597`의 `LAYOUT_OVERFLOW` 사라짐.
- targeted sweep `2024-11-practice-above0-between20-below2`:
  - 이전: `flagged=9/22`, p10 회귀 없이 검증 전에는 p13 qflow/frame 문제가 남음.
  - 이후: `flagged=8/22`, `frame=[]`, `qflow=[]`, p10 회귀 없음.
- 대표 matrix:
  - `above0-between20-below2`: `flagged=8/22`, `SVG/PDF=23/22`
  - `above20-between0-below20`: `flagged=5/21`, `SVG/PDF=21/21`
  - `above20-between7-below2`: `flagged=3/21`, `SVG/PDF=21/21`
  - `no-separator-above20-between20-below20`: `flagged=0/23`, `SVG/PDF=23/23`

남은 핵심은 `above0-between20-below2`의 page count mismatch다. 문20은 PDF와 같은 p13 오른쪽 단
상단으로 맞았지만, 뒤쪽 page15~22에서 여전히 누적 흐름 차이가 있어 다음 분석은 큰 `미주 사이`
문서의 중후반 누적 gap/line band drift 공통 원인을 대상으로 한다.

### 공통 수정 4: endnote 내부 lineSeg reset split 보존

`above0-between20-below2`의 page15는 문27 마지막 tail이 다음 단으로 이어져야 하는데,
pagination이 `line_seg[1].vpos=0`인 내부 reset을 단순 single-line rewind로 보아 split을 지우면서
문27 tail 전체를 왼쪽 단 하단에 남겼다.

- HWP 구조: `pi=671`, `line_seg[0].vpos=6905`, `line_seg[1].vpos=0`
- 문제: `split=1`을 제거하면 page15 왼쪽 단에서 `pi=671`의 35~37행이 frame 밖으로 내려간다.
- 수정: 보이는 구분선, 큰 `미주 사이`, 마지막 단, 내부 rewind 대상 lineSeg가 실제 reset
  (`vertical_pos == 0`)인 경우에는 `split=1`을 보존한다.
- 결과:
  - `pi=671`은 `PartialParagraph lines=0..1`과 `lines=1..4`로 분리된다.
  - `export-render-tree`에서 `pi=670/671` frame overflow가 사라졌다.
  - `above0-between20-below2` targeted sweep은 `SVG/PDF/render tree=22/22/22`,
    `flagged=7/22`가 됐다.

이 수정은 특정 문항 번호가 아니라 lineSeg reset이라는 HWP layout 신호를 보존하는 공통 처리다.

### 7개 대표 matrix 재검증

작업지시자 피드백에 따라 문항별 보정이 아니라 미주 모양 옵션 조합별 공통 실패 축을 다시 보았다.
산출물은 `output/task1293_stage86_common_endnote_matrix7/summary.json`이다.

| target | SVG/PDF/render tree | flagged pages | 주요 후보 |
|---|---:|---:|---|
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 0 | 없음 |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | 2 | p11/p16 title 또는 large drift 후보 |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | 6 | p11/p14/p19 tail, p14/p20 marker drift |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 7 | p14/p16/p19 tail, p12/p17/p21 marker drift |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 5 | p18 tail, p11/p19/p20 content/marker drift |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 3 | p11 tail, p12/p16 title 후보 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 0 | 없음 |

패턴:

- `0/0/0`과 `구분선 없음 20/20/20`은 안정적이다.
- 남은 문제는 보이는 구분선이 있을 때만 반복된다.
- `구분선 아래20`은 `미주 사이7`에서도 tail/marker drift를 크게 늘린다.
- `미주 사이20`은 page count는 맞지만 여러 page에서 tail/line/column drift를 남긴다.
- `구분선위20 + 구분선아래20`은 `미주 사이0`에서도 drift가 남으므로, `미주 사이`만 줄이거나
  늘리는 방식은 근본 해결이 아니다.

따라서 다음 수정은 더 이상 특정 문항/쪽 번호를 조건으로 삼지 않는다. 보이는 구분선이 있는
endnote flow에서 다음 세 boundary를 공통 상태로 분리해야 한다.

1. 본문 영역 끝에서 separator가 실제로 표시되는 첫 미주 block까지의 `구분선 위` 예약
2. separator와 첫 미주 내용 사이의 `구분선 아래` 예약
3. 번호가 있는 이전 미주 내용 끝과 다음 번호 미주 내용 시작 사이의 `미주 사이` 예약

현재 코드에는 이 세 boundary가 `line_spacing`, `vpos_offset`, `advance_for_fit`,
`HeightCursor` 보정에 흩어져 있고, column/page 경계에서 중복 소비 또는 미소비가 발생한다.
stage86의 다음 작업은 이 세 boundary를 한 곳에서 판단하도록 공통 helper를 만들고,
보이는 구분선/구분선 없음의 정책을 분리하는 것이다.

### 공통 수정 5: reset split head의 render-y overflow 판정

작업지시자 피드백에 따라 문항별 보정 대신 visible separator + 큰 `미주 사이` 조합에서 반복되는
공통 실패를 다시 추적했다. `above0-between20-below2` page16 문30은 sequential pagination 기준으로
`pi=754`의 internal reset split head(`lines=0..2`)가 들어가는 것처럼 보였지만, render tree에서는
첫 줄부터 frame bottom 아래에서 시작했다.

- 원본 구조: `s0:p259:ci0:note33`, `line_seg[2].vpos=0`
- 기존 판단: `internal_rewind_split=Some(2)`, `internal_rewind_head_allows_current_column=true`
- 실제 render tree: `pi=754` 첫 줄 bbox `y=1097.6`, frame bottom `1096`
- 원인: `lineSeg reset`은 보존해야 하지만, 보존할 head 자체가 render-y 기준으로 frame 밖이면 현재
  단 tail로 남기면 안 된다.

수정:

- `predict_current_column_para_y`로 현재 단의 실제 render-y를 예측한다.
- visible separator, 비기본 `미주 사이`, 마지막 단, `line_seg[split].vertical_pos == 0`, `split > 1`
  조건에서 reset split head가 render-y 기준 frame을 넘으면 `advance_for_fit`이 작동하도록 했다.
- advance 후에는 이전 단 기준의 `internal_rewind_split`을 재사용하지 않고 다음 단/쪽에서 다시
  판단하도록 reset한다.

검증:

- `cargo fmt && cargo build --bin rhwp`: 통과
- targeted sweep 산출물: `output/task1293_stage86_internal_reset_head_gate_probe`
  - `above0-between20-below2`: `SVG/PDF/render tree=22/22/22`
  - 기존 `flagged=7/22` -> `flagged=5/22`
  - p16/p17의 `render_tree_frame_tail_overflow` 후보가 사라졌다.
- 7개 대표 matrix 산출물: `output/task1293_stage86_common_matrix_after_reset_head`
  - `above0-between0-below0`: `flagged=0/21` 유지
  - `no-separator-above20-between20-below20`: `flagged=0/23` 유지
  - `above0-between20-below2`: `flagged=5/22`로 개선

### 폐기한 시도: 일반 tail render-y 보강

남은 page19 문30 tail도 sequential 높이와 render-y가 어긋나는 패턴처럼 보였으나,
기존 `large_between_tail_render_overflows`에 `predict_current_column_para_y`를 넓게 적용하자
`above0-between20-below2`가 `SVG/PDF=23/22`로 회귀했다. p19 tail은 pagination advance로 통째로
넘길 문제가 아니라, 같은 쪽에 남아야 하는 문단의 render-y 또는 경계 기준을 조정해야 하는 문제로
분류한다.

따라서 다음 수정은 `large_between_tail_render_overflows`를 넓게 advance시키는 방식이 아니라,
`HeightCursor`의 visible separator + 큰 미주 사이 page/lazy base 전환 또는 `구분선 아래/미주 사이`
경계 소비 위치를 더 좁게 분석한다.

### 공통 수정 6: 문항 번호 특례 제거와 설정 기반 기본-gap tail 후보

작업지시자 피드백에 따라 `문29`, `문30`처럼 특정 문항 번호를 직접 조건으로 쓰던 보정을
공통 미주 설정 신호로 바꾸었다. 하나씩 문제를 고치는 방식은 다른 문제집/쪽에서 회귀를 계속
만들기 때문에, stage86에서는 문항 번호가 아니라 다음 구조만 본다.

- compact endnote separator profile이다.
- `미주 사이` 값이 0보다 크고, 기본 flow 기준 이하이다.
- 실제 미주 번호가 있는 항목이다.

이를 `default_nonzero_between_note_tail_candidate`로 묶고, 기존
`matches!(en_ref.number, 29 | 30)` 및 `en_ref.number == 29` 기반 late question tail 조건을
이 구조 신호로 대체했다. `미주 사이 0`은 별도 계열이므로 이 후보에서 제외해, `0/0/0` 샘플의
안정 상태를 유지한다.

검증:

- `cargo fmt && cargo build --bin rhwp`: 통과
- focused sweep:
  `output/task1293_stage86_common_tail_candidate_final_probe/summary.json`

| target | 기준 matrix | focused sweep | tail 후보 변화 |
|---|---:|---:|---:|
| `2024-11-practice-above0-between0-below0` | `flagged=0/21` | `flagged=0/21` | `[] -> []` |
| `2024-11-practice-above0-between7-below20` | `flagged=6/21` | `flagged=5/21` | `[11,14,19] -> [11,19]` |
| `2024-11-practice-above20-between0-below20` | `flagged=5/21` | `flagged=4/21` | `[18] -> [17]` |
| `2024-11-practice-above20-between7-below2` | `flagged=3/21` | `flagged=3/21` | `[11] -> [11]` |

### 공통 미주 처리 원칙 재정의

작업지시자 재피드백에 따라 stage86 후반부터는 문항별/쪽별 보정을 중단한다. 남은 문제는
`문20`, `문26` 같은 특정 문항의 문제가 아니라 보이는 구분선이 있는 미주 flow에서 세 종류의
미주 모양 값이 서로 다른 위치에서 중복 소비되는 문제다.

앞으로 코드 조건은 다음 값만 직접 본다.

- `구분선 위`: 본문 끝과 구분선 사이 예약 높이
- `구분선 아래`: 구분선과 첫 미주 내용 사이 예약 높이
- `미주 사이`: 이전 번호 미주 내용 끝과 다음 번호 미주 내용 시작 사이의 최소 gap
- 구분선 표시 여부
- lineSeg reset/rewind, treat-as-char 수식/그림, textless object tail 같은 구조 신호

따라서 특정 문제 번호, 특정 쪽 번호, 특정 파일명으로 흐름을 가르는 조건은 폐기 대상이다.
기존 WIP에 남아 있던 `문29/문30` 직접 조건은 설정 기반 후보로 바꾸었고, layout 단계에 남은
`question_number < 29` 필터도 제거해 "단일 수식 tail 뒤 새 미주 제목"이라는 구조 조건으로
좁힌다.

검증도 한 문항 단위가 아니라 대표 matrix 기준으로 수행한다.

- 안정 기준: `above0-between0-below0`, `no-separator-above20-between20-below20`
- 보이는 구분선 기본값: `above0-between7-below2`, `above20-between7-below2`
- 큰 구분선 주변 여백: `above0-between7-below20`, `above20-between0-below20`
- 큰 미주 사이: `above0-between20-below2`

### 공통 수정 7: 기본 미주 사이와 큰 구분선 margin의 boundary 중복 제거

대표 matrix에서 `above0-between7-below20`와 `above20-between7-below2`는 page11에서
`between_notes_marker_gap` delta가 정확히 `26.5px`였다. 이는 7mm 기본 `미주 사이` 환산값과
같다.

원인:

- 보이는 구분선이 있는 미주에서 직전 미주 tail이 같은 단에 이미 이어진다.
- 직전 tail의 `line_spacing`에 7mm `미주 사이`를 다시 주입한다.
- 큰 `구분선 위` 또는 `구분선 아래` margin 때문에 같은 경계의 새 미주 시작 판단이 늦어지며,
  결과적으로 7mm가 한 번 더 보이는 gap으로 남는다.

수정:

- `EndnoteFlowProfile`을 추가해 `구분선 위`, `구분선 아래`, `미주 사이`, 구분선 표시 여부를
  한 곳에서 정규화했다.
- 보이는 구분선 + 기본 non-zero `미주 사이` + 큰 separator margin + 현재 단에 직전 미주 tail이
  이어지고, 단 높이 70~75%의 중후반 boundary 구간에서는 renderer용 직전
  `line_spacing = between_notes` 주입을 하지 않는다.
- 단 하단의 새 문항 제목 tail은 한컴이 7mm gap을 보존하므로 이 조건에서 제외한다.
- `미주 사이 20mm`, `미주 사이 0`, 구분선 없는 20/20/20은 이 조건에서 제외한다.

검증:

- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture`
  - 통과: 8 passed
- 대표 matrix sweep:
  `output/task1293_stage86_common_boundary_gap_band_matrix/summary.json`

| target | 이전 대표값 | 수정 후 | 판단 |
|---|---:|---:|---|
| `above0-between0-below0` | `0/21` | `0/21` | 안정 유지 |
| `above0-between7-below2` | `2/21` | `2/21` | 안정 유지 |
| `above0-between7-below20` | `4/21` | `2/21` | p11/p19 7mm 중복 tail 해소 |
| `above0-between20-below2` | `5/22` | `5/22` | 큰 미주 사이 계열은 별도 유지 |
| `above20-between0-below20` | `4/21` | `4/21` | zero-between 계열은 별도 유지 |
| `above20-between7-below2` | `3/21` | `2/21` | p11 7mm 중복 tail 해소 |
| `no-separator-above20-between20-below20` | `0/23` | `0/23` | 안정 유지 |

7mm 계열에서 남은 page12/page16 후보는 `between_notes_marker_gap`이 0~8px 범위로 맞고,
render tree frame overflow도 없다. compare PNG 기준으로 실제 미주 overflow라기보다 line-band/title
검출 후보에 가깝다. 다음 실제 수정 대상은 20mm `미주 사이`와 zero-between 계열의 marker/page 흐름이다.

폐기한 시도:

- `구분선 위/아래`가 크면 separator pagination height를 별도로 예약하는 시도는 7개 matrix에서
  수치 변화가 없어 폐기했다.
- 문항 번호 조건을 무조건 넓게 제거하는 시도는 `above0-between0-below0`을 `flagged=0`에서
  `flagged=2`로 회귀시켜 폐기했다. 기본-gap 후보는 반드시 `미주 사이 > 0`이어야 한다.
- 기본-gap + 큰 visible separator 여백에서 render-y 기준 split을 추가하는 시도는 focused sweep
  결과가 개선되지 않아 제거했다. 전역 bleed tolerance 대신 `available + 2px`로 더 엄격히 보이는
  line bottom을 검사하는 변형도 확인했지만,
  `output/task1293_stage86_visible_bottom_strict_probe/summary.json`에서
  `above0-between7-below20`은 `tail=[11,19]`에서 `tail=[11,12,19]`로,
  `above20-between7-below2`는 `tail=[11]`에서 `tail=[11,12]`로 악화했다. 따라서 남은 문제는
  현재 문단을 단순히 라인 단위로 더 빨리 쪼개는 문제가 아니라, 미주 구분선/미주 사이/저장 vpos가
  column/page 경계에서 소비되는 공통 flow 기준 문제로 본다.

남은 방향:

- 남은 후보는 특정 문항이 아니라 visible separator가 있는 endnote flow에서 `구분선 위`,
  `구분선 아래`, `미주 사이`, 저장 `line_seg.vertical_pos` reset/rewind가 column/page 경계에서
  언제 소비되는지의 공통 문제다.
- 다음 분석은 문제 번호가 아니라 `lineSegArray > line_seg`, render tree bbox, 저장 vpos reset,
  글자처럼 취급해야 하는 TAC 수식/그림 여부를 함께 보아야 한다.

### 공통 수정 7: 큰 TAC 그림 tail은 비가시 bleed 예외에서 제외

`above0-between7-below20` page18/page19에서 문30 그래프가 한컴/PDF와 달랐다. RHWP는
문30 그래프(`pi=882`)를 page18 오른쪽 단 하단에 남겼고, PDF/한컴은 같은 그래프를 page19
왼쪽 단 첫 항목으로 보낸다.

이 문제는 문30 특례가 아니라 미주 안의 TAC 객체 높이 모델 문제였다.

- `pi=882`는 텍스트가 없는 TAC 그림/도형 단독 문단이다.
- render tree 실제 bbox는 `y≈899.9px`, `h≈209.4px`로 frame bottom을 넘는다.
- 하지만 pagination 쪽 `fmt`/일반 tail 판단은 이 문단을 비가시 tail처럼 취급해
  `non_visible_endnote_tail_bleeds_previous_column`으로 현재 단 bleed를 허용했다.
- 즉 빈 텍스트 tail 허용 규칙이 큰 TAC 그림까지 같이 허용해, 실제 보이는 객체가 frame 아래로
  남는 구조였다.

수정:

- `visible_separator_large_tac_tail_overflows_frame`을 추가했다.
- 보이는 구분선, 0/0/0이 아닌 compact 미주, TAC 그림/도형 단독 문단, 실제 객체 높이가 충분히 큰
  경우에만 `predict_current_column_para_y`로 현재 단의 render-y를 예측한다.
- `render_y + TAC 객체 높이 > frame bottom`이면 큰 TAC tail은 다음 단/쪽으로 advance한다.
- 이 경우 기존 `non_visible_endnote_tail_bleeds_previous_column` 예외가 advance를 막지 못하게 했다.

검증:

- `cargo fmt && cargo build --bin rhwp`: 통과
- 직접 dump:
  - 수정 전 page18 오른쪽 단: `pi=881`, `pi=882 Shape tac=true`
  - 수정 후 page18 오른쪽 단: `pi=881`까지만 남음
  - 수정 후 page19 왼쪽 단 시작: `pi=882 Shape tac=true`, 이어서 `pi=883`
- focused sweep:
  `output/task1293_stage86_large_tac_render_tail_probe/summary.json`

| target | 이전 focused | 이번 focused | 변화 |
|---|---:|---:|---|
| `2024-11-practice-above0-between0-below0` | `flagged=0/21` | `flagged=0/21` | 안정 유지 |
| `2024-11-practice-above0-between7-below20` | `flagged=5/21` | `flagged=4/21` | p20 marker/drift 후보 해소 |
| `2024-11-practice-above20-between7-below2` | `flagged=3/21` | `flagged=3/21` | 회귀 없음 |

추가 회귀 테스트:

- `issue_1293_2024_visible_separator_large_tac_picture_tail_starts_next_page`
  - page18에는 `pi=881`까지만 남고 `pi=882`는 없어야 한다.
  - page19는 `pi=882` TAC 그림에서 시작하고 후속 `pi=883`으로 이어져야 한다.
  - render tree에서 문30 그래프 bbox는 page19 왼쪽 단 상단에 있어야 한다.

### 공통 수정 8: 큰 `미주 사이` title-tail 저장 vpos backtrack

`above0-between20-below2`는 보이는 구분선이 있고 `미주 사이`가 20mm인 대표 큰 gap 샘플이다.
page12/page14에서는 새 문항 제목 직전 spacer의 `line_spacing=5669`가 이미 문항 사이 간격을
만드는데, render 단계에서 제목/첫 본문이 저장 vpos보다 약 20px 낮은 y로 유지되어 하단 tail이
frame 아래로 내려갔다.

공통 원인:

- `HeightCursor`의 injected-between-notes 보존 분기가 직전 spacer의 큰 line spacing을 그대로
  유지했다.
- page12 문16, page14 문23처럼 현재 단 80% 이후에 새 문항 제목이 시작되는 경우, 저장 vpos
  위치는 frame 안쪽이고 한컴/PDF도 해당 제목/본문을 조금 위쪽에서 시작한다.
- 따라서 큰 `미주 사이`를 새로 더하는 것이 아니라, 저장 vpos가 이미 표현한 title-tail 위치를
  신뢰해야 한다.

수정:

- `compact_endnote_large_between_title_tail_backtrack`을 추가했다.
- 보이는/없는 구분선 구분 없이 compact 미주, 큰 `미주 사이`, 새 문항 제목, 단 하단 80% 이후,
  저장 vpos가 현재 y보다 8~36px 위이고 frame 안쪽이면 저장 vpos로 backtrack한다.
- 이 조건에서는 injected-between-notes 보존 분기가 다시 `y_offset`으로 되돌리지 못하게 제외했다.

검증:

- `cargo test --lib compact_endnote_large_between_title_tail_uses_saved_vpos_at_bottom -- --nocapture`
  - 통과: 1 passed
- focused matrix sweep:
  `output/task1293_stage86_large_between_title_tail_backtrack_matrix/summary.json`

| target | 결과 | 판단 |
|---|---:|---|
| `above0-between0-below0` | `flagged=0/21` | 안정 유지 |
| `above0-between7-below2` | `flagged=2/21` | 기존 수준 유지 |
| `above0-between7-below20` | `flagged=2/21` | 기존 수준 유지 |
| `above0-between20-below2` | `flagged=5/22` | frame overflow는 줄었지만 tail/column 후보는 잔존 |
| `above20-between0-below20` | `flagged=0/21` | 안정 유지 |
| `above20-between7-below2` | `flagged=2/21` | 기존 수준 유지 |
| `no-separator-above20-between20-below20` | `flagged=0/23` | 안정 유지 |

남은 판단:

- `above0-between20-below2`의 page12/page14/page19/page22 후보는 단일 문항 특례가 아니라
  큰 `미주 사이`에서 제목/본문/tail을 어느 단에 남기는지의 공통 flow 문제다.
- stage86 현재 커밋은 안정 대상과 no-separator 20/20/20을 유지한 상태에서, 이 잔여 큰 gap
  visible separator 후보를 다음 분석 대상으로 넘긴다.
