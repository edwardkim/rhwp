# Stage 3 완료 보고서 — Task #986

## 범위

Stage 2 에서 pagination 에 적용한 가로 lane 판단을 실제 render tree layout
경로에도 연결했다. 목적은 `PageItem::Table` 이 page 1 에 배치되더라도
`layout_table_item` 이 전역 `y_offset` 을 다음 표의 시작점으로 넘겨 오른쪽 표를
다시 아래로 밀어내는 문제를 막는 것이다.

## 변경 파일

- `src/renderer/layout.rs`
  - `build_single_column` 단위로 문단별 `FloatLaneSet` 상태 추가
  - `layout_column_item` → `layout_table_item` 호출 체인에 lane 상태 전달
  - 텍스트 없는 호스트 문단의 `TopAndBottom + vert=Para + non-TAC` 표를
    `FloatLaneSet` 으로 배치
  - 같은 문단의 표라도 가로 범위가 겹치지 않으면 같은 세로 시작점 사용
  - 겹치는 표만 해당 lane bottom 아래로 push
  - empty para-float 표는 pagination 과 동일하게 host `spacing_before` 를 제외
  - lane 범위 계산 시 문단 margin/indent 를 반영하도록 `typeset.rs` 와 정합
- `src/renderer/typeset.rs`
  - Stage 2 helper 의 horizontal range 계산에 문단 margin/indent 를 반영해
    layout 의 lane overlap 판정과 맞춤

## 검증

```bash
cargo fmt --all --check
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --lib float_placement
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo run --release --bin rhwp -- dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
target/release/rhwp export-svg /private/tmp/rhwp-issue-986/receipt.hwp --debug-overlay -o /private/tmp/rhwp-issue-986-svg
rg -n "s0:pi=0 ci=(2|3|4|5|6|7|8)" /private/tmp/rhwp-issue-986-svg/receipt_001.svg
```

결과:

- `cargo fmt --all --check`: 통과
- `cargo test --lib float_placement`: 6개 테스트 통과
- 기본 `dump-pages`: 2페이지, page 1 에 `ci=2..8` 전체 표가 `Table` 로 배치,
  `PartialTable` 없음
- SVG debug overlay:
  - `ci=2`, `ci=4`, `ci=6`: `y=18.9`
  - `ci=3`: `y=639.0` (왼쪽 lane 에서 `ci=2` 아래)
  - `ci=5`: `y=305.1` (가운데 lane 에서 `ci=4` 아래)
  - `ci=7`: `y=350.7`, `ci=8`: `y=511.2` (오른쪽 lane 에서 `ci=6` 아래)

## 판단

오른쪽 표가 왼쪽 큰 표 아래로 전역 누적되는 현상은 render tree 좌표에서도
해소됐다. 현재 기본 경로는 제보 샘플의 모든 표를 page 1 에 유지하고, 실제 SVG
위치도 좌/중/우 lane 별로 병렬 배치한다.

## 남은 문제

page 2 에 빈 문단 `pi=2` 가 아직 남아 최종 page count 는 1 이 아니라 2이다.
또한 구현계획서의 Stage 4 범위인 debug build composer range panic 방어는 아직
진행하지 않았다.

## 다음 단계

Stage 4 에서 `src/renderer/composer.rs` 의 비정상 line range 방어를 추가하고,
debug build 로 제보 샘플이 panic 없이 처리되는지 확인한다.
