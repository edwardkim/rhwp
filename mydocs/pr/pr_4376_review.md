---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4376 검토 - 설치 산출물의 요청 태그 정합

## 검토 경로

기본 경로는 `maintainer_general.md`, 보조 경로는 `intake_and_review.md`,
`local_validation.md`, `multi_pr_update_branch.md`, `review_only_fast_pass.md`다.
설치 workflow, package metadata와 install scripts만 바뀐다. renderer/layout, sample, fixture와
시각 출력 영향은 없다.

## 접수 메타데이터

| 항목 | 접수 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#4376](https://github.com/edwardkim/rhwp/pull/4376) / `kevin9327` |
| 관련 이슈 | [#4375](https://github.com/edwardkim/rhwp/issues/4375) |
| base / contributor head | `devel` / `12f3950644430d7692baec6587f8a1d149f8cae4` |
| 규모 | 8 files, +376 / -0, contributor commits 2개 |
| 상태 | `MERGEABLE` / `CLEAN`, Full CI·CodeQL·Render Diff 성공 |
| 가시성 branch | `review/kevin9327-20260810-pr4376` |
| 메인터너 code candidate | `b1bb5a56a15aa06f84ee99eeecdfb208d5cc406b` |

`release-installers.yml`은 tag push와 manual dispatch만 트리거하므로 contributor head에서
deb/rpm/MSI/crates.io workflow 자체는 실행되지 않았다.

## Contributor 변경 범위

`cc94b1c0e6a5b3972212cbaf1fc8bca102b8be37`은 Cargo packaging metadata,
deb/rpm/MSI/crates.io workflow, POSIX·PowerShell installer, AUR, MCP registry metadata와
운영 가이드를 추가했다. `12f3950644430d7692baec6587f8a1d149f8cae4`는 미병합 가이드의
내부 상대 링크를 PR 링크로 바로잡았다. 두 contributor commit을 그대로 보존했다.

## 원래 차단점

- manual dispatch 입력 tag는 upload 대상 이름에만 쓰고 각 job은 workflow 실행 ref를 checkout했다.
  따라서 현재 default source로 만든 deb/rpm/MSI나 crate를 과거 release에 덮어쓸 수 있었다.
- `contrib/install/install.sh`는 Darwin을 지원한다고 선언했지만 stock macOS에 없는
  `sha256sum`만 호출해 checksum 검증 전에 종료됐다.

## 메인터너 보정

`b1bb5a56a15aa06f84ee99eeecdfb208d5cc406b`
(`fix(maintainer): #4376 설치 산출물을 요청 태그에 고정`)은 다음을 바꿨다.

- `.github/workflows/release-installers.yml`: 요청 tag checkout, tag/HEAD/Cargo version 검증,
  검증된 불변 SHA를 모든 packaging/publish job에 전달
- `contrib/install/install.sh`: 선택 자산의 checksum 행만 추출하고 macOS `shasum -a 256` fallback 제공
- `scripts/tests/test_release_installers_workflow.py`: immutable source와 portable checksum 계약
- `.github/workflows/ci.yml`: 새 계약 테스트 배선

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `python -m unittest scripts.tests.test_release_installers_workflow scripts.tests.test_workflow_contract_wiring -v` | 5 / 5 통과 |
| commit의 `contrib/install/install.sh` 출력을 `bash -n`으로 검사 | 통과 |
| `git diff --check origin/pr/4376..b1bb5a56a15aa06f84ee99eeecdfb208d5cc406b` | 통과 |
| commit graph | correction commit의 유일한 parent가 contributor head와 일치 |

로컬에는 `actionlint`, macOS, WiX 및 publishing credentials가 없다. 실제 deb/rpm/MSI build,
macOS installer smoke와 crates.io dry-run은 최신 GitHub-hosted run에서 확인해야 한다.

## 최종 권고

**메인터너 보정 포함 조건부 수용 권고.** 기존 녹색 CI는 installer workflow를 실행하지 않았고,
새 correction은 workflow/test/script를 바꿨다. push 승인 뒤 correction과 review-doc commit을
fast-forward로 반영하고 최신 Full CI 및 별도 Release Installers 검증을 확인해야 한다.
required checks와 mergeability가 성공해도 별도 merge 승인 전에는 publish나 merge를 수행하지 않는다.

실행 및 rollback은 [PR #4376 구현·통합 계획](pr_4376_review_impl.md)을 따른다.
