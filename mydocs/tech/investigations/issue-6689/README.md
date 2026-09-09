---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_6689.md
last_verified: 2026-09-05
---

# #6689 workflow 승격 공백 기준선

## 조사 질문

`devel`에서 추가·변경된 workflow가 실제 GitHub-hosted runner 실행 없이 `main`에 처음 등록되는 경로를
어떻게 검출하고 막을 것인가를 조사한다. 개별 workflow의 성공 여부와 일반 CI의 성공 여부를 섞지 않고,
다음 세 사실을 별도로 고정한다.

1. `main` tree와 candidate `devel` tree 사이에서 어떤 workflow content가 바뀌었는가
2. candidate exact SHA에서 해당 workflow가 실제로 실행됐는가
3. 필수 job·verdict가 성공했는가, 또는 승인된 제한 waiver가 있는가

기계 판독 snapshot은 [`baseline.json`](baseline.json)에 있다. Stage 5의 exact `devel` Fuzz 실실행
영수증은 [`fuzz-smoke-dogfood.json`](fuzz-smoke-dogfood.json)에 있다. 모든 시각은 별도 표기가 없으면
UTC다.

## Git 기준선

| 항목 | SHA |
| --- | --- |
| `upstream/main` | `e8800c8def63449808a4092798442652ed460552` |
| `upstream/devel` | `1c49df3d33a323d459c8e90517f4a0f5bd3c790b` |
| merge-base | `51043f5f8d0453b9bc929233de443fa60cb3df4b` |

`git rev-list --left-right --count upstream/main...upstream/devel`은 `2 238`이다. main 고유 commit은 v0.8.6
release merge와 release CI hotfix이고, devel은 그 뒤 238개 commit이 전진했다. 따라서 승격 검사는
`main`이 candidate의 조상이라고 추정해서는 안 되며 base, head, merge-base와 실제 integration tree를
각각 기록해야 한다.

`main..devel`의 `.github/workflows/**`·`.github/actions/**` 차이는 수정 8개, 추가·삭제 0개다.
`baseline.json`은 각 path의 before/after Git blob과 파일 byte SHA-256을 고정한다.

## live Actions 기준선

2026-09-05 10:55 KST 기준 candidate exact head에는 다음 성공 run이 있다.

| workflow | event | run |
| --- | --- | --- |
| CI | push | [33935431341](https://github.com/edwardkim/rhwp/actions/runs/33935431341) |
| CodeQL | push | [33935431316](https://github.com/edwardkim/rhwp/actions/runs/33935431316) |
| CodeQL | workflow_dispatch | [33935568575](https://github.com/edwardkim/rhwp/actions/runs/33935568575) |
| Adapter inter-diff | push | [33935431349](https://github.com/edwardkim/rhwp/actions/runs/33935431349) |
| Proptest roundtrip | push | [33935431404](https://github.com/edwardkim/rhwp/actions/runs/33935431404) |

같은 SHA에서 수정 workflow 중 Deploy Pages, Gym, Oracle advisory, Render Diff run은 없다. 이 표는 Stage 1
기준선이지 최종 증적이 아니다. #6689 구현 commit으로 candidate SHA와 workflow hash가 바뀌므로 최종
preflight에서는 새 exact-head 결과를 요구한다.

`main`·`devel`의 `fuzz-smoke.yml` Git blob은
`b9c7394b1711d6d73463682a2989188cc42e5c30`으로 같다. 현재 원격 run은 `main` schedule 세 번뿐이고,
모두 `parse_wmf`만 실패했다.

| run | head SHA | 결과 |
| --- | --- | --- |
| [33602887449](https://github.com/edwardkim/rhwp/actions/runs/33602887449) | `f1f9c6ae58344ee9368996d3543f76b9345cf227` | 5 success, `parse_wmf` failure |
| [33727700561](https://github.com/edwardkim/rhwp/actions/runs/33727700561) | `e8800c8def63449808a4092798442652ed460552` | 5 success, `parse_wmf` failure |
| [33848537271](https://github.com/edwardkim/rhwp/actions/runs/33848537271) | `e8800c8def63449808a4092798442652ed460552` | 5 success, `parse_wmf` failure |

## 누락 원인 계보

- PR #5366은 신규 `Fuzz smoke`와 6-target matrix를 제안했다.
- 검토 문서 `mydocs/pr/archives/pr_5366_review.md`는 “nightly runner는 원격 workflow에서 확인 필요”라고
  명시했지만 이를 merge 종료 조건이나 구조화된 증적으로 만들지 않았다.
- 원 PR #5366에는 check가 없었고, 통합 PR #5425에는 CI·CodeQL·Render Diff·Proptest 등 일반 check가
  성공했지만 `Fuzz smoke` check가 없다.
- contributor commit `39c11deca5a05cdaa89689269eeb851a9090de03`이 2026-08-18 devel에 수용된 뒤
  `main`에는 2026-09-02 PR #6592로 처음 포함됐다.
- 첫 schedule run이 그날 `parse_wmf` 제품 panic을 드러냈다. 즉 검토자가 필요성을 알아차렸지만
  텍스트 메모를 기계 gate로 바꾸지 않은 운영 결함이다.

## RED 계약의 판정 경계

Stage 1 test는 다음 상태에서 의도적으로 실패해야 한다.

- YAML scalar 밖의 주석·빈 줄만 바뀐 경우만 comment-only로 인정한다.
- block scalar 안의 shell 내용은 `#`로 시작해도 실행 자산으로 보존한다.
- trigger, permission, secret, matrix, action ref, cache, artifact, timeout, concurrency와 job command
  변화는 위험 축으로 보고한다.
- 일반 CI run이 녹색이어도 변경된 workflow path의 run이 없으면 증적 누락이다.
- run head SHA·workflow hash가 candidate와 다르면 stale evidence다.
- 필수 job의 `skipped`, `failure`, `cancelled`, `pending`은 성공이 아니다.
- `continue-on-error` workflow는 녹색 run만으로 통과하지 않고 별도 verdict와 artifact를 요구한다.
- waiver는 maintainer, exact candidate·workflow hash, scope, reason, URL, 만료가 모두 맞아야 한다.

이 계약을 통과시키는 구현은 Stage 2에서 시작한다.
