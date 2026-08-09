---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4372 검토 - GHCR 소스와 stable latest 보호

## 검토 경로

기본 경로는 `maintainer_general.md`, 보조 경로는 `intake_and_review.md`,
`local_validation.md`, `multi_pr_update_branch.md`, `review_only_fast_pass.md`다.
Docker CLI packaging과 workflow만 바뀌며 renderer, layout, fixture, sample 영향은 없다.

## 접수 메타데이터

| 항목 | 접수 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#4372](https://github.com/edwardkim/rhwp/pull/4372) / `kevin9327` |
| 관련 이슈 | [#4354](https://github.com/edwardkim/rhwp/issues/4354) |
| base / contributor head | `devel` / `91782d3364b3b8929cac00a4d449e6785d93e6a1` |
| 규모 | 3 files, +98 / -0, contributor commits 2개 |
| 상태 | `MERGEABLE` / `CLEAN`, Full CI·CodeQL·Docker CLI Image build 성공 |
| 가시성 branch | `review/kevin9327-20260810-pr4372` |
| 메인터너 code candidate | `71aecd1273864ae42b5b19fa9382aa43c8f0ef77` |

## Contributor 변경 범위

`c1d4da0878c94c898a4ebb903c4b9f6b46fc14b2`는 `Dockerfile.cli`과 GHCR publish
workflow를 추가했다. `91782d3364b3b8929cac00a4d449e6785d93e6a1`은 embedded manual이
Docker build context에서 누락되지 않도록 `.dockerignore`를 보정했다. 두 commit의 history를
그대로 유지했다.

## 원래 차단점

- 수동 dispatch의 tag 입력은 image label에만 반영되고 checkout은 workflow 실행 ref를 사용해,
  요청 태그와 다른 source를 게시할 수 있었다.
- tag push 조건 `v*`에는 prerelease도 포함되지만 모든 publish가 `:latest`를 갱신해,
  rc/beta/alpha image가 stable latest를 대체할 수 있었다.

## 메인터너 보정

`71aecd1273864ae42b5b19fa9382aa43c8f0ef77`
(`fix(maintainer): #4372 배포 소스와 latest 태그를 보호`)은 다음 파일을 바꿨다.

- `.github/workflows/docker-publish.yml`: dispatch tag checkout, tag/HEAD/Cargo 검증,
  PR event ref 유지, prerelease의 latest publish 차단
- `scripts/tests/test_docker_publish_workflow.py`: source와 latest 보호 계약
- `.github/workflows/ci.yml`: 새 계약 테스트 배선

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `python -m unittest scripts.tests.test_docker_publish_workflow scripts.tests.test_workflow_contract_wiring -v` | 5 / 5 통과 |
| `git diff --check origin/pr/4372..71aecd1273864ae42b5b19fa9382aa43c8f0ef77` | 통과 |
| commit graph | correction commit의 유일한 parent가 contributor head와 일치 |

실제 Docker build와 GHCR publish는 로컬에서 재실행하지 않았다. `actionlint`도 로컬에 없어,
최신 원격 head의 workflow parser, Docker build smoke와 Full CI가 남아 있다.

## 최종 권고

**메인터너 보정 포함 조건부 수용 권고.** 기존 contributor head의 녹색 Docker/CI 결과는 새
workflow code candidate를 검증하지 않는다. push 승인 뒤 correction과 review 기록을 fast-forward로
반영하고 최신 head의 Full CI, Docker CLI Image check, required aggregate와 mergeability가 모두
성공해야 한다. 그 후에도 별도 merge 승인이 필요하며, 현재는 원격 변경 권한이 없다.

실행 및 rollback은 [PR #4372 구현·통합 계획](pr_4372_review_impl.md)을 따른다.
