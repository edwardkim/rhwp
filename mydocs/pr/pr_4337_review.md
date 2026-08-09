---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4337 검토 - PyPI/npm 릴리스 소스 고정

## 검토 경로

기본 경로는 `maintainer_general.md`다. 보조 경로는 `intake_and_review.md`,
`local_validation.md`, `multi_pr_update_branch.md`, `review_only_fast_pass.md`다.
릴리스 workflow와 바인딩 패키징만 바뀌며 renderer, layout, fixture, sample 변경은 없어
시각 검증 대상이 아니다.

## 접수 메타데이터

| 항목 | 접수 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#4337](https://github.com/edwardkim/rhwp/pull/4337) / `kevin9327` |
| 관련 이슈 | [#4336](https://github.com/edwardkim/rhwp/issues/4336) |
| base / contributor head | `devel` / `85fb44bf63e753af6e4c03055f9d96c7b23ffaba` |
| 규모 | 6 files, +489 / -0, contributor commit 1개 |
| 상태 | `MERGEABLE` / `CLEAN`, contributor head의 Full CI·CodeQL·Python binding 성공 |
| 가시성 branch | `review/kevin9327-20260810-pr4337` |
| 메인터너 code candidate | `13a41d380e3ad117e680872c3d5148df142cfaf6` |

접수 상태와 녹색 check는 contributor head 기준이다. 새 release workflow는 tag push와
`workflow_dispatch`만 트리거하므로 그 head에서도 실제 게시 경로는 실행되지 않았다.

## Contributor 변경 범위

contributor commit `85fb44bf`는 플랫폼별 Python wheel과 sdist, `@rhwp/node` 패키지
빌드·게시 workflow를 추가했다. `hatch_build.py`, Python package metadata,
`tools/set_package_version.py`, 릴리스 가이드와 stage 기록도 함께 추가했다.
contributor commit은 수정하거나 재작성하지 않았다.

## 원래 차단점

`workflow_dispatch.tag`는 버전 문자열에만 쓰이고 checkout은 workflow를 실행한 기본 ref를
사용했다. 따라서 과거 태그를 지정한 수동 실행이 현재 branch 소스를 그 태그 버전으로 빌드·게시할
수 있었다. 버전 문자열 검사는 Cargo와 일치해도 실제 checkout commit이 해당 태그라는 사실을
보증하지 못했다.

## 메인터너 보정

`13a41d380e3ad117e680872c3d5148df142cfaf6`
(`fix(maintainer): #4337 릴리스 소스를 요청 태그로 고정`)은 다음을 보정했다.

- `.github/workflows/release-packages.yml`: dispatch는 입력 태그를 checkout하고, tag 형식,
  tag commit, HEAD와 Cargo version을 검증한다. downstream build는 검증된 불변 SHA를 checkout한다.
- `scripts/tests/test_release_packages_workflow.py`: dispatch와 downstream source 선택 계약을 고정한다.
- `.github/workflows/ci.yml`: 새 workflow 계약 테스트를 lint job에 배선한다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `python -m unittest scripts.tests.test_release_packages_workflow scripts.tests.test_workflow_contract_wiring -v` | 5 / 5 통과 |
| `git diff --check origin/pr/4337..13a41d380e3ad117e680872c3d5148df142cfaf6` | 통과 |
| commit graph | correction commit의 유일한 parent가 contributor head와 일치 |

Rust source나 renderer를 바꾸지 않아 Cargo·시각 회귀는 로컬 범위에서 생략했다. 로컬 환경에는
`actionlint`가 없어 GitHub Actions parser와 실제 Linux/macOS/Windows wheel build는 최신 원격
head의 Full CI에서 확인해야 한다.

## 최종 권고

**메인터너 보정 포함 조건부 수용 권고.** code/test/workflow 보정이 있으므로 contributor head의
기존 녹색 결과를 review-only fast-pass로 재사용할 수 없다. 작업지시자가 push를 승인한 뒤 correction
commit과 이 trailing review 기록을 source branch에 fast-forward로 반영하고, 최신 head의 Full CI,
required checks와 mergeability를 확인해야 한다. 그 뒤에도 별도의 merge 승인이 있어야 한다.
현재 기록은 push·review 게시·merge 권한을 부여하지 않는다.

실행 및 rollback 경계는 [PR #4337 구현·통합 계획](pr_4337_review_impl.md)을 따른다.
