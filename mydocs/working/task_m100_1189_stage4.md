# Task M100 #1189 Stage 4

## 목적

Stage3 커밋 이후 `3-10월_교육_통합_2022.hwp` 17쪽 미주 영역에서 한컴오피스 기준 드래그 선택이 되지 않는 문제를 보정한다.

## 시작 기준

- 이슈: [#1189](https://github.com/edwardkim/rhwp/issues/1189)
- 작업 브랜치: `local/task_m100_1189`
- 선행 커밋: `decee292` (`task 1189: 11쪽 미주 제목 간격 보정`)
- 대상 문서: `samples/3-10월_교육_통합_2022.hwp`
- 대상 페이지: 17쪽
- 사용자 시각 판정: 한컴오피스에서는 17쪽 하단 미주 `문27)` 부근을 드래그 선택할 수 있으나 rhwp 기준 드래그 선택 동작이 맞지 않는다.
- 작업지시자 승인: 자동 승인. Stage4 문서 생성 후 분석/수정/검증/커밋까지 진행한다.

## 초기 판단

1. 화면상 대상은 본문이 아니라 페이지 하단 미주 흐름이다.
2. 드래그 선택은 `rhwp-studio`의 mouse hit-test와 Rust `get_selection_rects*` 계열 API가 함께 관여한다.
3. Stage31 이후 미주 편집/선택용 가상 문단 인덱스가 도입되었으므로, 17쪽의 미주 가상 문단 좌표가 hit-test 또는 selection rect 계산에서 누락되는지 먼저 확인한다.

## 진행 계획

1. `3-10월_교육_통합_2022.hwp` 17쪽의 미주 문단 배치와 가상 문단 인덱스를 덤프한다.
2. 대상 좌표에서 `hitTest`, `hitTestInFootnote`, `getSelectionRects`, `getSelectionRectsInFootnote` 계열 동작을 재현한다.
3. 한컴처럼 같은 페이지 미주 문단을 드래그 선택할 수 있도록 가장 좁은 경로를 수정한다.
4. 전용 회귀 테스트와 `issue_1139_inline_picture_duplicate` 단일 테스트를 수행하고, 자동 승인에 따라 커밋한다.

## 현재 상태

- 2026-06-01: 작업지시자가 한컴오피스 기준 드래그 선택 불일치를 보고하고 Stage4 자동 승인을 지시했다.
- 2026-06-01: `dump-pages` 기준 대상은 `3-10월_교육_통합_2022.hwp` 17쪽 좌측 단 미주 `문27)` 흐름(`pi=915..921`)으로 확인했다.
- 2026-06-01: `get_selection_rects(0, 915, 0, 921, 3)` 진단 결과 수정 전에는 `pi=915`, `pi=916`, `pi=921` 3줄만 selection rect가 생성되고, 수식 컨트롤이 줄 끝에 섞인 `pi=917..920` 4줄이 빠졌다.
- 2026-06-01: 원인은 선택 끝 커서를 TextRun 기준으로만 찾는 경로였다. 수식 꼬리 때문에 `range_end`와 `range_end - 1`이 TextRun에 걸리지 않으면 해당 줄 전체 rect 생성이 생략됐다.
- 2026-06-01: 본문/미주 가상 문단 선택에서 trailing TextRun 커서를 못 찾는 경우 같은 `TextLine`의 오른쪽 끝을 fallback cursor로 사용하도록 보정했다. 셀 내부 선택 경로는 기존 동작을 유지했다.

## 검증 기록

- `cargo test --test issue_1139_inline_picture_duplicate issue_1189_2022_oct_page17_endnote_drag_selection_covers_equation_tail_lines -- --nocapture` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(32 passed).
- `cargo fmt --all --check` 통과.
- `git diff --check` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
- `cargo test --tests` 통과.
