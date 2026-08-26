---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #5179 검토 - PR 파생 Rust suite 산출물 분리

## 접수

| 항목 | 내용 |
| --- | --- |
| PR | [#5179](https://github.com/edwardkim/rhwp/pull/5179) |
| 관련 이슈 | [#5177](https://github.com/edwardkim/rhwp/issues/5177) |
| 작성자·head | `jangster77` / `fix/5177-derived-suite-artifacts` |
| 대상 브랜치 | `devel` |
| 변경 범위 | Rust integration suite 정책, CI workflow, Node·Python 계약 테스트, 개발·검토 가이드 |

## 검토 결과

- 기여자는 `tests/cases/**` 원본만 제출하고, `tests/generated/**`,
  `tests/suites/manifest.json`, Cargo generated test target block은 PR base diff에서 거부한다.
- CI와 nextest archive가 checkout 내부에서만 `rust-test-suite-manifest.mjs --prepare`를 실행하므로,
  병합 순서에 따라 파생 산출물이 충돌하는 문제를 제거한다.
- 최초 CI의 manifest 단계 실패는 base SHA를 `--depth=1`로 가져와 `base...HEAD`의 공통 조상이 없었던
  checkout 결함이었다. lint checkout을 전체 계보로 전환해 3-way diff 기반의 fail-closed 정책은 유지했다.

## 로컬 검증

- `node --test scripts/tests/rust-test-suite-manifest.test.mjs`
- `python3 -m unittest scripts/tests/test_ci_impact_workflow.py`
- Rust 제품 코드와 Studio 코드는 변경하지 않았으므로 Rust 전체 회귀·WASM 시각 검증은 이 보정 범위에
  적용하지 않았다.

## 판단

파생 산출물의 제출 금지와 CI 내부 생성이라는 #5177의 경계는 유지된다. 로컬 계약 검증 후 trailing
commit을 push하며, GitHub Actions 재실행 성공을 병합의 최종 조건으로 둔다.
