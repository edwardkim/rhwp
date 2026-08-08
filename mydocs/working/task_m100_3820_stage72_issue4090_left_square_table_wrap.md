---
kind: analysis
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 72 — issue4090 우측 Square 표의 좌측 본문 wrap 복원

## 출발점

Stage 71의 전체 17쪽 PDF 대조는 `samples/issue4090/156492236_규제샌드박스_min.hwpx`에서
p5, p7, p15, p17의 우측 점선 placeholder 표와 나란히 있어야 할 본문이 rhwp 출력에서
사라지는 결함을 확정했다. Stage 65의 `review_144.png`처럼 보정 전 증적을 최종 결과로
사용하지 않으며, 이 Stage는 issue4090의 별도 `Square` 표 계약만 다룬다.

## PDF와 HWPX의 계약

대표 p5의 anchor는 global `pi=44`의 1×1 비인라인 표다.

- `textWrap="SQUARE"`, `textFlow="BOTH_SIDES"`, `treatAsChar="0"`
- 문단 기준 우측 위치: `horzRelTo="PARA"`, `horzOffset=26319`
- 표 폭 `21022 HU`, 높이 `13241 HU`
- 바로 다음 본문 `pi=45`의 저장 `LINE_SEG`는 처음 6줄에
  `column_start=0`, `segment_width=26319 HU`를 기록하고, 마지막 1줄만 전폭
  `segment_width=48188 HU`로 돌아온다.

즉 PDF의 p5는 표 왼쪽 띠에 6줄을 그리고, 표 아래에서 같은 문단의 마지막 줄을 전폭으로
계속한다. p7/p15/p17도 같은 right-side Square-table + left-strip + full-width tail 형상이다.
이는 단순 reflow 선택이나 font raster 차이가 아니라, HWPX가 명시한 줄 영역 계약이다.

## 현재 경로와 결함 위치

현재 release-test `dump-pages`의 p5는 `pi=45`를 `startLine=6, endLine=7`로 일반
`PartialParagraph`에 남긴다. 따라서 마지막 전폭 tail은 정상적으로 배치된다. 하지만 p5
render tree에는 그 앞 6줄의 `TextLine`이 전혀 없고, placeholder 표만 그려진다.

`typeset_wrap_around_paragraph()`은 이 형상에서 `WrapAroundPara { start_line: 0,
end_line: 6 }`를 기록하고 tail만 `PartialParagraph`로 남기도록 설계되어 있다.
`layout_wrap_around_paras()`는 이를 `strip_area`에 전달한다. 그러나
`layout_partial_paragraph()`는 기존 전폭 `ComposedParagraph`의 줄을 재사용할 수 있어,
저장 `LINE_SEG`가 요구하는 26319 HU 좌측 띠 폭으로 앞 6줄을 별도 compose/paint한다는
보장이 없다. 결과적으로 suffix는 존재하지만 wrap prefix가 render tree에서 소실된다.

## 보정 범위

1. 우측 non-TAC `Square` 표의 empty host에서만, `WrapAroundPara`가 요구하는 strip 폭을
   레이아웃까지 명시적으로 전달한다.
2. wrap prefix는 해당 strip 폭으로 compose하거나 저장된 line-segment 줄 경계와 일치하는
   별도 composed view를 사용해 paint한다. 전폭 tail은 현재처럼 일반 `PartialParagraph`로
   남긴다.
3. 다른 그림 wrap, 좌측 표 wrap, text host 표, 그리고 p6/p8/p16의 explicit tail page에는
   적용하지 않는다.
4. 자동 sweep의 overlap/overflow 지표가 이 **wrap exclusion**을 놓친 사실은 별도
   automation 후보로 남긴다. 이번 renderer 보정에 무리하게 결합하지 않는다.

## 회귀 계약과 검증 계획

- 새 focused regression: p5 render tree에 `pi=45`의 prefix text run이 존재하고 그 우측이
  `pi=44` 표의 좌측선 밖을 침범하지 않음을 확인한다. 동시에 마지막 tail text run은 표 아래
  전폭에 남아야 한다.
- 기존 `tests/issue_4090_hwpx_tail_page_break.rs`의 17쪽 및 p5→6, p7→8, p15→16 tail
  계약을 유지한다.
- 수정 뒤 focused test, `issue_1891` 중첩 표 regression, overflow-cell baseline을 실행하고
  p1--17 visual sweep을 다시 PDF와 직접 비교한다.

다음 코드는 위 contract 밖의 전역 재조판을 하지 않는다. 줄 수·page count가 달라지거나
tail이 표 옆으로 되돌아가면 보정을 중단하고 원인을 다시 분리한다.
