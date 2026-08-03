---
kind: investigation
status: active
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 6 — HWPX 그림 23 next-vpos rewind `RowBreak` 분석

- 선행 commit: `a006a4ebf` (`fix: #3738 HWPX 저장 anchor 이월 정합`)
- 기준 자료: 개인정보 제거 원본 HWP·HWPX와 각각의 한컴오피스 2020 PDF
  ([경로·SHA-256·Git/LFS 판정](../../pdf/pr3740/README.md))
- 직전 증적: [Stage 5 visual sweep](task_m100_3738_stage5_visual_sweep.md)

## 잔여 결함의 관측값

Stage 5는 HWPX p273(그림 11)의 **증가하는** 다음 저장 vpos를 원래 page anchor로 써서 최초
page drift를 해소했다. 그러나 그림 23의 host p344는 다른 경계다. 원본 HWP/HWPX의 이 구조는 다음과
같다.

```text
outer p344: first LINE_SEG.vpos = 52230 HU
  1×1, TopAndBottom / Para / RowBreak table
  vertical offset = 560 HU, cell height = 36782 HU
  non-inline picture vertical offset = -52790 HU, height = 24791 HU
  Bottom caption
next p345: first LINE_SEG.vpos = 30689 HU  (host보다 작음 = 새 물리 페이지 anchor)
```

HWPX Stage 5 render tree는 이 표를 renderer index 22(문서 23쪽) `y=548.9px`에 예약하고,
내부 image를 `y=276.9px`에 둔다. image가 자기 표 위로 이탈하므로 p23 내용과 충돌하며, index 23
(문서 24쪽)에는 기준 PDF의 그림 23 graph/caption이 없다. 같은 p344를 native HWP 경로는 index 23에
배치하고 image `y=92.5px`부터 렌더한다.

## 코드 경계와 가설

`typeset_block_table_inner`의 `rewound_empty_figure_float_should_defer`는 위와 같은 빈 1×1
`TopAndBottom`/`RowBreak` 그림 표, 다음 vpos 되감기, fresh page에 표 전체가 들어가는 조건을 정확히
판별한다. 현재는 `native_hwp5_layout()`만 허용해 HWPX가 일반 1×1 force-split 경로로 떨어진다. 이것이
Stage 5 p344의 table/image 분리와 맞는다.

HWP가 fresh page로 이월된 뒤 쓰는 두 후속 보정도 같은 profile gate다.

1. `layout/table_layout.rs`의
   `native_hwp5_relocated_empty_rowbreak_picture_resets_offset`은
   `52230 + 560 - 52790 = 0 HU`인 page-boundary 상쇄를 page-local picture top으로 정규화한다.
2. `layout.rs`의 `native_hwp5_relocated_empty_rowbreak_picture_next_flow_top`은 Bottom caption 뒤
   다음 p345의 저장 anchor에서 flow cursor를 재개한다.

두 입력은 이 p344 주변의 저장 geometry가 같지만, Stage 5의 p273 수정은 **next vpos가 증가할 때만**
적용되므로 이 rewind 형상에는 영향을 주지 않는다. 따라서 다음 코드는 단순히 모든 native-HWP5 보정을
HWPX에 확장하지 않고, 위 세 함수의 이미 엄격한 형상·경계 합·caption 조건을 보존한 채 original HWPX
stored-layout까지 허용할 수 있는지 확인하는 범위로 제한한다.

## 구현과 검증 결과

세 gate를 기존의 형상·경계 합·caption 조건은 그대로 둔 채 `native HWP5 또는 original HWPX
stored-layout`으로 확장했다. helper 이름도 더 이상 HWP5 전용으로 오해되지 않도록 `stored_layout_*`로
정리했다. focused unit regression
`cargo test stored_layout_relocated_empty_rowbreak_picture --lib --quiet`와 release-test binary build를
완료했다.

HWPX 기준 PDF와의 144 DPI sweep은 p23–p24(그림 23) 2/2 및 p13–p15(앞선 그림 11 회귀) 3/3 페이지를
완료했다. p13–p15에는 새 visual flag가 없고, p344 table은 예상대로 renderer index 23(문서 24쪽),
table top `y=90.6px`로 이동했다. 23쪽에는 더 이상 p344 table이 없다.

하지만 결과를 전체 해결로 해석할 수 없다. p24 render tree에서 p344의 내부 image bbox는 여전히
`y=-181.4px`이고 caption은 `y=160.5px`에 남는다. 기준 PDF의 그림 23 full graph는 page-local top에서
보여야 하나 rhwp 출력은 음수 영역이 clip된 일부만 보인다. 뒤따르는 p345 flow는 `y=492.3px`에 재개된다.
p24의 `question_marker_flow_drift` 자동 후보와 review PNG도 이 차이를 기록한다.

따라서 Stage 6은 **표 page ownership만 부분 해소**했다. HWPX에서 offset-reset helper가 아직 적용되지
않는 정확한 predicate/입력 차이는 다음 Stage의 새 분석 대상이다. 원본 HWP·HWPX·기준 PDF와 이 회차 review
PNG는 모두 계속 보관한다.
