---
kind: pr-review
status: review-complete-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5778 검토 - 검토 PDF 증적의 독립 워크플로 fast-pass

## 접수 메타데이터

| 항목 | 검토 기록 |
| --- | --- |
| PR / 작성자 | [#5778](https://github.com/edwardkim/rhwp/pull/5778) / `jangster77` self-review |
| 관련 issue | [#5777](https://github.com/edwardkim/rhwp/issues/5777) |
| base / code candidate | `devel` / `dbc91c0b823d6065ddb4bb081725777d280a6965` |
| source branch | `fix/ci-review-reference-fast-pass` |
| 변경 규모 | workflow 2개, Python 계약 test 3개, fast-pass 절차 문서 1개 |
| 라우팅 | `collaborator_self_merge` + `intake_and_review` + `local_validation` |
| PR 생성 확인 | Open, non-draft, `MERGEABLE`; reviewer 미지정 |

## 원인과 변경 범위

[#5772](https://github.com/edwardkim/rhwp/pull/5772)의 최초 증적 trailing commit은 `mydocs/`와
루트 `pdf/` 아래의 **새 PDF**만 추가했다. 중앙 Build & Test는 이를 review-only로 판정했지만,
`Proptest roundtrip`과 `Adapter inter-diff`는 각각 `mydocs/` 밖의 코드 변경으로 분류해 full worker를
실행했다.

또한 그 PR의 후속 `mydocs/` trailing commit은 fork PR run의 `pull_requests` 배열이 비어 있어 이전
성공 run을 찾지 못했다. 두 워크플로는 full worker를 실행해 성공했으나, 문서 증적 tail에 필요한
fast-pass가 적용되지 않았다.

이 PR은 두 독립 워크플로의 review-only 정책을 중앙 CI와 동일하게 맞춘다.

- `mydocs/**` 전체와 새 `samples/**`의 `.hwp`, `.hwpx`, `.pdf`, `.png`만 허용한다.
- 새 `pdf/**`, `pdf-2020/**`, `pdf-large/**` PDF도 허용하되, 기존 reference PDF 수정은 계속 full
  검증으로 보낸다.
- trailing candidate run은 `pull_requests` 배열에 의존하지 않고 `pull_request` event, head SHA,
  head branch, head repository ID 및 PR 생성 시각으로 확인한다.
- 검토 전용 PR 자체는 두 worker를 skip하고, 허용되지 않은 path 또는 다른 fork의 run은 이전 결과를
  재사용하지 않는다.

변경은 GitHub Actions 경로 필터와 그 계약 테스트에만 한정한다. Rust, renderer, fixture, PDF 생성물 및
기존 CI 결과 해석을 바꾸지 않으므로 시각 검토와 전체 Rust 회귀는 적용 대상이 아니다.

## 로컬 검증

- `node scripts/rust-test-suite-manifest.mjs --prepare`: `cargo fmt`가 참조하는 ignored 파생 suite만 준비.
  생성물은 수정 diff에 남지 않았다.
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`: 통과.
- 아래 workflow contract test: **37 passed, exit 0, 1.104초**.

```text
python3 -m unittest \
  scripts/tests/test_review_only_fast_pass_workflows.py \
  scripts/tests/test_proptest_roundtrip_workflow.py \
  scripts/tests/test_adapter_diff_workflow.py \
  scripts/tests/test_workflow_contract_wiring.py
```

새 테스트는 허용된 추가 PDF, `pull_requests`가 없는 동일 fork trailing 후보 재사용, 기존 PDF 수정 및
다른 fork 후보의 full 검증 fallback을 실제 workflow JavaScript를 mock GitHub API로 실행해 확인한다.

## 최종 권고

**수용 권고, 최신 PR head의 workflow CI 대기.** 경로 허용은 review 증적의 추가에만 제한되고 기존 기준
파일 수정, code 변경, 다른 fork run 재사용을 여전히 차단한다. CI가 성공하면 이 문서와 오늘할일을 포함한
최신 head의 mergeability를 다시 확인한 뒤 작업지시자 승인에 따라 merge 및 #5777 후속 처리를 진행한다.
