# Task M100 #1209 Stage 8

## 목적

PR 준비용 전체 테스트에서 발견된 `issue_1082_endnote_multicolumn_drift` 회귀를 Stage7 이후 별도 단계로 분리해 보정한다.
Stage6의 compact 미주 VPOS 되감김 보정이 `3-11월_실전_통합_2022.hwp` 1쪽 미주/본문 흐름에 과도하게 적용되어 페이지 하단 오버플로우를 만든 것인지 확인한다.

## 시작 기준

- 이슈: [#1209](https://github.com/edwardkim/rhwp/issues/1209)
- 작업 브랜치: `local/task_m100_1209`
- 선행 커밋: `a436af00` (`task 1209: Stage7 모달 드래그 공통화`)
- 요청 일시: 2026-06-02

## 발견 경로

- Stage7 커밋 후 PR 준비 단계에서 `cargo test --tests` 실행.
- `tests/issue_1082_endnote_multicolumn_drift.rs`의 `exam_3_11_2022_hwp_endnote_drift_capped`가 실패.
- 같은 테스트를 별도 worktree의 `upstream/devel` 기준에서 실행하면 통과하므로 Task #1209 브랜치 회귀로 판단한다.

## 현재 관찰

- 현재 브랜치 page 1 SVG: `height=1122.5`, `maxY=1377.2`, overflow 약 `254.7px`.
- `upstream/devel` page 1 SVG: `height=1122.5`, `maxY=965.9`, overflow 없음.
- `dump-pages` 항목 배치는 현재/업스트림이 거의 동일하여 pagination보다 render-stage y 보정 쪽을 우선 확인한다.
- `RHWP_VPOS_DEBUG=1` 기준 page 1 `pi=69` 빈 문단에서 `FullParagraph`가 `y_after=1201.6`까지 튀며 오버플로우를 만든다.
- 해당 문단의 그림은 기존 #959 진단처럼 `pic_emit_x=767 > col_right=759`로 현재 단 오른쪽 바깥에 있다. 따라서 그림 자체는 현재 단 흐름을 밀면 안 된다.

## 원인

Stage5에서 문단 기준 `TopAndBottom` 개체가 있는 문단의 `LINE_SEG.vertical_pos`를 한컴 저장 좌표로 반영하도록 보정했지만, 개체의 가로 범위가 현재 단과 교차하는지 확인하지 않았다.

`3-11월_실전_통합_2022.hwp` 1쪽의 `pi=69`는 문단 기준 `TopAndBottom` 그림이 있으나 실제 배치 x 좌표가 현재 단 밖이다. 이 경우 #959에서 이미 그림 advance를 건너뛰는 구조였는데, Stage5의 문단 vpos 보정 경로만 같은 가로 교차 조건을 적용하지 않아 빈 문단이 페이지 하단 밖으로 이동했다.

## 보정

- 문단 기준 `TopAndBottom` 그림/도형이 현재 단 흐름에 영향을 주는지 판단하는 공통 가로 교차 검사를 추가했다.
- `HorzRelTo::Column`/`Para`는 개체 폭, 정렬, 가로 오프셋으로 현재 단과의 교차 여부를 계산한다.
- `Paper`/`Page` 기준 개체는 단 밖 판단이 불명확하므로 기존처럼 흐름 영향 가능성을 유지한다.
- 문단 `LINE_SEG.vertical_pos` 보정은 현재 단과 가로 범위가 교차하는 문단 기준 `TopAndBottom` 개체가 있을 때만 적용한다.

## 검증

- `cargo test --test issue_1082_endnote_multicolumn_drift exam_3_11_2022_hwp_endnote_drift_capped -- --nocapture` 통과.
- `cargo fmt --all --check` 통과.
- `cargo test --test issue_1082_endnote_multicolumn_drift -- --nocapture` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과.
- `cargo run --quiet --bin rhwp -- export-svg samples/3-11월_실전_통합_2022.hwp -p 0 -o output/diag_1209_stage8_fixed` 후 SVG 측정 결과 `height=1122.5`, `maxY=965.9`, overflow `0.0` 확인.

## 남은 작업

- PR 준비용 전체 테스트, WASM 빌드, Studio 빌드를 Stage8 커밋 후 재실행한다.
