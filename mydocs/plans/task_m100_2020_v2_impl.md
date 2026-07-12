# Task M100-2020 v2 구현 계획

작성일: 2026-07-08
대상 이슈: #2020
작업 브랜치: `task/m100-2020-remaining-visual`

## 1. 착수 상태

- #2020 assignee 를 `jangster77` 로 지정했다.
- 열린 PR 없음 확인.
- `devel` 과 `upstream/devel` 동기화 상태에서 작업 브랜치를 만들었다.
- 작업지시자의 "계속 진행" 지시에 따라 계획 문서 작성 후 Stage 1 분석과 구현을 진행한다.

## 2. Stage 1 세부 작업 — 복학원서

1. `samples/복학원서.hwp` 의 p1 render tree 와 원본 dump 를 비교한다.
2. `pi=16` 하단 접수증 영역의 PUA 선 문자, table fragment, InFrontOfText 타원 배치 순서를 확인한다.
3. `src/renderer/typeset.rs`, `src/renderer/render_tree.rs`, `src/renderer/svg.rs` 에서 table/shape/text emit 순서를 추적한다.
4. 코드 결함이면 좁은 가드로 수정하고 `tests/issue_2020.rs` 에 복학원서 배치 회귀 테스트를 추가한다.
5. visual sweep 으로 p1 `line_order_overlap` 변화와 원형/도형 위치 변화를 확인한다.

## 3. Stage 2 세부 작업 — 여권신청서

1. `samples/issue2020/passport_application_lawgo.hwp` 의 `pi=0` 35x13 RowBreak/TAC 표 구조를 분석한다.
2. HWP 2022 기준 PDF 와 공식 law.go.kr PDF 의 차이를 분리해, HWP 원본 기준으로 무엇을 맞출지 기록한다.
3. 하단 용지규격 문구 overflow 후보와 RowBreak 표 내부 셀/LineSeg drift 후보를 나눠 테스트한다.

## 4. Stage 3 세부 작업 — FSC 폰트/자간

1. FSC p1 은 자동 플래그 0이므로 layout blocker 와 font fidelity 문제를 분리한다.
2. 저장소 font alias/metrics 에서 금융위 보도자료에 쓰인 글꼴 mapping 이 누락됐는지 확인한다.
3. 한컴 전용 폰트 부재로만 설명되는 pixel-level font fidelity 는 #2020 close 판단에서 분리하고, 환경 의존 선택 검증으로 명시한다.

## 5. 커밋 구성

권장 커밋:

1. `task 2020: 복학원서 도형 배치 회귀 가드 추가`
2. `task 2020: 복학원서 도형/표 렌더 순서 보정`
3. 필요 시 `task 2020: 여권신청서 표 흐름 회귀 가드 보강`

단, 실제 코드 수정이 하나의 좁은 변경으로 끝나면 하나의 `task 2020:` 커밋으로 합친다.

## 6. 구현 결과

- Stage 1: 복학원서 `line_order_overlap` 은 U+F081C TAC filler 전용 render-tree 라인이 만든 visual sweep 오탐으로 확인했다. 렌더 정책은 기존 #937 처럼 filler 미출력을 유지하고, sweep 수집기에서 보이는 텍스트가 없는 filler 라인을 제외했다.
- Stage 2: 여권신청서 낫표 `「」` glyph advance 를 반각으로 측정/렌더하도록 보정했다. `tests/issue_2020.rs` 에 `2.「여권법」제9조` 줄 간격 회귀 테스트를 추가했다.
- Stage 2 보조: 하단 `210mm×297mm[...]` 용지 규격 footer 와 `-2-` 페이지 번호 footer 는 기준 PDF 와 같은 하단 footer bleed 인 경우 tail/frame overflow 플래그에서 제외했다.
- Stage 3: FSC 1/2쪽 자동 구조 플래그는 0으로 정리됐다. 기준 PDF 의 embedded font 가 DejaVu 계열이라 pixel-level font fidelity 는 별도 선택 검증 축으로 남기되, 한컴 전용 폰트가 없는 공개 기본 검증에서는 #2020 close blocker 로 보지 않는다.
- 검증:
  - `cargo fmt --check`
  - `git diff --check`
  - `python3 -m py_compile scripts/task1274_visual_sweep.py`
  - `CARGO_INCREMENTAL=0 cargo test --test issue_2020 -- --nocapture`
  - `CARGO_INCREMENTAL=0 cargo test --lib corner_quote -- --nocapture`
  - `CARGO_INCREMENTAL=0 cargo test --lib test_630_middle_dot_full_width_in_registered_font -- --nocapture`
  - `CARGO_INCREMENTAL=0 cargo test --test issue_630 -- --nocapture`
  - `CARGO_INCREMENTAL=0 cargo clippy --release --lib -- -D warnings`
  - `CARGO_INCREMENTAL=0 cargo build --bin rhwp`
  - `wasm-pack build --target web --out-dir pkg`
