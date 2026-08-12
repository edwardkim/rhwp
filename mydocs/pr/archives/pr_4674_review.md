---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4674 검토 - stale run 취소에 시스템 CA 사용

## 라우팅과 접수

```text
base route: maintainer_general
modifiers: intake_and_review.md, local_validation.md,
  review_only_fast_pass.md
```

| 항목 | 기록 |
| --- | --- |
| PR | [#4674](https://github.com/edwardkim/rhwp/pull/4674) |
| 관련 이슈 | [#4673](https://github.com/edwardkim/rhwp/issues/4673) |
| code candidate | `92dbeda54f01953b172c04fb88617421ddfc0bef` |
| 기준선 | `upstream/devel` `7a04b1f72569b1be19309fcd02012cec75cf4784` |
| 변경 범위 | stale run 취소 workflow와 Python workflow contract test |
| reviewer request | 작성자 maintainer self-review 경로라 별도 reviewer request 없음 |

## 변경 판단

[PR #4670의 실패 job](https://github.com/edwardkim/rhwp/actions/runs/31571978785/job/94035729608)은
GitHub API의 stale run 목록 조회 중 `DEPTH_ZERO_SELF_SIGNED_CERT`로 중단됐다. 실패는 Studio bridge
변경과 무관한 GitHub-hosted runner의 Node TLS 신뢰 저장소 경로였고, 같은 PR의 Build & Test·CodeQL·Render
Diff는 성공했다.

`actions/github-script` step에 `NODE_OPTIONS=--use-system-ca`를 적용해 시스템 CA 저장소를 사용한다.
`NODE_TLS_REJECT_UNAUTHORIZED=0`처럼 TLS 검증을 끄는 설정은 사용하지 않으며, 기존 same-repository/fork
guard와 stale run 완료 race 재조회 규칙도 바꾸지 않는다.

## 완료한 검증

- `python3 -m unittest scripts/tests/test_cancel_stale_pr_runs_workflow.py -v`: 2건 통과.
- 로컬 Node `v24.15.0`에서 `--use-system-ca` 지원을 확인했다.
- `git diff --check`: 통과.
- code candidate의 [CI run](https://github.com/edwardkim/rhwp/actions/runs/31575675269)과
  [CodeQL run](https://github.com/edwardkim/rhwp/actions/runs/31575675263)은 Full CI·Native Skia·nextest
  shard·Build & Test까지 모두 통과했다.

Rust 소스는 변경하지 않아 로컬 전체 Cargo 회귀는 실행하지 않았다. workflow 변경이므로 GitHub CI의 full
lane 결과를 검증 근거로 사용했다.

## trailing docs와 최종 판단

이 문서와 오늘할일은 code candidate가 녹색이 된 뒤 추가한 review-only commit이다. 이 push의 `synchronize`
이벤트에서 `cancel-stale-runs`가 실제로 시스템 CA 설정으로 성공하는지 확인한 뒤 merge를 판단한다.
