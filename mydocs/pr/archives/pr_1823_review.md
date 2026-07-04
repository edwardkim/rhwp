# PR #1823 리뷰 — ir-diff Shape 글상자 문단 재귀 비교

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1823 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1807` |
| 작성 시점 참고 head | `6d1a3af3e208689d5a3e8dfd8c494e8d019e74ee` |
| 작성 시점 참고 상태 | `MERGEABLE`, GitHub Actions 통과 |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- `src/main.rs` 의 `ir-diff` 비교 경로에서 Shape 내부 `text_box` 문단을 재귀 비교 대상에 포함한다.
- 기존 문서 diff 가 본문 문단만 보고 Shape 글상자 내부 텍스트 차이를 놓칠 수 있던 영역을 CLI 진단 범위로 끌어올린다.
- 문서/계획/보고서가 함께 포함되어 있으며 렌더 출력 자체를 변경하는 PR 은 아니다.

## 로컬 검증

누적 검토 브랜치: `local/batch-pr1823-1840-review`

- 체리픽 커밋: `6d1a3af3e208` -> `dc07469ae`
- 충돌: 없음
- 공통 검증 통과:
  - `git diff --check`
  - `cargo fmt --check`
  - `python3 -m py_compile tools/compare_line_baselines.py`
  - `env CARGO_INCREMENTAL=0 cargo test test_cell_field_name_extra_roundtrip --lib`
  - `env CARGO_INCREMENTAL=0 cargo test render_geom_diff::tests --lib`
  - `env CARGO_INCREMENTAL=0 cargo test --test svg_snapshot`
  - `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
  - `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`

## 판단

PR 목적이 CLI diff 검출 범위 확대이므로 visual sweep 대상은 아니다. Shape 글상자 내부 문단을 구조 diff 에서
누락하지 않도록 하는 변경이며, 누적 검증에서 관련 Rust 빌드/테스트/Clippy 회귀는 확인되지 않았다.

## 결론

merge 후보. GitHub merge 전에는 최신 head 와 required checks 를 다시 확인한다.
