---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# PR #7071 — CI 증거 수렴과 병합 후 duration 전용 처리 self-review

**최종 판정: 승인.** 구현과 로컬 검증의 판정이며 GitHub 병합을 뜻하지 않는다.
최종 head의 PR CI와 별도 병합 판단이 남아 있다.

| 항목 | 확인 내용 |
| --- | --- |
| PR | [#7071](https://github.com/edwardkim/rhwp/pull/7071) |
| 관련 issue | Closes [#7069](https://github.com/edwardkim/rhwp/issues/7069), Closes [#7070](https://github.com/edwardkim/rhwp/issues/7070) |
| 작성자 / 검토자 | jangster77 / jangster77 self-review, reviewer 미지정 |
| base / branch | devel / fix/7069-ci-evidence-convergence |
| 최초 검증 후보 | `4855fffdba4d7985200fbe6ea0d90f3bb4758316` |
| 기준 / 병합 simulation | `3f34869b9c4d15a27b181dd22c63cd0a3730d46a` / tree `044e1f598878029f00d4e0bc2afa569cb4c8bb61`, 충돌 없음 |
| 최초 규모 | 28 files, +1309 / -295; 본 문서와 오늘할일은 문서-only 후행 변경 |
| 생성 직후 상태 | Open, non-draft, MERGEABLE / BLOCKED, PR CI 진행 중인 시점의 참고값 |
| base route | collaborator_self_merge |
| modifiers | intake_and_review, local_validation, review_only_fast_pass, rework_and_exceptions |

## 범위·원인·코드 검토

1,000줄 초과 변경이므로 [#7069 계획](../../plans/task_m100_7069.md)과
[#7070 계획](../../plans/task_m100_7070.md)을 나눠 구현·검증·커밋했다.
[source-side 보고서](../../report/task_m100_7069_report.md)와
[duration 보고서](../../report/task_m100_7070_report.md)에 실제 API, 원인과 적용 경계를 남겼다.

#7069는 성공 run 아래 미완료 job/step을 failure와 구분하고, 같은 run의 API 전후 snapshot을
대조하여 제한적으로 재조회한다. job 성공을 step 성공으로 합성하지 않는다. identity 변경,
완료 실패·취소·잘못된 skip·누락·중복에 대한 방어는 유지한다. 수렴하지 않으면 pending으로
남기고 해당 policy audit 재실행을 안내한다. 자동 재실행 루프와 추가 쓰기 권한은 없다.

#7070은 검증 CI의 branch push 시작 경로 자체를 제거한다. devel의 duration 전용 workflow는
병합된 코드만 실행하고, PR artifact는 엄격히 검증한 JSON 자료로만 소비한다. 증거 부족은
메타데이터 갱신 보류이며 전체 검증 fallback이 없다. 모든 workflow의 devel push 구독을
allowlist 검사해 새 경로가 생기면 테스트가 실패한다.

#7068의 latest Jobs API가 원래 실행된 B/C/D 작업을 새 ID/attempt로 복사한 점을 실제 자료로
확인했다. 원본 attempt API와 실행 시각까지 동일할 때만 artifact의 실제 측정 attempt를 쓴다.
새로 실행한 worker를 옛 성공 worker로 대체하지 않는다. duration 자료는 실행 시간 추정용이며
PR 승인·병합 tree 검증의 대체물이 아니다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| Node classifier/policy/evidence/duration/reuse | 450 PASS, exit 0 |
| Python workflow 계약 | 240 PASS, exit 0 |
| actionlint 구조·expression | 변경 workflow 7개 통과 |
| ShellCheck 포함 비교 | 기존 CI·Oracle SC2016 2건 동일; 신규 duration workflow 없음 |
| 실제 #7068 artifact read-only 수집 | run `34702678657`, latest attempt 2, 측정 B/C/D attempt 1, provenance 통과 |
| 기존 refresh script의 로컬 갱신 | 42 target, 원격 metrics 쓰기 미실행 |
| 변경 문서 링크·canonical/guide metadata | 통과 |
| 공백 검사 / 병합 simulation | 통과 / 충돌 없음 |

새 회귀는 실제 API 최소 응답, 수렴·미수렴, 부분 재실행·혼합 attempt·복사 job, 실행 시각 변경,
identity 불일치, 자료 누락·중복·만료·과대 크기, run 중간 변경과 code/review-only 후보를 포함한다.
원본 ZIP 검증은 경로·키·Unicode 이름·수치·target 소유 관계 보호를 유지한다.
Rust 및 renderer 변경이 없으므로 Cargo, HWP/HWPX/PDF fixture, visual sweep은 비해당이다.

## 공통 검토 원칙

- 구현 근거·일반성: 실제 GitHub API와 workflow trigger를 근거로 하며 PR 번호별 분기는 없다.
- 측정·배치·줄 소속·점유 높이·좌표계: 렌더러 조판 변경이 없어 비해당.
- 사례와 증거 독립성: 실제 API/ZIP read-only 검증과 synthetic 반례를 구분했다.
- 기준값·허용치: golden/baseline 수정과 검증 완화 없음. 미완료 자료는 승인하지 않는다.
- 주장 범위: 로컬 검사와 운영 활성화를 구분한다. 미실행 원격 갱신이나 새 정책의 병합 후 관찰을 완료로 표시하지 않는다.

## 병합 후 확인할 항목

최신 head CI 통과와 병합 판단 뒤 merge SHA에 검증 workflow가 새로 시작하지 않는지,
`Refresh nextest target duration data` 결과와 #7069/#7070 close 상태를 확인한다.
main/devel push 제거와 duration workflow는 devel 병합부터 적용되지만, #7069 controller 수집
배선은 기본 branch main의 정상 release 이후 활성화된다. main 직접 push는 범위에 없다.
코멘트·devel 동기화 후 이 PR 소유 branch/worktree만 정리하며 공유 target과 다른 작업은 보존한다.

## 최초 CI 실패 보정

`5a5095d58`의 lint에서 새 Node evidence 테스트의 discovery 기대 목록 누락을 발견했다.
`test_workflow_contract_wiring.py`를 갱신하고 실제 CI의 `Validate workflow contracts` 명령을
그대로 재실행해 exit 0을 확인했다. Python 검색 범위는 `test_*workflow*.py`로 넓혀 240 PASS를
확인했다. 이후 변경은 workflow 계약 테스트 2개와 본 검증 기록이며 제품 코드 변경은 없다.
새 최종 head의 CI 결과로 병합 판단해야 한다.
