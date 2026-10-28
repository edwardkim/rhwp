---
pr: 6257
issue: 6256
author: jangster77
base: devel
head: codex/6256-squash-codeql-reuse
code_candidate_sha: 59c06a4ed92a65aace886d0cd53bb5fd89639787
created_at: 2026-08-28
---

# PR #6257 검토 기록

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#6257](https://github.com/edwardkim/rhwp/pull/6257) |
| 관련 이슈 | [#6256](https://github.com/edwardkim/rhwp/issues/6256) |
| 작성자 | jangster77 self-review |
| base / head | devel / codex/6256-squash-codeql-reuse |
| 코드 후보 SHA | 59c06a4ed92a65aace886d0cd53bb5fd89639787 |
| 변경 규모 | 11 files, +255 / -17 |
| 작성 시점 상태 | MERGEABLE, required check 대기로 BLOCKED |

상태값은 문서 작성 시점 참고값이다. merge 판단 전에는 review 기록 commit을 포함한 최신
head의 mergeability와 GitHub Actions 결과를 다시 확인한다.

## 변경 범위와 판단

- reusable trusted post-merge workflow가 caller의 원래 event, ref, SHA를 받도록 변경한다.
- 단일 부모 squash commit은 동일 저장소의 단일 merged PR, PR head와 같은 결과 tree,
  성공한 정확한 PR workflow가 모두 확인될 때만 worker 재사용을 허용한다.
- direct push, merge queue, 모호한 PR 연결, tree 불일치, enforcement surface 변경,
  missing/pending/failed candidate는 full CI로 fail-closed한다.
- evaluator의 기존 및 squash 계약을 CI workflow contracts 단계에 배선하고, 해당 배선의
  존재를 Python workflow 계약으로 고정한다.

변경은 CI 정책, JavaScript/Python 계약 테스트, 검증 가이드에 한정된다. renderer, HWP/HWPX,
PDF fixture, 제품 출력은 변경하지 않으므로 visual sweep 증적은 대상이 아니다.

## 로컬 검증

다음 계약 검증을 코드 후보 SHA에서 완료했다.

| 명령 | 결과 |
| --- | --- |
| node --test ci-impact-classifier, ci-impact-policy, 기존/squash trusted reuse evaluator | 70 passed |
| python3 -m unittest discover -s scripts/tests -p test_*workflow.py | 166 passed |
| git diff --check | passed |

전체 Studio Node glob은 설치되지 않은 @noble/hashes와 OS resource-limit 테스트처럼 이 PR 범위와
무관한 환경 의존 검사를 포함하므로, CI topology 회귀 gate로 사용하지 않았다. 대신 이 PR이 변경한
CI policy, trusted reuse evaluator, 모든 Python workflow contract를 명시적으로 실행했다.

## CI 및 merge 조건

1. review 기록 commit을 포함한 최신 PR head의 required checks가 성공해야 한다.
2. CI workflow와 CodeQL workflow가 full lane에서 정책 변경을 검증해야 한다.
3. merge 뒤 devel push에서 trusted post-merge reuse가 squash merge PR을 정확히 식별하고,
   검증된 PR run을 재사용하는지 확인한다.
4. direct push 또는 evidence가 불완전한 squash merge가 full lane으로 fail-closed하는 기존 보안
   성질은 유지되어야 한다.

## 최종 권고

**CI 대기.** 로컬 계약 검증은 통과했다. 최신 PR head의 GitHub Actions와 squash merge 뒤
devel run에서의 실제 재사용 결과를 확인한 뒤 merge 여부를 판단한다.
