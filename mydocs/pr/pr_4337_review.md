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
| 메인터너 code candidate | `5926828dcc0d5c9baf212aa8707d8ed033714f64` |

접수 상태와 녹색 check는 contributor head 기준이다. 새 release workflow는 tag push와
`workflow_dispatch`만 트리거하므로 그 head에서도 실제 게시 경로는 실행되지 않았다.

## Contributor 변경 범위

contributor commit `85fb44bf`는 플랫폼별 Python wheel과 sdist, `@rhwp/node` 패키지
빌드·게시 workflow를 추가했다. `hatch_build.py`, Python package metadata,
`tools/set_package_version.py`, 릴리스 가이드와 stage 기록도 함께 추가했다.
contributor commit은 수정하거나 재작성하지 않았다.

## 원래 차단점

- `workflow_dispatch.tag`는 버전 문자열에만 쓰이고 checkout은 workflow를 실행한 기본 ref를
  사용해, 과거 태그 이름으로 현재 branch source를 게시할 수 있었다.
- prerelease npm publish에 non-latest dist-tag가 없어 prerelease가 `latest`를 점유하거나 npm에서
  거절될 수 있었다.
- x86_64와 arm64 macOS wheel smoke가 같은 Apple Silicon runner의 기본 Python을 사용해,
  x86_64 wheel이 arm64 Python supported tags에서 설치 거절될 수 있었다.

## 메인터너 보정

`13a41d380e3ad117e680872c3d5148df142cfaf6`
(`fix(maintainer): #4337 릴리스 소스를 요청 태그로 고정`)은 다음을 보정했다.

- `.github/workflows/release-packages.yml`: dispatch는 입력 태그를 checkout하고, tag 형식,
  tag commit, HEAD와 Cargo version을 검증한다. downstream build는 검증된 불변 SHA를 checkout한다.
- `scripts/tests/test_release_packages_workflow.py`: dispatch와 downstream source 선택 계약을 고정한다.
- `.github/workflows/ci.yml`: 새 workflow 계약 테스트를 lint job에 배선한다.

독립 후속 검토 뒤 `5926828dcc0d5c9baf212aa8707d8ed033714f64`
(`fix(maintainer): #4337 prerelease 게시와 wheel arch 보호`)을 추가했다.

- `.github/workflows/release-packages.yml`: prerelease npm publish는 `--tag next`, stable은
  명시적 `latest`를 사용한다. wheel matrix는 artifact target에 맞춰 x64 또는 arm64 Python을 설치한다.
- `scripts/tests/test_release_packages_workflow.py`: dist-tag와 Python architecture 배선을 고정한다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `python -m unittest scripts.tests.test_release_packages_workflow scripts.tests.test_workflow_contract_wiring -v` | 7 / 7 통과 |
| `git diff --check origin/pr/4337..5926828dcc0d5c9baf212aa8707d8ed033714f64` | 통과 |
| commit graph | contributor history를 rewrite하지 않고 maintainer code/docs/code를 single-parent로 연결 |

Rust source나 renderer를 바꾸지 않아 Cargo·시각 회귀는 로컬 범위에서 생략했다. 로컬 환경에는
`actionlint`가 없다. 또한 이 release workflow에는 `pull_request` trigger가 없으므로 PR Full CI는
focused static contract만 실행하며 실제 Linux/macOS/Windows wheel matrix와 npm/PyPI publish E2E를
검증할 수 없다. x64 Python의 Rosetta 실행을 포함한 release E2E는 잔여 risk다.

## 최종 권고

**메인터너 보정 포함 조건부 수용 권고.** code/test/workflow 보정이 있으므로 contributor head의
기존 녹색 결과를 review-only fast-pass로 재사용할 수 없다. 작업지시자가 push를 승인한 뒤 correction
commit들과 이 trailing review 기록을 source branch에 fast-forward로 반영하고, 최신 head의 Full CI
static/focused contract, required checks와 mergeability를 확인해야 한다. release publish E2E 미검증
risk를 명시적으로 수용한 뒤에도 별도의 merge 승인이 있어야 한다. 현재 기록은 push·review 게시·merge
권한을 부여하지 않는다.

실행 및 rollback 경계는 [PR #4337 구현·통합 계획](pr_4337_review_impl.md)을 따른다.
