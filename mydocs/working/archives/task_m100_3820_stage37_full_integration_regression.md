---
kind: verification
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 37 — native/HWPX viewport 분리 전체 integration 회귀

## 목적

Stage 36의 native HWP RowBreak viewport 복구가 issue2007의 focused 계약뿐 아니라 전체
integration fixture에 회귀를 만들지 않는지 확인한다. 기준 commit은 `6af881f29`이다.

## 실행 계약

```bash
CARGO_TARGET_DIR=target/task-3820-3821-fidelity \
CARGO_INCREMENTAL=0 \
cargo test --profile release-test --tests
```

전체 integration test와 실물 fixture를 수행하므로 장시간이 정상이다. 출력 공백이나 실행 도구의
응답 분리만으로 종료시키지 않고, 최종 exit code와 test summary를 확인한다.

## 판정 기준

- 전체 test binary가 성공해야 한다.
- Stage 36의 native HWP issue2007, HWPX #3637, deferred pagination #2214/#2424 focused
  검증 결과와 모순되는 failure가 없어야 한다.
- 새 실패가 나오면 이 문서는 실패한 test·로그와 관련 source 범위만 기록하고, 수정은 다음 Stage로
  분리한다.

## 2026-08-07 분석 기록 — 전체 게이트에서 발견된 실제 범위

### 관측 순서와 분리

1. 첫 전체 실행은 `issue_884_charshape_diagnostic`에서 멈췄다. SVG가
   `font-family="&apos;HY수평선B&apos;"`처럼 XML escape한 정상 출력을, test가 escape하지 않은
   문자열만 찾은 것이 원인이었다. 렌더러가 아닌 assertion을 escape 형식 두 가지를 허용하도록
   좁혀 정정했고 focused test가 통과했다.
2. 다음 실행은 `rowbreak-problem-pages.hwpx` p8의 continued nested reference line에서 멈췄다.
   render tree상 이 줄은 현재 쪽 source owner이지만 이전 쪽 절대 y를 유지해 cell top에서 한
   픽셀만 교차하고 전체 glyph가 ancestor clip에 가려졌다. 한컴 2024 PDF 직접 대조로 실제
   결함임을 확인했다. 실제 HWPX 컨테이너의 해당 crossing line만 cell top으로 재배치하는 보정을
   적용했고 `issue_rowbreak_chart_overlap` 20개 test가 통과했다.
3. 이 보정의 원래 게이트인 `hwpx_stored_layout()`에는 rhwp HWPX→HWP 변환 계보도 포함된다.
   HWP5 파일에 HWPX 전용 clipping 보정이 번지는 것을 막기 위해
   `LayoutCompatibilityProfile::hwpx_container()`를 추가했다. 실제 ZIP HWPX만 true이며,
   저장 LINE_SEG 해석 계약(`hwpx_stored_layout`) 자체는 변경하지 않는다.

### #1921 59043 p16 반증 결과

전체 overflow-cell 래칫에서 `issue1921/59043_regulatory_analysis.hwp`가 baseline 41줄에서
108줄로 증가했다. CLI의 per-page `overflowCellLines`로 분해하면 p16이 신규 67줄, p37이 기존
41줄이다. p16 render tree의 분할 표 높이는 1,629.1px로 물리 page body(971.3px)를 크게 넘으며,
이는 "감소한 페이지 수"가 실제 내용 복구를 뜻하지 않음을 보인다.

- 현재 39쪽 후보와 한컴 2022 PDF p16 직접 비교: pixel diff 21.30%.
- 기준 commit `6af881f29`는 41쪽이고 p16 diff 17.44%였지만, p16 표 소유도 PDF와 일치하지
  않는다. 따라서 과거 overflow baseline을 정답으로 올리거나 낮추는 것은 근거가 없다.
- native HWP5 RowBreak 1×1 outer wrapper를 항상 행 기하로 사용해 본 반증에서는 p16의 신규
  overflow가 사라지지만 41쪽으로 회귀해 기존 `issue_1921_59043_pagination_pin`을 실패시켰다.
  이 조건은 즉시 되돌렸다. 해결은 outer/inner wrapper를 전역 선택하는 방식이 아니라, p16의
  실제 fragment owner와 다음 페이지 cut을 동시에 보존하는 별도 Stage의 과제다.

따라서 #1921 p16은 **실제 미해결 결함**이다. 페이지 수 39만을 고정하는 현 test로는 충분하지
않으며, p8 PDF 대조 결과까지 확인한 뒤 p8 또는 p16의 source-owner/overflow가 재발하지 않도록
회귀 assertion을 추가할지 결정한다.

### 이번 Stage에서 통과한 focused 증거

```text
issue_2430_cell_rewrap_threshold: 1 passed
issue_rowbreak_chart_overlap:     20 passed
issue_1921_59043_pagination_pin:  1 passed (39쪽 후보에서 확인)
```

단, 마지막 #1921 pin은 위 p16 결함을 검출하지 못하므로 전체 회귀 성공 근거로 사용하지 않는다.

