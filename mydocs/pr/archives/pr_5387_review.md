---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5387 검토 - HWP5 편집 왕복 IrDiff-0

## 접수 메타데이터

| 항목 | 검토 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5387](https://github.com/edwardkim/rhwp/pull/5387) / kevin9327 |
| base / contributor head | devel / 3763ba2d67f8c42c476a91d8a1971bb327c47b4d |
| 가시성 branch | review/kevin9327-20260818-r1 |
| local cherry-pick | 98d6fc59a91261c7f309528b62fae3faf0cea564 |
| 원격 상태 | OPEN, 비 draft, MERGEABLE, BLOCKED |
| 검토 기준 | upstream/devel 0bc05ef81 이후 누적 branch |

## 변경 범위

HWP5 편집 왕복의 IrDiff-0 property test를 추가했다.

PR 고유 diff는 upstream/devel...review/kevin9327-20260818-r1 기준으로 확인했고, contributor
commit은 rewrite하지 않았다. 체리픽 순서는 5210부터 5387까지의 의존 순서를 따랐다.

## 충돌 및 메인터너 보정

- conflict: Cargo.toml의 M04 주석·proptest 의존성과 fuzz/README.md를 통합 정리했다.
- 보정: 기능 범위 밖 보정 없음.
- 공통 통합 정리 commit: 5b9599ffb98c5dcc72c0e73937df076ea1e77031
- 공통 정합화 commit: 5f0d7e0a8482617747bdc7d7df57e98e6717a02c

## 검증

- focused: prop_hwp5_roundtrip 6/6
- 통합 branch 기준 Python 전체: 1737 tests, 1 skipped, OK
- Rust unit tiers/manifest check: 4225 tests, 298 modules, 6559 nextest minimum cases
- cargo fmt --all -- --check 및 git diff --check 통과
- GitHub 상태는 2026-08-18 수집 기준 OPEN·비 draft·devel·MERGEABLE·BLOCKED이며 merge 전 최신 head와 required check를 다시 확인한다

## 판정

로컬 통합 검증에서 이 PR에 대한 차단 결함은 발견하지 못했다. 메인터너 보정을 포함한 현재
review branch의 수용 후보로 기록한다. 원격 PR의 최신 head, required GitHub checks, reviewer 승인과
mergeability는 문서 작성 시점 참고값이므로 merge 직전에 다시 확인해야 한다.
