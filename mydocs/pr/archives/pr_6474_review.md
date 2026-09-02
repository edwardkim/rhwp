---
kind: pr-review
status: active
pr: 6474
source-pr: 6415
---

# PR #6474 검토 - #6415 Oracle page-count 선택 계약 보정

## 결론 - 수용 후보, CI 대기

[PR #6474](https://github.com/edwardkim/rhwp/pull/6474)는 원 PR #6415의 Hancom Oracle 쪽수
baseline과 gate 문서를 최신 `devel`에 이식하되, #6466이 정한 source-relative canonical PDF
선택 계약을 보존한다. 현재 head는 `74e824e265325ef0ac7fb75c6957257b9fb6fde4`이며, 최신 required
CI 성공과 mergeability 확인 전에는 병합하지 않는다.

## 변경 판단

- `tools/oracle_page_count/regenerate.py`는 `rhwp info --json`의
  `lastSavedWith.product`로 engine을 고른다. `hancom-office-2024`는 engine 2024, 2022 이하와
  메타데이터가 없는 문서는 engine 2020이다.
- canonical PDF는 원본의 `samples/` 상대 경로, HWP/HWPX 형식, 선택된 engine이 모두 맞아야 한다.
  같은 stem의 다른 형식 또는 다른 engine PDF를 최신 파일이라는 이유만으로 선택하지 않는다.
- 원 PR #6415의 느슨한 pair-index와 newest-engine 선택 변경은 제외했다. #6466의 fail-closed 선택
  계약을 되돌리면 원본과 무관한 PDF가 Oracle 기준으로 선택될 수 있기 때문이다.
- `samples/2025 행정업무운영 편람(최종).hwp`는 Hancom PDF와 rhwp가 모두 384쪽인 baseline이다.
  HWPX는 Hancom PDF 384쪽과 rhwp 382쪽 사이의 미해결 레이아웃 차이를 baseline에 그대로 기록한다.
  이 PR은 그 차이를 수용하거나 renderer fidelity가 통과했다고 주장하지 않는다.

## 검증 기록

- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check` - passed
- `cargo fmt --all -- --check` - passed
- native Clippy, WASM lib Clippy, workspace build, workspace all-target Clippy - passed
- `cargo test --locked --test regression_suite_007 page_counts_do_not_drift_from_hancom_oracle_partition -- --nocapture`
  - 16 passed, 0 failed
- `python3 -m py_compile tools/oracle_page_count/regenerate.py` - passed
- `git diff --check` - passed

## 후속 상태

- 이 PR은 Rust baseline helper와 Oracle 기준 데이터를 함께 바꾸므로 review-only fast-pass로
  가정하지 않는다. 최신 head의 실제 required CI와 Rust CodeQL 실행 결과를 확인한 뒤 병합 여부를
  재판정한다.
- 원 PR #6415는 자동 종료하지 않는다. 이 PR은 fork 권한 제약으로 원 PR에 직접 push할 수 없었던
  maintainer 보정을 별도 upstream PR로 분리한 것이다.
