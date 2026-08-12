---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4571 검토 - HWP3 음영과 HWPX 차트 변환 정합

## 라우팅

base route: `collaborator_self_merge.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `visual_fixture_evidence.md`, `rework_and_exceptions.md`

## 메타데이터와 범위

| 항목 | 값 |
| --- | --- |
| PR | [#4571](https://github.com/edwardkim/rhwp/pull/4571) |
| 작성자 | @jangster77 (collaborator) |
| base / head | `devel` / `pr/devel-johndoekim-hwp3-chart` |
| code candidate | `b26a31c6a3ea4e47a7674754f112e9c1a7eab3a3` |
| 기준 devel | `32ecfd113`을 merge한 `559e170bf` |
| 작성 시점 원격 상태 | open, mergeable `MERGEABLE`, checks 대기라 `BLOCKED` |
| 관련 원 PR | [#4366](https://github.com/edwardkim/rhwp/pull/4366), [#4499](https://github.com/edwardkim/rhwp/pull/4499) |

이 PR은 두 원 PR의 기능·테스트·계획·보고서를 최신 `devel` 위에 누적하고, HWPX 라이터가 실제
검정 음영 `0x00000000`을 `none`으로 잃던 결함을 메인터너 보정으로 추가한다. 원 PR별 검토와
시각 증적은 [#4366 검토](pr_4366_review.md), [#4499 검토](pr_4499_review.md),
[누적 적용 기록](pr_4366_4499_review_impl.md)에 archive로 보존했다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `git diff --check` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --profile release-test --all-targets -- -D warnings` | 통과 |
| HWPX 검정 음영 저장 단위 테스트 | 1 passed |
| HWP3 음영 저장 계약 | 7 passed |
| 전체 nextest | 5,730 passed, 7 slow, 36 skipped, 437.285s |
| 한컴 2020 PDF - 차트 | 원본/변환본 1쪽 144 DPI 래스터 SHA-256 동일 |
| 한컴 2020 PDF - HWP3 음영 | `SO-SUEOP.hwp` 변환본 3쪽 검정 막대 소멸 |
| archive 문서 링크·메타데이터 | 통과 |

기준 `devel`은 로드맵·문서 변경만 포함해 누적 branch에 충돌 없이 병합했다. 코드와 테스트 변경이
없으므로 기존 전체 nextest 결과는 유효하며, 최신 PR head에서는 GitHub Actions가 최종 gate다.

## reviewer와 merge 조건

PR 작성자 @jangster77은 GitHub 규칙상 자신의 PR에 review request를 보낼 수 없어 요청 API가
422로 거부됐다. 다른 사용자를 임의로 지정하지 않았다. 최신 head의 required checks와 branch
protection이 실제로 요구하는 approval을 CI 완료 뒤 재확인한다.

**권고: 최신 head의 GitHub Actions 통과 및 필요한 승인 충족 후 merge.** merge 뒤에는 #4155와
#4099 close 상태, 원 #4366·#4499 PR close/comment, `devel` sync, 정확한 branch/worktree 정리를
순서대로 수행한다.
