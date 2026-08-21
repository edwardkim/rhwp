---
kind: pr-review
status: approved-integration-candidate
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5794 검토 - 명령별 도움말

## 판정

- source head `70af8d672dd17e03bff155ad5259d1ce4f49445b`를 최신 `upstream/devel` 통합 후보에 적용했다.
- 명령·그룹·값 슬롯별 `--help` 해석을 명시적으로 분리하며, 기존 오류 경로를 보존한다. 차단 결함은 없다.

## 검증

- source CI는 작성 시점 `clean`이었다.
- `cli_scoped_help_contract` 9/9, `threat_scan_cli_contract` 5/5 통과.
- 통합 후보 전체 nextest 8,109/8,109, fmt, clippy, native-Skia, WASM 표준 빌드가 통과했다.