### 다음 확인 순서

1. 현재 39쪽 후보의 #1921 p8을 `pdf/issue1921/59043_regulatory_analysis-2022.pdf`와 직접
   비교한다.
2. p8에 source-owner/표 fragment 결함이 있으면 PDF가 보이는 불변량을 focused test로 추가한다.
3. p16의 67줄 소실은 다음 Stage의 별도 분석·수정 범위로 이관한다. baseline을 현재값으로 갱신하지
   않는다.
4. 현재 Stage 코드의 범위를 확정한 뒤 `cargo test --profile release-test --tests`를 다시 실행해
   최종 exit code와 summary를 기록한다.

### 전체 integration 재실행 결과

위 실행 계약 그대로 전체 integration test를 끝까지 수행했다. 컴파일 뒤 unit test는
`3,289 passed; 0 failed; 8 ignored`였고, integration binary들은 `overflow_cell_baseline` 전까지
성공했다. 최종 종료 코드는 **101**이며, 실패는 다음 한 test다.

```text
tests/overflow_cell_baseline.rs
overflow_cell_lines_do_not_grow
```

이 gate는 page 밖으로 그려지는 cell line 수가 기준보다 늘면 실패한다. 이번 결과는 baseline을
갱신할 근거가 아니라, 다음 실제 증가를 분리해 조사해야 한다는 신호다.

```text
76076_regulatory_analysis.hwp                 baseline 없음 → 10
86712_regulatory_analysis.hwp                 66 → 91
issue1891/76076_regulatory_analysis.hwpx      baseline 없음 → 10
issue1891/86712_regulatory_analysis.hwpx      66 → 91
issue1921/59043_regulatory_analysis.hwp       41 → 108
issue3637/regulatory_impact_nested_table_escape.hwpx  19 → 23
```

이 중 #1921의 41 → 108은 위 p16에서 이미 분리한 67줄과 일치한다. 나머지 네 fixture는 현재
Stage 36/37 변경 전 clean 기준 commit에서도 이 gate가 통과하는지 먼저 대조하고, 그 결과가
확인되기 전에는 baseline 또는 테스트 기대값을 수정하지 않는다.

### clean 기준 재현 결과

`6af881f29`를 별도 worktree에서 같은 profile/독립 target으로 실행해 위 전제를 검증했다.
그 결과도 exit code **101**로 실패했다.

```text
76076_regulatory_analysis.hwp                 baseline 없음 → 10
86712_regulatory_analysis.hwp                 66 → 91
issue1891/76076_regulatory_analysis.hwpx      baseline 없음 → 10
issue3637/regulatory_impact_nested_table_escape.hwpx  19 → 23
```

즉 이 네 항목은 이번 worktree의 미커밋 코드가 만든 회귀가 아니라, 기준 commit 자체가 현재
fixture·baseline 조합에서 이미 위반하던 **기존 gate 부채**다. 반면 현재 후보에만 추가된
`59043: 41 → 108`은 이번 Stage 변경과 연관된 실제 회귀다. 후속 수정은 이 67줄을 우선
제거하고, 기존 네 항목은 별도 baseline 정비/결함 분석 항목으로 섞지 않는다.

### #1921 p16 fragment-owner 가설

current와 clean 기준의 p16 `dump-pages`/render tree를 비교했다. 현재 후보는 `pi=124`의
1×1 RowBreak wrapper에 대해 pagination이 내부표 row `0..5`와 `end_cut=[5,9]`을 선택한다.
그러나 partial renderer가 `hwp5_origin_hwpx()`까지 native HWP5와 같은 분기로 묶어,
cursor가 outer wrapper의 1행 범위를 이미 벗어났어도 **outer 1×1 전체**를 renderer에
넘긴다. 그 결과 내부 12행 전체가 `h=1,629.1px`로 p16에 paint되어 67줄이 body bottom
밖으로 소실된다.

```text
current p16: PartialTable pi=124, rows 0..5, render height 1,629.1px
clean   p16: PartialTable pi=124, rows 0..1, render height   406.4px
PDF p14: 같은 이해관계자 표가 body 안에서 다음 쪽으로 정상 분할됨
```

`59043`은 HWP5 파일이지만 HWPX-origin 계보다. native HWP5 wrapper의 물리 clip/frame 계약과
동일시하면 안 된다. 수정 가설은 **pagination이 고른 row cursor가 outer domain을 넘을 때
HWPX-origin HWP는 `fragment_row_geometry_table()`의 내부표를 renderer에도 전달한다**는 것이다.
native HWP5에는 기존 outer-wrapper 경로를 유지하므로 issue2007의 native HWP continuation
계약을 바꾸지 않는다. 검증 기준은 (a) #1921 p8의 사진 cell containment 및 39쪽 pin 유지,
(b) p16 `overflowCellLines` 67줄 제거, (c) #2007/#2430/#1939 focused 회귀 통과다.

### #1921 p8 PDF 직접 대조 (추가 요청)

