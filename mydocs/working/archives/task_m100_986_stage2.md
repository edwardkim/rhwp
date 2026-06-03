# Stage 2 완료 보고서 — Task #986

## 범위

기본 조판 경로인 `TypesetEngine` 에 Stage 1 의 가로 lane helper 를 연결했다.
이번 단계는 pagination 단계에서 제보 샘플의 오른쪽 표가 왼쪽 표 아래로
누적되어 `PartialTable` 로 분할되는 현상을 줄이는 데 초점을 두었다.

## 변경 파일

- `src/renderer/typeset.rs`
  - 텍스트가 없는 호스트 문단의 `TopAndBottom + vert=Para + non-TAC` 표에
    `FloatLaneSet` 기반 배치 적용
  - 같은 문단의 표라도 가로 범위가 겹치지 않으면 같은 세로 영역에 배치 허용
  - 실제 overflow 시에는 기존 `typeset_block_table` 분할 경로로 fallback

## 검증

```bash
cargo fmt --all --check
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --lib float_placement
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo run --release --bin rhwp -- dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
```

결과:

- `cargo fmt --all --check`: 통과
- `cargo test --lib float_placement`: 6개 테스트 통과
- 기본 `dump-pages`: `ci=4`, `ci=6` 의 `PartialTable` 분할 제거

## 재현 결과 변화

수정 전 기본 경로:

- 3페이지
- `ci=4` 가 page 1/2 로 분할
- `ci=6` 이 page 2/3 으로 분할

수정 후 기본 경로:

- 2페이지
- page 1 에 `ci=2..8` 전체 표가 모두 `Table` 로 배치
- `PartialTable` 없음
- page 2 에 빈 문단 `pi=2` 1개가 남음

## 남은 문제

pagination 상 표 분할은 해소됐지만, trailing 빈 문단 때문에 page count 가 아직 1이
아니라 2이다. 또한 Stage 2 는 pagination item 배치만 조정했으므로, 실제 render tree
좌표는 Stage 3 에서 layout lane 동기화를 적용해야 한다.

## 다음 단계

Stage 3 에서 `src/renderer/layout.rs` / `src/renderer/layout/table_layout.rs` 에
동일한 lane 판단을 적용한다. 목표는 SVG/debug overlay 에서 오른쪽 표가 실제로
첫 페이지 오른쪽 영역에 배치되도록 만드는 것이다.
