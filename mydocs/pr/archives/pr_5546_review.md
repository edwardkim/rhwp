---
kind: pr-review
status: approved-local-validation
pr: 5546
issue: 5508
base: devel
source_head: b3a4987765995bc0de9436d6ca72c04caa5649e9
integration_commit: 0aadd6a26d64d9085a6bb2a2198159ad8c120970
---

# PR #5546 V-abstain 봉투 모순 기권 검토

## 라우팅

```text
base route: collaborator maintainer integration
modifiers: external PR cherry-pick, cumulative validation, individual review record
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, post_merge.md
```

## 적용 범위

| 항목 | 값 |
| --- | --- |
| Source PR | [#5546](https://github.com/edwardkim/rhwp/pull/5546) |
| 작성자 | `kevin9327` |
| 관련 이슈 | [#5508](https://github.com/edwardkim/rhwp/issues/5508) |
| 원본 head | `b3a4987765995bc0de9436d6ca72c04caa5649e9` |
| 적용 commit | `0aadd6a26d64d9085a6bb2a2198159ad8c120970` |

- 명명된 봉투 필드가 모순되면 결과를 pass/fail로 추정하지 않고 닫힌 집합 `pass | fail | abstain`에서 `abstain`을 반환한다.
- 동일 노드의 page count 일치와 `STRUCT_MISMATCH`, `reproduced`와 exit 3 등 선언된 모순을 golden fixture와 대규모 코퍼스로 고정한다.
- TSV golden fixture의 끝 탭은 고정 열 수 계약을 나타내므로 유지한다. 이를 제거하면 strict row binding이 실패한다.

## 검증 기록

- `python3 -m unittest discover -s tools/llm_verifier/abstain/tests -v`: 18 passed
- `python3 -m unittest scripts.tests.test_llm_verifier_workflow`: 1 passed
- `cargo fmt --all -- --check`: 통과
- `node scripts/rust-test-suite-manifest.mjs --check`: 통과
- `node scripts/rust-unit-test-tiers.mjs --check`: 통과

## 결론

**로컬 승인.** 원 저자 commit을 `-x` 계보로 적용했다. 누적 integration PR의 required CI가 성공하면
[#5508](https://github.com/edwardkim/rhwp/issues/5508)을 aggregate PR로 종료하고, 원 PR #5546에는
적용 commit과 병합 PR을 남긴 뒤 close한다.