현재 39쪽 후보의 p8을 같은 한컴 2022 PDF와 비교했다. pixel diff는 35.82%였다. 이 수치에는
현재 raster 환경의 한글 fallback(일부 glyph가 tofu로 보임)이 섞여 있어 단독 통과/실패 기준으로
쓰지 않는다. 그러나 글꼴과 독립적인 다음 geometry 결함은 명확하다.

- 좌하단 세로 사진이 자기 표 셀의 하단을 지나 바로 아래 본문 영역까지 paint된다.
- 한컴 PDF에서는 같은 사진과 caption이 표의 마지막 행 안에 완전히 포함된다.

따라서 p8은 page-count pin만으로는 보호되지 않는 **실제 표-cell clipping 회귀**다. 다음 코드
수정 전에 render tree에서 해당 image node와 직접 소유 cell bbox를 식별하고, `image bottom <=
cell bottom + tolerance`를 검증하는 focused test를 추가한다. 글꼴 의존 텍스트 비교는 이 gate에
넣지 않는다.

### #1921 p8 원본 제어 분석과 수정 가설

회귀 게이트 추가 전에 `sections[0].paragraphs[73].controls[0]`의 실제 표(8행, 2열)와 p8의
6행 0열 셀을 조사했다. 문제의 셀에는 text가 비어 있고 첫 `LINE_SEG.vertical_pos`도 `0`이다.
그 하나의 문단에 제어가 순서대로 두 개 들어 있다.

1. 첫 제어는 `bin_data=2`, `treat_as_char=false`, `flow_with_text=true`,
   `text_wrap=Square`, `vert_rel_to=Para`, `vert_align=Top`인 세로 사진이다.
2. 둘째 제어는 `bin_data=3`, `treat_as_char=true`인 인라인 사진이다.

한컴 2022 PDF에서는 두 사진 모두 이 6행 셀 안에 있다. 반면 현재 render tree에서 cell은
`y=479.4..773.3`, Square 사진은 `y=771.4..1060.5`로 나온다. 즉 원본의 음수
`vertical_offset`을 직접 재적용한 문제가 아니라, **첫 빈 문단을 compose한 뒤 advance된
`para_y_before_compose`가 Square 사진의 Para 앵커로 쓰인 것**이 직접 원인이다. 뒤따르는
인라인 사진은 같은 문단의 저장된 첫 line anchor에서 `y≈481`로 배치되어, 두 제어가 서로 다른
원점을 사용했음도 확인했다.

수정 범위는 기존 #4059의 일반 Square 앵커 규칙을 전역 변경하지 않는다. 다음의 실제 HWP5
형태에만 한정한다.

- 셀 문단 text가 비어 있고 첫 `LINE_SEG.vertical_pos == 0`이며,
- 같은 문단에 Para-relative, non-inline, flow-with-text `Square` 그림과
  treat-as-char 그림이 함께 있을 때,
- Square 그림의 Para 앵커를 compose 후의 `para_y`가 아니라 그 셀의 물리 content top
  (`content_cell_y + pad_top`)으로 복원한다.

이 조건은 #4059의 valign=Center인 일반 Square 셀과 #2226의 `vpos > 0`으로 밀려난 빈 줄을
포함하지 않는다. 수정 전에는 `cell_square_picture_anchor` 및 #2226의 focused 회귀를 함께
실행해 범위가 기존 계약에 번지지 않는지 확인한다.

추가로 p8의 표는 보통 표 렌더 경로가 아니라 페이지 분할된 `table_partial` 경로를 탄다는 것을
render tree와 임시 anchor trace로 확인했다. 따라서 일반 `table_layout`에만 둔 분기는 의도대로
실행되지 않았고 p8 좌표도 변하지 않았다. 실제 수정은 동일한 원본 조건을 `table_partial`의
non-inline Picture anchor에 적용하고, 일반 경로의 진단성 임시 분기는 제거한다. 이는 같은
의미의 셀 배치 규칙이 full/partial 구현에 중복되어 있는 현재 구조를 고려한 범위 한정이다.

수정 뒤 p8 gate는 다음 조건으로 통과했다.

```text
issue_1921_59043_pagination_pin:              2 passed
cell_square_picture_anchor (#4059):           1 passed
issue_2226_cell_flow_pictures_overlap:        1 passed
issue_2430_cell_rewrap_threshold:             1 passed
issue_rowbreak_chart_overlap:                  20 passed
issue_2007_nested_cell_pagination:              9 passed
issue_2097_band_fill:                           1 passed
issue_1939:                                     1 passed
```

직접 PDF 재대조(`fidelity_compare`, p8만)에서 pixel diff는 35.82% → 32.25%로 감소했다.
현재 산출물의 세로 사진은 셀 안에 들어가며, 남은 raster 차이의 대부분은 이 환경에서의 한글
glyph fallback으로 보인다. 따라서 수치만으로 전체 정합을 선언하지 않고, `image bottom <= cell
bottom + 0.75px`의 글꼴 독립 gate와 비교 시트를 함께 증거로 보관한다.
