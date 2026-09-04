---
kind: analysis
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 38 — #1921 fragment cursor와 renderer 소유 표 일치

## 출발점

Stage 37의 전체 integration 실행은 중단하지 않고 최종 exit code까지 대기했다. unit test는
`3,289 passed; 0 failed; 8 ignored`였지만, `overflow_cell_baseline`가
`issue1921/59043_regulatory_analysis.hwp: 41 -> 108`로 실패했다. 같은 명령을 기준 commit
`6af881f29`의 별도 worktree에서 재현했을 때에는 이 항목이 없었다. 따라서 baseline 갱신이나
기존 게이트 완화가 아니라, 현재 미커밋 변경에서 생긴 #1921 회귀를 먼저 해소한다.

## 관측 사실

- 원본은 실제 HWP5 컨테이너이며 `native_hwp5_layout()` 경로를 사용한다. `hwp5_origin_hwpx()`
  가설은 이 fixture에 적용되지 않는다.
- p16의 PageItem은 outer control `pi=124`를 가리키지만 pagination cursor는 nested content table의
  row `0..5`, `end_cut=[5,9]`을 가리킨다. outer table은 투명한 1×1 RowBreak wrapper이고 내부는
  12×3 table이다.
- 현재 typesetter는 이 형태에서 내부 12×3 table을 row geometry로 선택한다. 그러나
  `layout_partial_table`은 native HWP5라는 이유만으로 outer 1×1 wrapper 전체를 renderer에 넘긴다.
  cursor domain(내부 12행)과 renderer domain(outer 1행)이 달라진다.
- 그 결과 p16 renderer tree에는 `h=1,629.1px` table이 생겨 body 밖 67줄을 paint 후보로 만든다.
  이는 PDF p16의 정상적인 표 조각과 다르다.

## 반증한 접근

`layout_partial_table`의 native/HWPX-origin 분기만 바꾸어 HWPX-origin에 inner table을 넘기는
실험은 #1921 출력에 아무 변화가 없었다. 이 파일은 native HWP5이므로 그 가설은 적용 대상이
아니다. 해당 가설을 해결책으로 기록하거나 commit하지 않는다.

## 수정 가설과 범위

renderer가 outer wrapper를 유지할 수 있는 것은 **partial cursor가 outer row domain 안에 있을
때뿐**이다. `end_row >= outer_table.row_count`이면 이미 cursor가 inner content table의 행을
가리키므로 `fragment_row_geometry_table(outer_table, end_row)`를 renderer에도 사용해야 한다.

이 조건은 다음을 보존한다.

- issue2007의 native HWP continuation처럼 cursor가 outer 1행을 실제로 가리키는 physical frame은
  기존 outer-wrapper clip/frame 경로를 유지한다.
- #1921 p8의 Square 그림 앵커 수정과 무관하다. p8 cell containment gate는 그대로 유지한다.
- typesetter의 pagination 선택 자체를 다시 전역적으로 outer geometry로 되돌리지 않는다. 그 방법은
  #1921 페이지 수를 39에서 41로 회귀시킨 선행 반증이 있다.

## 검증 순서

1. 위 renderer-domain 조건만 적용한다.
2. `tests/issue_1921_59043_pagination_pin.rs`를 먼저 실행해 p8 containment 및 페이지 수 pin을
   확인한다.
3. 사용자 요청 순서대로 `tests/issue_2430_cell_rewrap_threshold.rs`,
   `tests/issue_2007_nested_cell_pagination.rs`, `tests/issue_1939.rs`를 실행한다.
4. CLI render tree와 한컴 2022 PDF를 p16에서 다시 비교한다. `overflowCellLines`의 p16 신규 67줄이
   없어져야 한다.
5. `overflow_cell_baseline`을 재실행한다. 기준 commit에도 있는 네 baseline debt와 신규 #1921 항목을
   분리해 기록한다. 전체 `--tests` 재실행은 수정 결과가 focused gate를 통과한 뒤에만 수행한다.

## 판정

현재는 분석 단계다. PDF와 page boundary가 일치하고 #1921 신규 overflow가 사라지기 전에는
"해결" 또는 baseline 갱신을 하지 않는다.

## 1차 수정 결과

`layout_partial_table`에서 profile만으로 outer wrapper를 강제하는 분기를 제거하고,
`fragment_row_geometry_table(outer_table, end_row)`로 cursor domain을 renderer에도 적용했다.
이는 `end_row <= outer.row_count`일 때 outer를 그대로 보존한다.

수정 직후 #1921 focused gate는 2개 모두 통과했다. 이어서 overflow gate를 최종 종료까지
실행한 결과는 exit code `101`이지만, 실패 목록에서
`issue1921/59043_regulatory_analysis.hwp — 41 -> 108`은 사라졌다. 남은 목록은
clean 기준 `6af881f29`에서도 재현된 기존 baseline debt 다섯 항목뿐이다.

```text
76076_regulatory_analysis.hwp                         baseline 없음 -> 10
86712_regulatory_analysis.hwp                         66 -> 91
issue1891/76076_regulatory_analysis.hwpx              baseline 없음 -> 10
issue1891/86712_regulatory_analysis.hwpx              66 -> 91
issue3637/regulatory_impact_nested_table_escape.hwpx  19 -> 23
```

이는 #1921 회귀 해소의 필요조건만 충족한 것이다. 다음 focused 회귀와 p16 PDF 직접 대조가
통과하기 전까지 Stage 38을 완료로 표시하지 않는다.

## focused 재검증

다음은 수정 뒤 모두 통과했다.

```text
issue_1921_59043_pagination_pin:       2 passed
issue_2430_cell_rewrap_threshold:      1 passed
issue_2007_nested_cell_pagination:     9 passed
issue_1939:                            1 passed
```

따라서 native HWP cursor가 outer row domain 안에 있는 issue2007의 physical continuation
frame 계약은 이 수정으로 깨지지 않았다.

## PDF 직접 대조 결과 — p8과 p16을 분리해 판정

한컴 2022 PDF와 `fidelity_compare`로 직접 비교했다.

- p8: pixel diff `35.82% -> 32.25%`. PDF와 SVG의 text multiset은 `0 / 0`으로 같고, 두
  사진이 모두 6행 왼쪽 cell 안에 있다. 남은 raster 차이는 이 환경의 한글 glyph fallback이므로
  p8에는 새 image-bottom/cell-bottom gate가 적합하다.
- p16: p16 overflow는 `0`이 됐지만 pixel diff는 `19.57%`이며, PDF p16의
  `③ 대안의 선택 및 근거`가 아니라 rhwp p16이 앞선 `① 규제대안의 비교 / ② 의견수렴` 표를
  보유한다. 즉 숨은 paint를 없앤 것과 **페이지 owner를 PDF에 맞춘 것**은 별개다.

PDF p13은 current p15의 첫 본문과, PDF p16은 current p18의 `③ 대안의 선택 및 근거`와 각각
대응한다. p16의 임시 67줄 overflow는 해소됐지만, 이 구간은 current가 PDF보다 두 쪽 늦다.
이 소유 지연의 첫 발생 지점은 p98 table의 p11--p14 분할과 그 이전 경계까지 별도 Stage에서
추적해야 한다. 이 문서의 수정은 안전하게 유지하되, PDF fidelity 전체 해결로 표기하지 않는다.
