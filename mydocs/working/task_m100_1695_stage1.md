# task m100 #1695 stage1: LINE_SEG vpos reset/rewind 원인 검증

## 목표

SO-SUEOP HWP3 샘플에서 `LINE_SEG` `vpos` reset/rewind가 페이지/단 경계 처리에 반영되지 않는 원인을 확인하고, 최소 수정으로 회귀 테스트를 추가한다.

## 시작 상태

- 브랜치: `task/m100-1695-vpos-reset`
- 기준 커밋: `440abd254`
- 워킹트리: 깨끗함
- 이슈: https://github.com/edwardkim/rhwp/issues/1695

## 사전 확인

- `target/debug/rhwp dump-pages samples/SO-SUEOP.hwp` 결과는 46페이지다.
- `target/debug/rhwp dump-pages samples/SO-SUEOP.hwp --respect-vpos-reset` 결과는 기본 출력과 동일하다.
- `target/debug/rhwp export-svg` 기본 출력과 `--respect-vpos-reset` 출력도 동일하다.
- 현재 `--respect-vpos-reset` 옵션은 legacy `Paginator` 경로에만 반영되고, 기본 `TypesetEngine` 경로에는 실질 영향이 없다.

## stage1 작업

1. `TypesetEngine`의 내부 `vpos` reset/rewind 감지 조건을 검토한다.
2. SO-SUEOP 본문 후보(`pi=179`, `pi=634`, `pi=748`)를 테스트로 고정한다.
3. 경계 힌트 처리 규칙을 최소 범위로 수정한다.
4. 페이지 수와 visual sweep 기준 회귀 여부를 확인한다.

## 구현 결과

- `TypesetEngine`의 내부 `vpos` reset/rewind 감지를 원본 HWP3 source에도 적용하도록 `is_hwp3_source` 전달 경로를 추가했다.
- 기존 sample16 전용 예외는 유지하되, 원본 HWP3 source의 일반 텍스트 문단에서 저장 `LINE_SEG`가 페이지 하단부에서 near-top으로 되감기는 경우만 내부 page break 후보로 본다.
- SO-SUEOP의 `pi=179`, `pi=634`, `pi=748`은 각각 reset 줄에서 `PartialParagraph`로 분할된다.

## 검증

- `env CARGO_INCREMENTAL=0 cargo test --test issue_1695 -- --nocapture`
  - 결과: 통과, 2 passed
- `env CARGO_INCREMENTAL=0 cargo build`
  - 결과: 통과
- `target/debug/rhwp export-render-tree samples/SO-SUEOP.hwp -p 5/23/28`
  - 결과: 기존 `LAYOUT_OVERFLOW` 로그가 사라짐
- `python3 scripts/task1274_visual_sweep.py --key issue1695-so-sueop-hwp --hwp samples/SO-SUEOP.hwp --pdf pdf/SO-SUEOP-2024.pdf --out output/issue1695_after_visual --rhwp-bin target/debug/rhwp`
  - 결과: SVG 46페이지, PDF 46페이지, `flagged=0/46`
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1692`
  - 결과: 통과, 11 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1375_endnote_rewind_column_overflow`
  - 결과: 통과, 2 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1733`
  - 결과: 통과, 2 passed
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
  - 결과: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
  - 결과: `tests/issue_1035_alignment.rs::hwp3_sample16_hwp5_2022_page_count_64` 실패
  - 원인: 새 일반 rewind 규칙이 HWP5 변환본(`is_hwp3_variant`)까지 열려 sample16-hwp5-2022가 64→65페이지로 over-split
  - 조치: 새 일반 규칙을 원본 HWP3 source에만 적용하도록 축소
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1695 -- --nocapture`
  - 결과: 축소 후 재통과, 2 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1035_alignment`
  - 결과: 통과, 4 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
  - 결과: 축소 후 재실행 통과
- `git diff --check`
  - 결과: 통과
