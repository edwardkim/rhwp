---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5360 검토 - 한컴 PDF 오라클 쌍 매니페스트

## 접수 메타데이터

| 항목 | 검토 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5360](https://github.com/edwardkim/rhwp/pull/5360) / kevin9327 |
| base / contributor head | devel / 7e60a409bd96ef2c231fb81c8fbb34c992ec72ab |
| 가시성 branch | review/kevin9327-20260818-r1 |
| local cherry-pick | 8e28a9d569f7965aff77262a066a3d24f902c4bd |
| 원격 상태 | OPEN, 비 draft, MERGEABLE, BLOCKED |
| 검토 기준 | upstream/devel 0bc05ef81 이후 누적 branch |

## 변경 범위

한컴 PDF 오라클 쌍 매니페스트 생성기와 검증 경로를 추가했다.

PR 고유 diff는 upstream/devel...review/kevin9327-20260818-r1 기준으로 확인했고, contributor
commit은 rewrite하지 않았다. 체리픽 순서는 5210부터 5360까지의 의존 순서를 따랐다.

## 충돌 및 메인터너 보정

- conflict: tools/oracle_public/README.md add/add는 기존 page-smoke와 M01-1 문서를 함께 보존했다.
- 보정: multi PDF 편입에 맞춰 expected manifest count를 재생성했다.
- 공통 통합 정리 commit: 5b9599ffb98c5dcc72c0e73937df076ea1e77031
- 공통 정합화 commit: 5f0d7e0a8482617747bdc7d7df57e98e6717a02c

## 검증

- focused: tools.oracle_public.tests.test_oracle_resolver, tools.oracle_public.test_coverage_report
- 통합 branch 기준 Python 전체: 1737 tests, 1 skipped, OK
- Rust unit tiers/manifest check: 4225 tests, 298 modules, 6559 nextest minimum cases
- cargo fmt --all -- --check 및 git diff --check 통과
- GitHub 상태는 2026-08-18 수집 기준 OPEN·비 draft·devel·MERGEABLE·BLOCKED이며 merge 전 최신 head와 required check를 다시 확인한다

## 판정

로컬 통합 검증에서 이 PR에 대한 차단 결함은 발견하지 못했다. 메인터너 보정을 포함한 현재
review branch의 수용 후보로 기록한다. 원격 PR의 최신 head, required GitHub checks, reviewer 승인과
mergeability는 문서 작성 시점 참고값이므로 merge 직전에 다시 확인해야 한다.
