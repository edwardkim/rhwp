# #3790 재착수 수행계획서 — devel 전용 CI 영향도 정책 정상화

- Issue: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- 작성: 2026-09-14 KST
- 상태: **2026-09-14 수행계획 승인 완료. [N1 조사](../working/task_m100_3790_normalize_n1.md) 완료,
  [구현계획](task_m100_3790_normalize_impl.md) 승인 대기. 구현 미착수.**
- 브랜치: `task_m100_3790_normalize`
- 기준: 최신 `upstream/devel` = `922946438fd89346a022df82e76648f88472f0cc`
- 운영 비교 기준 `upstream/main`: `cac9b4f7cc743535cd7c00fe4f286abd67e7145b`
- 착수 시 로컬 devel은 clean하고 원격 devel과 동일했다. 기존 작업공간에서 분기했으며 다른
  작업의 `rhwp-review-7040` worktree는 변경하지 않았다.
- 기존 assignee `postmelee`는 유지하고, 메인테이너의 명시적 재착수 지시에 따라 `edwardkim`을
  추가한 뒤 API로 두 계정을 확인했다. 관련 열린 PR 검색 결과는 0건이다.
- 기존 [수행계획](archives/task_m100_3790.md)과 [구현계획](archives/task_m100_3790_impl.md)은
  당시 구현·판정 기록으로 보존한다. 이번 문서는 메인테이너의 새 운영 경계와 정상화 범위만 다룬다.

## 1. 목적과 확정 정책

원래 목적은 **devel 개발 과정에서 필요한 검증은 유지하면서 영향 없는 CI 실행과 불필요한
재실행을 줄이는 것**이다. Classifier는 필요한 검사를 선택하며 Controller는 그 검사가
실제 수행됐는지 독립적으로 확인한다. Controller 자체를 비용 절감기의 전부로 해석하지 않는다.

이번 세션에서 작업지시자가 확정한 정책은 다음과 같다.

1. CI Impact Policy Controller의 대상은 **base가 devel인 PR**이다. PR source branch 이름을
   devel로 제한하는 의미가 아니며 정상 contributor/fork PR도 선택 실행 대상이다.
2. **devel → main PR 및 병합 후 main에서는 Controller 감사를 실행하거나 해당 정책 상태를
   게시하지 않는다.** `pull_request_target`와 CI 완료 `workflow_run` 우회 진입을 모두 점검한다.
3. 앞서 논의했던 devel → main 전용 advisory 방침은 위 결정으로 대체한다.
4. `CI Impact Policy`는 advisory를 유지한다. required status 추가·branch protection 변경은
   이번 범위가 아니며 현재 required `Build & Test`는 유지한다.
5. #7070의 병합 후 중복 검증 제거를 유지한다. devel/main push에서 증거 부재를 이유로 전체 CI를
   다시 시작하지 않는다. devel duration 메타데이터 갱신과 issue close 자동화는 보존한다.
6. main·태그·릴리즈의 기존 검증과 `Workflow promotion preflight`는 제거하지 않는다.
   이번 정책 제외를 릴리즈 검사 전체 생략으로 해석하지 않는다.

## 2. 재사용할 조사 근거와 현재 결함

기존 조사를 전수 반복하지 않고 아래 근거를 최신 구현과 연결해 최소 재현부터 고정한다.

