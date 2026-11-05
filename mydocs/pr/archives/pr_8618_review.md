---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-05
---

# PR #8618 검토·병합 기록 — 자유 형식 이슈 자동 라벨 분류

## 최종 판정

**승인 — 명확한 자유 형식 보고서의 보수적 분류와 기존 폼·수동 결정 보존을 검증했습니다.**
작업지시자의 명시적 승인 뒤 Ready 전환과 일반 squash merge를 완료했습니다. GitHub review의
approve 제출이나 관리자 우회는 수행하지 않았습니다. 이 기록은 workflow 변경의 옵션 2에 따른
별도 후속 기록이며 원 코드 PR에 trailing commit을 추가하지 않았습니다.

아래 head·base·CI·merge SHA는 원 코드 PR #8618의 검증 기록입니다. 이 문서 두 파일만 추가하는
후속 기록 PR의 head·preflight·병합 결과와 구분하며, 기록 PR의 실제 결과는 해당 PR 본문과
최종 보고에 남깁니다. 기록 PR 자체에 대한 재귀적인 후속 기록 PR은 만들지 않습니다.

## 대상과 구현 검토

- [PR #8618](https://github.com/edwardkim/rhwp/pull/8618), 작성자 `edwardkim`, base `devel`.
- 검토·게시·최종 head: `224b87adb8de004ebd54c548168c728d4957f7c2`.
- 병합 직전 최신 base: `ed057276c1e0e07e743218d95641e015b46b6715`. base 전진·리베이스 없음.
- squash merge: [`d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0`](https://github.com/edwardkim/rhwp/commit/d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0), `2026-11-05T03:18:49Z`.
- 원 head와 squash 결과의 전체 tree는 `c6c1898f9eefc6d06532e9d35c083e5309e16df1`로 같고,
  `git merge-base --is-ancestor <merge> upstream/devel`이 성공했습니다.
- 선행 구현은 [#7448](https://github.com/edwardkim/rhwp/pull/7448), merge `ba4fa3ccce2f1e2fe711a510bf36e04772451a4b`입니다.
  알려진 폼 구조·정확한 선택값만 처리하던 의도적 범위를 이번 명시 지시에 따라 확장했습니다.
- 변경은 `.github/workflows/issue-form-labels.yml`, `scripts/tests/issue-form-labels.test.mjs`,
  `mydocs/manual/github_operations.md`의 3개 파일입니다.
- 기존 4개 폼 × 5개 사용 경로를 유지합니다. 자유 형식은 명시적 제목 표시와 제품 경로를 사용하며,
  모호한 경로·유형은 독립적으로 보류하고 고정된 기존 라벨 11개 밖의 라벨은 만들지 않습니다.
- 기존 라벨·전체 제거 이력을 보존하고, 제목·본문·전체 라벨·상태를 쓰기 직전에 재확인합니다.
  코드 블록·인용·주석·URL은 근거에서 제외하고 원문을 실행하거나 로그에 출력하지 않습니다.
- 독립 검토에서 발견한 Visual Studio·Android Studio 오분류는 음성 회귀로 재현·보정했습니다.
  별도 `[studio]`와 `rhwp-studio/` 경로의 양성 대조군은 유지합니다.
- 완전한 폼의 미지원·복수 선택 및 중복·혼합 구조는 유형·영역 fallback도 차단합니다.
  불완전한 폼은 사용 경로를 보류하되 제목의 독립적인 유형·영역 근거는 처리합니다.

## 검증과 병합 조건

| 검증 | 결과 |
| --- | --- |
| `node --test scripts/tests/issue-form-labels.test.mjs` | 32개 통과. 원 폼·자유 형식/API 예시·모호성·수동 결정·멱등성·사유·경합·IDE 음성/양성 포함 |
| `python -m unittest scripts/tests/test_workflow_contract_wiring.py` | 6개 통과. 기존 CI 호출 유지 |
| `actionlint .github/workflows/issue-form-labels.yml` | v1.7.12 통과 |
| `git diff --check upstream/devel...HEAD`, `git diff --check` | 통과 |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/41957208427) | exact head 성공. workflow contracts, lint, frontend, Native Skia, archive A/B/C/D/E 및 Build & Test 포함 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41957208330) | exact head 성공 |
| [Proptest](https://github.com/edwardkim/rhwp/actions/runs/41957208550) | exact head 성공 |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/41957208250) | exact head 성공 |
| [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/41957208409) | exact head 성공 |

병합 직전 REST 재조회에서 36개 check가 종료됐고 실패·대기 항목은 없었습니다. 필수
`Build & Test` 성공, `mergeable=true`, `mergeable_state=clean`, 미해결 review 없음과
추가 활성 branch rule 없음을 확인했습니다. 승인 review 요구는 없었고 squash merge가 허용됐습니다.
Ready 전환 뒤에도 같은 head와 조건을 다시 확인했으며 `gh pr merge --squash --match-head-commit`
정상 경로를 사용했습니다. `--admin`, auto-merge, 보호 규칙 변경, CI 재실행은 사용하지 않았습니다.

조판 원칙·시각 검증·HWP/HWPX/PDF 입력 확인은 비해당입니다. 변경된 코드는 저장소 metadata
분류만 수행하며 제품 실행·조판·출력 경로를 바꾸지 않습니다. 테스트 입력은 커밋된 Node 테스트의
이슈 객체·Markdown 문자열이며 별도 실물 문서를 사용하지 않았습니다.

## 병합 후 관찰과 남은 경계

- [Refresh nextest target duration data](https://github.com/edwardkim/rhwp/actions/runs/41960801880) 성공.
  검증된 PR worker 실측으로 `ready=true`를 확인했고, metadata commit
  `57c76a2d81cf9397768aefdbc4e7a1300c69f3d5`를 정책 branch에 게시했습니다.
- [Close Issues on devel Push](https://github.com/edwardkim/rhwp/actions/runs/41960801883) 성공.
  merge commit의 새 Full CI·CodeQL·Adapter·Proptest 실행은 없었으며 수동 실행도 하지 않았습니다.
- 관련 [#7447](https://github.com/edwardkim/rhwp/issues/7447)은 이번 병합 전인 2026-10-23에 닫힌
  선행 이슈입니다. `Refs` 관계만 유지했고 이번 작업에서 이슈 상태를 바꾸거나 댓글을 게시하지 않았습니다.
- 기본 branch는 `main`입니다. 이번 `devel` 병합으로 새 이슈 workflow가 활성화되지는 않습니다.
  main 반영과 실제 opened/edited 이벤트 검증은 이후 별도 승인 단계입니다.
- GitHub의 최종 재조회와 라벨 추가는 원자적 연산이 아니므로 그 사이의 동시 변경을 완전히 잠그지는 못합니다.
- 이 문서와 오늘할일은 원 코드 PR의 별도 기록 PR로 반영합니다. 해당 기록 PR의 실제 preflight·required
  check와 병합 가능 상태를 확인한 뒤 정상 절차로 병합하고, 임시 branch·checkout은 운영 가이드 7.7의
  소유·최종 SHA·병합·활성 작업 조건을 확인한 뒤 정리합니다.
