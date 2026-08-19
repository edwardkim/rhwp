---
kind: pr-review
status: approved-pending-ci
pr: 5617
author: jangster77
base: devel
---

# PR #5617 검토: planet6897 기여 변경 6건을 통합한다

PR: [#5617](https://github.com/edwardkim/rhwp/pull/5617)  
브랜치: `review/planet6897-excluding-5610-20260819`  
기준: 최신 `upstream/devel`

## 범위와 계보

| 원본 PR | 원본 head | 판정 |
| --- | --- | --- |
| #5580 | `b95bda5d84dafd1d69009fa29893c8c23ff2ed7c` | 승인 |
| #5591 | `f98e743876521366c3ae7109ed4e7b3e083009d2` | 승인 |
| #5594 | `4ea70b285c5d780e5d218f58aeca58564ffbb104` | 승인 |
| #5604 | `db80a116d8f6fb21605d1838c10ca7d551e1228f` | 승인 |
| #5608 | `310c37b27aa4e652863d5bb492aa60242e8567ea` | 승인 |
| #5609 | `a4d01b9a777dfba80f37f48cf061994bfc847302` | 승인, test 정책 보정 |

PR #5610에 이미 통합된 #5544, #5552, #5559, #5560, #5562, #5564, #5565, #5567, #5574, #5577은 의도적으로 제외했다.

## 검토 결론

**승인, CI 대기.** 구현 결함은 발견하지 못했다. #5609의 source-side 테스트 정책 위반은 integration contract로 이관해 보정했으며, 원본 기능 커밋의 author·출처는 보존했다.

## 로컬 검증

- `cargo fmt --all -- --check`: 통과
- `node scripts/rust-unit-test-tiers.mjs --check`: 통과
- full integration nextest: 7,782 passed, 38 skipped, 4분 48초
- native-Skia lib: 58 passed
- Studio unit: 991 passed, 1 skipped
- Studio production build: 통과
- native-Skia PNG fixture: rotated picture와 square table 확인

## 병합 조건과 후속 처리

최신 head의 필수 CI가 모두 성공하면 병합한다. 병합 후에는 원본 PR별 review 기록에 따라 원본 PR #5580, #5591, #5594, #5604, #5608, #5609와 해당 완료 issue를 댓글 후 close하고, 통합 브랜치 및 review worktree를 표준 절차로 정리한다.