| 항목 | 확보된 근거 | 이번 처리 |
| --- | --- | --- |
| 선택 실행의 원래 효과 | 기존 계획의 동일 SHA canary 및 [최근 frontend CI](https://github.com/edwardkim/rhwp/actions/runs/34800361317)에서 Rust·Native job 생략 | 정상 동작을 보존하는 대조군으로 사용. 과거 절감률을 현재 전체 PR의 절감률로 인용하지 않음 |
| 발행·소비 버전 불일치 | `scripts/ci-impact-policy.cjs`는 v6, CI·CodeQL·Render Diff의 trusted reuse는 v5만 허용 | 계약 일치 및 실제 발행 결과를 소비자가 읽는 연결 테스트 |
| 버전 회귀 기점 | `d959ac732951b17f8a42a4c53f4f22cc31b7f688`에서 v5→v6 변경 | PR 전체 workflow 변경 뒤 review-only tail의 재사용을 복원. 일반 frontend 선택 실행 전체가 실패했다는 주장은 하지 않음 |
| 검출하지 못한 테스트 | `test_ci_impact_policy_workflow.py`도 v5 문자열을 기대 | 문자열 mirror만으로 완료하지 않음. 정상/위조/구버전 상태를 실제 소비 함수로 검증 |
| 실행끼리 취소 | [34799585601](https://github.com/edwardkim/rhwp/actions/runs/34799585601)은 같은 head concurrency의 다른 요청으로 취소됨 | #6819/#6820의 기존 devel 수정을 재사용하고 동시 완료·새 head 경계를 검증 |
| 수정 코드와 운영 배선의 차이 | devel은 PR 초기 이벤트만 취소 허용, 현재 main은 무조건 취소. policy helper는 live base 로드 | 기존 구현을 다시 만들지 않고 배포 전/후 상태를 구분 |
| 비동기 증거 수렴 | [#7069 보고서](../report/task_m100_7069_report.md), 보존된 API fixture 및 bounded collector | pending/failure 구분·재조회 상한·부분 재실행 계약 보존 |
| 병합 후 중복 검증 제거 | [#7070 보고서](../report/task_m100_7070_report.md) | push full fallback을 되살리지 않음 |

선행 조사에서 actual `statusDescription()` 출력과 workflow에서 추출한 소비 함수에 정상 모의 API
응답을 제공했을 때, 세 소비자는 v6을 거절하고 v5 대조군을 수용했다. 이것은 **로컬 계약 재현**이지
실제 GitHub의 재사용 성공 증거가 아니다. 선행 Node 96개·Python 93개 통과는 연결 테스트 부족을
드러낸 자료이며, 이번 변경 뒤 통과로 재사용하지 않는다.

현재 workflow의 `workflow_run` 진입은 모든 대상 CI 완료를 수신하고 resolve에서 devel PR을 찾는다.
따라서 기존 코드가 main에 정책 상태를 잘못 게시했다고 단정하지 않는다. 불필요한 job 진입,
PR base 변경, 이벤트의 PR 정보 누락 및 실제 live PR 확인까지 따로 검증한다.

## 3. 범위와 보호 불변식

예상 변경 표면:

- `.github/workflows/ci-impact-policy.yml`: devel PR 식별·event/job 실행 경계·기존 concurrency 연결.
- `scripts/ci-impact-policy.cjs`와 CI·CodeQL·Render Diff의 trusted reuse 소비 경로:
  버전/필드/권한/증거 계약 정합. 공유화 여부는 신뢰된 base 로드 가능성을 확인한 뒤 결정한다.
- 관련 `scripts/tests/`의 Node/Python 계약·workflow 테스트 및 필요한 최소 fixture.
- `mydocs/manual/github_operations.md`와 필요한 재사용 운영 문서의 정책 설명.

보존할 조건:

- PR head 코드나 PR artifact를 privileged controller에서 실행하지 않는다.
- 정확한 repository·PR·head·base SHA·workflow identity·attempt와 status 발행자를 검증한다.
- 정상 fork PR의 selective CI는 유지하지만, same-repository 전용 trusted reuse를 fork로 넓히지 않는다.
- 필요한 검사 실패·누락·잘못된 skip을 success로 바꾸지 않는다. 불완전한 API 자료는 성공으로
  합성하지 않고 기존 pending/실패 및 제한된 재조회 계약을 따른다.
- 미분류·불완전 변경 목록에서 full을 선택하는 안전성은 유지한다. 단, **Controller 적용 대상이
  아닌 main 이벤트를 full fallback 대상으로 취급하지 않는다.**
- 기존 #6899 상세 실패 보고를 유지한다. 실제 검증 실패, controller 내부 실패, 증거 수렴 대기,
  적용 대상 아님을 구분한다.

비범위: Rust/renderer/Studio 기능, 테스트 shard·cache backend·runner 변경, timeout 일괄 확대,
새 자식 이슈, draft 경량화, artifact retry 확장, branch protection·required check 변경,
main 직접 push·별도 릴리즈·자동 workflow 재실행 루프.

## 4. 수행 순서와 승인 게이트

| 단계 | 수행 내용 | 종료 조건 |
| --- | --- | --- |
| N1 조사·구현 설계 | 기존 재현 고정, event→live PR→감사→status→소비자 연결과 main 배포 차이 확인 | 한 개 구현계획서에 파일별 최소 변경·반례·활성화 조건을 제시하고 승인 요청 |
| N2 구현·집중 검증 | devel 경계, 발행/소비 계약, 기존 취소·수렴 보정 연결 및 회귀 테스트 | 아래 매트릭스 통과, 기존 selective와 실패 차단 보존, 단계 보고 |
| N3 통합 검증·PR 준비 | 전체 관련 Node/Python·YAML 검증, 비용/증거 재사용 점검, 최종 diff·복구 방법 정리 | 정확한 head의 로컬 검증 완료 후 push·PR 승인 요청 |
| N4 적용 확인 | 승인된 PR의 CI·self-review·병합, 실제 devel 대상 동작 확인 및 main 운영 배선 반영 여부 추적 | 구현 병합과 운영 활성화를 분리해서 보고. main 배선 미반영이면 운영 완료·이슈 close로 간주하지 않음 |

현재 승인은 수행계획과 N1 진행까지다. 제품/CI 구현은 구현계획 승인 뒤 시작한다.
단계 변경 전 기록을 커밋하며, 원격 push·PR·댓글·merge·실행 요청은 해당 승인 후에만 수행한다.
오늘할일·PR review 문서는 최종 제출 절차 시점에 작성한다.

## 5. 검증 매트릭스

| 입력/상황 | 기대 결과 |
| --- | --- |
| same-repository 및 fork → devel PR | 필요한 축만 실행하고 Controller가 정확한 PR/head/base를 감사 |
| devel → main 또는 다른 branch → main PR | Controller 감사·policy 상태 게시 없음. 기존 main CI는 유지 |
| main PR의 CI·CodeQL·Render Diff 완료 | Controller 감사·게시로 우회하지 않음 |
| main/devel branch push·tag·manual 등 비대상 완료 | Controller 대상 아님. 기존 목적별 workflow는 유지 |
| `workflow_run` PR 연결 목록 없음·복수·stale 및 PR retarget | head branch 이름으로 base를 추정하지 않음. 검증된 live devel PR만 활성화 |
| 유효한 최신 발행 상태 + 정상 trusted run | 세 consumer 모두 동일 계약으로 정상 재사용 |
| 위조 발행자·다른 base/head/repo·미지원 버전·잘린 상태·실패 candidate | 재사용 거절. 대상 devel PR에서 필요한 full 검증 유지 |
| 일반 code PR의 review-only tail | 기존 fast-pass 보존 |
| workflow PR의 review-only tail | 정확한 Full candidate·계보가 증명된 범위만 trusted 재사용 |
| 같은 head의 완료 이벤트 경합·새 PR head | 완료 감사끼리 정상 실행을 취소하지 않음. stale 결과가 최신 결과를 덮어쓰지 않음 |
| successful run의 nonterminal jobs·부분 재실행 | #7069의 수렴·attempt 계약 유지. 완료 실패를 성공으로 바꾸지 않음 |
| merge 후 증거 부족 | 전체 CI 재실행이 아니라 기존 정책에 따른 메타데이터 갱신 보류 |

`workflow_run.branches`를 PR base 필터로 오용하지 않는다. GitHub 이벤트가 workflow 기록을
만드는 것, job이 skip되는 것, runner에서 resolve만 하는 것, 실제 감사·상태를 게시하는 것을
각각 기록한다. **main에 workflow run 기록 자체가 전혀 생기지 않는다고 미리 보장하지 않는다.**
N1에서 현재 trigger 방식으로 가능한 가장 이른 차단 지점과 남는 최소 비용을 제시한다.
요구한 비동작을 충족하려면 이벤트 구조 변경이 필요한 경우 구현 전에 승인받는다.

로컬 기본 검증은 관련 classifier/policy/evidence/report Node tests와 CI/Controller/CodeQL/
Render Diff/review-only Python tests, YAML 검사, `git diff --check`다. 정확한 실행 파일과 명령은
N1에서 현재 registry와 일치시킨다. CI 파일 변경이라는 이유만으로 Cargo·WASM 전체를 실행하지 않는다.

## 6. 효과 측정·활성화·복구

- 기존 historical fixture와 최근 실행 증거를 우선 재사용한다. 측정을 위해 자동으로 canary PR,
  Full CI 또는 릴리즈를 생성하지 않는다.
- 동일 논리 candidate와 review-only tail에서 예상/실제 heavy job 실행 수, fast-pass 이유,
  controller 횟수·취소·API 요청·wall time 및 job 실행시간 합계를 비교한다.
- 로컬 모의 재현 수치와 GitHub runner 실측을 구분하고 cache·대기·runner 편차를 기록한다.
  비교 가능한 실측이 없으면 절감률은 미측정으로 남긴다.
- main은 릴리즈 보관 브랜치다. default-branch 등록 workflow 파일이 main에 존재하는 것과
  **감사 대상이 main인 것**은 다르다. main의 등록 파일을 삭제하거나 default branch를 바꾸지 않는다.
- devel 병합으로 live base helper가 바뀌는 부분과 정상 devel → main 승격 뒤 활성화되는 YAML
  배선을 나누어 기록한다. 기존 #6819/#7069 보정도 이 구분을 따른다.
- main 배선의 적용 시점은 정규 릴리즈 절차와 별도 승인에 따른다. 이를 앞당기려고 main 직접 push,
  배포 PR 생성 또는 branch protection 변경을 수행하지 않는다.
- 부작용 시 이번 변경 commit의 최소 revert를 PR로 검증한다. 보호 규칙을 끄거나 검증 생략으로
  복구하지 않는다. 배포된 main에 대한 복구는 작업지시자의 별도 운영 승인을 받는다.

## 7. 수행계획 승인 결과

**devel 전용 대상 경계 + trusted reuse 계약 복원 + 기존 concurrency/수렴 보정의 연결 검증**으로
이번 범위를 제한한다. 수행계획은 승인되었고 N1에서 구현계획을 작성했다. 완료된 선행 개선을 다시 구현하거나
이번 계획에 없는 CI 최적화를 추가하지 않는다.
