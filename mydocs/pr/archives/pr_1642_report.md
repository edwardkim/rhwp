# PR #1642 사전 처리 판단 보고서

- 작성일: 2026-06-29
- PR: https://github.com/edwardkim/rhwp/pull/1642
- 작성자: @planet6897
- 대상 이슈: #1636, #1637
- 판단: 문서 보정 및 리뷰 기록 push 후, 최신 CI 확인을 조건으로 approval/merge 가능

## 1. Route 판단

이 PR은 외부 contributor가 fork branch에서 작성한 PR이며, `maintainerCanModify=true`로 확인되었다.
따라서 collaborator-mediated 외부 PR 경로(Route A)를 적용한다.

적용 결과:

- 원본 PR을 그대로 merge 후보로 유지한다.
- contributor commit은 rewrite하지 않는다.
- collaborator의 문서 보정과 리뷰 기록은 PR head에 후속 커밋으로 추가한다.
- 별도 문서 PR은 만들지 않는다.

## 2. Source PR / commit provenance

문서 작성 직전 PR head 참고값:

- `0bc9a633c1c8466b570d408f4c974d620874d52f`

`upstream/devel..HEAD` 참고 commit list:

| SHA | 내용 |
|-----|------|
| `0bc9a633` | Merge branch `devel` into `task/1636-1637-pagination-fidelity` |
| `e658ee8e` | Merge branch `devel` into `task/1636-1637-pagination-fidelity` |
| `76738f98` | HWPX roundtrip IR-invisible 페이지네이션 변동 수정 + 페이지↔PI 검증 도구 (#1636, #1637) |

실제 기능 변경은 contributor commit `76738f98489920431132cf279944018dd2afef88`에 포함되어 있다.
후속 collaborator 커밋은 문서 보정과 리뷰 기록만 포함한다.

## 3. 검토 요약

PR은 HWPX roundtrip에서 IR diff가 0이어도 페이지네이션이 바뀌는 문제를 두 원인으로 나누어 수정한다.

- 원인 A: secPr visibility `hideFirstEmptyLine`이 템플릿 기본값으로 드롭됨
- 원인 B: table `hp:pos@flowWithText`가 `1`로 하드코딩되어 원본 `0`이 보존되지 않음

수정은 직렬화 출력 보존과 roundtrip diff 게이트 보강을 같이 포함한다. 검토 결과 코드 변경 자체에서
blocking 수준의 문제는 발견하지 못했다.

## 4. 검증 결과

로컬 worktree `/private/tmp/rhwp-pr1642-latest`에서 수행한 검증:

| 명령 | 결과 |
|------|------|
| `cargo test task1637 --lib` | pass |
| `cargo fmt --all --check` | pass |
| `python3 -m py_compile tools/verify_pi_page_roundtrip.py` | pass |
| `cargo test --test hwpx_roundtrip_baseline baseline_large_samples_roundtrip -- --nocapture` | pass |
| `cargo test --test hwpx_roundtrip_baseline baseline_all_samples_roundtrip -- --nocapture` | pass |
| `cargo test --test visual_roundtrip_baseline visual_baseline_all_samples -- --nocapture` | pass |
| `cargo clippy --all-targets -- -D warnings` | pass |

수동 시각 검증 산출물:

- `/private/tmp/rhwp-pr1642-review/output/pr1642_visual/compare.html`

자동 비교 기준:

- PR 전 roundtrip: `STRUCT_MISMATCH`, page 3 max displacement `622.00px`
- PR head roundtrip: `PASS`, page count `27 -> 27`, max displacement `0.00px`

## 5. Follow-up 분리

다음 두 항목은 별도 이슈로 등록하고 #1637의 sub-issue로 연결했다.

| 이슈 | 내용 | PR #1642 blocking 여부 |
|------|------|------------------------|
| #1654 | HWPX -> HWP 변환에서 `hideFirstEmptyLine` flags 동기화 검증 | non-blocking |
| #1655 | HWPX 수식 `flowWithText` roundtrip 보존 | non-blocking |

#1654는 HWPX roundtrip이 아니라 HWPX를 HWP로 내보낼 때의 `SectionDef.flags` 동기화 검증이다.
#1655는 수식 개체의 `flowWithText` 하드코딩 잔여 가능성으로, PR #1642의 표 cause B와 같은 속성 계열이지만
현재 보고된 페이지네이션 회귀의 직접 원인은 아니다.

## 6. GitHub 상태와 merge 전 조건

문서 작성 직전 PR metadata:

- `maintainerCanModify=true`
- `mergeable=MERGEABLE`
- `draft=false`
- label/assignee/review request 정렬 완료

GitHub Actions는 최신 head 기준으로 approval/merge 전 다시 확인해야 한다. 문서 전용 후속 커밋이 추가된 뒤에는
다음 중 하나가 필요하다.

- 최신 PR head 기준 relevant check 통과
- 또는 후속 커밋이 `mydocs/**` 문서만 변경하는 single-parent commit이고, 직전 코드 head의 relevant check가
  success/skipped/neutral인 fast-pass 조건 충족

## 7. Issue close 계획

PR body에 `Closes #1636`, `Closes #1637`가 있더라도 base가 `devel`이라 GitHub auto-close가 실패할 수 있다.
merge 후에는 다음을 확인한다.

- #1636 state
- #1637 state
- auto-close 실패 시 작업지시자 승인 후 수동 close comment 작성 및 close

## 8. 권고

문서 보정 커밋과 리뷰 기록 커밋을 PR head에 push한 뒤, PR comment로 검토 결과와 follow-up 분리 내용을 남긴다.
그 다음 최신 CI 또는 fast-pass 조건을 확인하고, 작업지시자 승인 후 approval review를 작성하는 순서가 적절하다.

현 시점 판단은 **merge 수용 가능**이다. 다만 approval review, merge, issue close는 이 보고서 작성 이후의
별도 명시 승인 단계로 남긴다.
