# Stage 1 완료 보고서 — Task #986

## 범위

구현계획서 Stage 1 범위인 `TopAndBottom` 비글자취급 표의 가로 lane 배치
helper 와 단위 테스트를 추가했다.

## 변경 파일

- `src/renderer/float_placement.rs`
  - signed HWPUNIT 해석 helper 추가
  - `TopAndBottom + vert=Para + non-TAC` 판정 helper 추가
  - 기존 table layout 의 depth=0 수평 위치 공식과 맞춘 horizontal range helper 추가
  - 가로 겹침 기반 `FloatLaneSet` 추가
- `src/renderer/mod.rs`
  - `float_placement` 모듈 등록

## 테스트

```bash
cargo fmt --all --check
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --lib float_placement
```

결과:

- `cargo fmt --all --check`: 통과
- `cargo test --lib float_placement`: 6개 테스트 통과

테스트 중 기존 코드의 warning 이 함께 출력되었으나 Stage 1 변경과 무관하다.

## 특이사항

초기 테스트 실행은 파일시스템 여유 공간 부족으로 실패했다.
이 worktree 의 `target/` 산출물만 `cargo clean` 으로 정리한 뒤 재실행하여 통과했다.

## 다음 단계

Stage 2 에서 `src/renderer/typeset.rs` 의 기본 조판 경로에 lane helper 를 연결한다.
목표는 제보 샘플의 기본 `TypesetEngine` 경로에서 `ci=4`, `ci=6` 이
`PartialTable` 로 분할되지 않도록 하는 것이다.
