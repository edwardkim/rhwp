---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-04
---

# Task #3820 Stage 9 — p127 deferred Square 그림 56의 page-top 좌표 분석

## 재현

Stage 8 뒤에도 사용자 p127에서 그림 56(`pi=1355`, `ci=0`)의 세로 위치가 한컴 2020 PDF보다
아래로 밀린다. 그림 옆 `pi=1356` narrow text의 horizontal wrap 폭은 source `sw=23057 HU`와
일치하고 그림과 교차하지 않는다. 따라서 이는 horizontal outer-margin 결함이 아니라, 다음
physical page로 이월된 Square picture가 원 anchor의 positive vertical offset을 다시 적용하는
**page-top geometry 결함**이다.

현재 render tree는 body top `y=83.2px`에서 그림 bbox를 `y=130.7px`으로 만든다. 차이
`47.5px`는 source `pi=1355`의 `vertical_offset=3566 HU`와 같다. 기준 PDF는 page-top owner로
이월된 그림 frame을 body top에서 시작한다. visible image content의 internal white padding 때문에
first ink는 body top보다 조금 낮지만, object frame은 positive anchor offset을 다시 소비하지 않는다.

## source 계약과 범위

`pi=1355`는 p126 tail의 visible anchor text + `Square`, non-TAC, `vert=Para`, bottom caption 그림이다.
뒤 `pi=1356`은 첫 line부터 `vpos=0`, `cs=0`, `sw=23057 HU`의 narrow band이고, #3821 Stage의
typeset path가 그림만 p127 시작으로 defer한다. p156 그림 64(`pi=1692`)와 달리 그 다음 문단은
full-width lines 뒤에 reset되는 형상이므로 원 anchor offset을 page-top position에 유지해야 하는
경우가 아니다.

보정은 layout의 deferred-page-start item에만 적용한다. item이 column 첫 `Shape`이고, 같은 column의
wrap anchor가 그 shape para를 가리키며, source host `FullParagraph`/`PartialParagraph`가 이 page에
없고, successor narrow band가 첫 `vpos=0`으로 시작하는 native deferred Square contract일 때만
`para_y`를 `body_top - vertical_offset`으로 조정한다. 기존 object-position 함수는 offset을 한 번
적용해 최종 frame을 body top에 둔다.

이 경계는 일반 Square 그림, p156처럼 full-width tail 뒤 reset되는 next-page owner, same-page float,
TopAndBottom/inline 그림, 표와 각주에는 적용하지 않는다.

## 수용 기준

1. p127 그림 56 bbox top은 body top과 일치하고, `pi=1356` narrow text는 `sw=23057 HU` wrap band를
   유지하며 그림과 겹치지 않는다.
2. p126에는 그림 56이 나타나지 않고 p127에는 정확히 한 번 나타난다.
3. p156 그림 64와 p118→p119 Stage 8 owner regression을 포함한 focused fixture tests를 유지한다.
4. p127 PDF 3-way review와 render-tree geometry를 증적으로 남긴다.

## 구현과 결과

`src/renderer/layout.rs`는 direct non-TAC picture layout에서 위 deferred contract를 다시 판별한다.
`layout_body_picture`가 paragraph-relative `vertical_offset`을 정확히 한 번 적용하도록 source offset을
상쇄했으며, 일반 Square/inline/TopAndBottom 그림에는 이 경로가 열리지 않는다.

render tree의 p127 `pi=1355/ci=0` bbox는 수정 전 `y=130.7px`에서 body top과 같은 `y=83.2px`로
바뀌었다. p127의 `pi=1356`은 같은 top부터 `sw=23057 HU` narrow wrap band를 유지하고, p156 그림
64는 기존 full-width-tail contract의 `y=90.1px`을 유지한다.

자동 판정도 함께 보강했다. `tools/fidelity_compare/fidelity_compare.py`의
`deferred_square_picture_page_top_drift_candidates`는 column 첫 Square image가 body top에서 20px 이상
아래에 있고, 같은 top의 side-wrap text가 존재하는 이전 p127 형상을 후보로 기록한다. visual sweep은
동일 detector를 `deferred_square_picture_top_drift` flag로 사용한다. 따라서 수정 후 p127이 flag 0인 것은
정상이며, 수정 전의 geometry는 Python fixture 회귀에서 명시적으로 flag가 된다.

## 검증

```text
CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment
# 20 passed; 0 failed

python3 -m py_compile tools/fidelity_compare/fidelity_compare.py scripts/visual_sweep.py
python3 -m unittest scripts/tests/test_fidelity_compare.py scripts/tests/test_visual_sweep.py
# Ran 59 tests ... OK
```

선택 3-way PDF 검증과 검토 PNG의 직접 판정은
[Stage 9 visual sweep](task_m100_3820_stage9_visual_sweep.md)에 남긴다. #3820의 p127은 이 stage에서
해소했지만, 기준 PDF 대비 전체 rhwp page count가 218/215인 D-03 연쇄 pagination divergence는 별도
후속 stage에서 계속 분석한다.
