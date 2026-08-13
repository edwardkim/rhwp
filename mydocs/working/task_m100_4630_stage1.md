# Task #4630 Stage 1 — WASM32 Clippy gate 정합

- Issue: [#4630](https://github.com/edwardkim/rhwp/issues/4630)
- Base: `upstream/devel` `be7dabdd1`
- 측정일: 2026-08-14 KST

## 보정

WASM canvas 전용 경로의 unit-return binding 13개, 색상 red channel의 무의미한 shift 2개,
manual range pattern 1개를 동작 보존 형태로 정리했다. CI Lint에는
`cargo clippy -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings`를 추가했다.

## 검증

- PowerShell, `target\\pr-review`: WASM32 Clippy 16개 오류 재현 뒤 보정 후 성공.
- `python scripts\\tests\\test_ci_impact_workflow.py`: 27/27 통과.
- `git diff --check`: 통과.

`CARGO_INCREMENTAL`은 설정하지 않았다. #4631의 wasm32 CLI 제외 여부와 #4089 Docker 구성은
변경하지 않는다.
