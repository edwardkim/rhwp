---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6634.md
issue: 6634
last_verified: 2026-09-05
---

# #6634 Stage 1 완료보고 — 원인 계보와 RED 계약

## 수행 결과

v0.8.0~v0.8.4와 v0.8.6의 Release API, Release Binary run, Publish All Packages run을 다시 조회해
정규화 fixture와 [원인 계보](../tech/investigations/issue-6634/release_publish_causal_lineage.md)로
보존했다.

확정된 결함은 세 가지다.

1. v0.8.4·v0.8.6은 `GITHUB_TOKEN`이 draft를 게시해 `release.published` run이 억제됐다.
2. v0.8.0~v0.8.3의 package run은 binary attachment보다 먼저 시작해 5-platform gate를 우회했다.
3. v0.8.6 수동 복구는 tag `f1f9c6ae...`가 아니라 후속 main hotfix `e8800c8d...`에서 실행됐다.

## RED 계약

`scripts/tests/test_release_publish_orchestration.py`는 historical evidence 4건과 목표 workflow 계약 8건을
분리한다.

| 구분 | 결과 | 의미 |
| --- | --- | --- |
| 계보 fixture 4건 | PASS | release 두 군의 시간축과 v0.8.6 SHA 이탈이 내부 일관성을 가짐 |
| 간접 release event 제거 | RED | 현재 `release.published`에 의존 |
| reusable workflow | RED | `workflow_call` 없음 |
| 수동 dispatch 안전 기본값 | RED | 실제 publish와 verify mode가 분리되지 않음 |
| binary 뒤 직접 호출 | RED | caller job 없음 |
| exact release source guard | RED | guard와 validation job 없음 |
| extension 채널 분리 | RED | VS Code/Open VSX가 한 직렬 job에 결합 |
| publish aggregate | RED | 외부 채널 전체 완료를 나타내는 명시적 gate 없음 |
| promotion policy | RED | 두 release workflow가 #6689 policy에 없음 |

예상 결과는 **12건 중 4 PASS, 8 RED**다. RED는 fixture 오류나 YAML parse 실패가 아니라 아직 구현되지
않은 승인 계획의 경계에서만 발생해야 한다.

실제 실행 결과도 12건 중 4 PASS, 8 RED로 정확히 일치했다. 기존 release channel·workflow promotion
회귀는 43건 모두 통과했다. 첫 회귀 명령에서 존재하지 않는
`scripts.tests.test_workflow_promotion_inventory`를 잘못 지정해 import error 1건이 발생했으나, 실제
파일 목록의 `test_workflow_promotion_preflight`, `test_workflow_promotion_evidence`,
`test_workflow_promotion_gate_workflow`로 즉시 정정했다. 정정 실행에는 import error나 제품 실패가 없다.

## 범위와 안전

- workflow와 publish script는 아직 수정하지 않았다.
- workflow dispatch, tag, Release, package·extension publish는 수행하지 않았다.
- secret은 이름과 배선만 확인했고 fixture·문서·test에 값을 넣지 않았다.
- Stage 2에서 reusable caller와 exact source guard를 먼저 GREEN으로 전환한다. 채널 상태·부분 재시도
  구현은 승인된 Stage 3 경계를 유지한다.

## 다음 게이트

메인테이너가 Stage 1 결과와 4 PASS/8 RED를 승인하면 Stage 2 reusable 호출·exact release guard 구현에
진입한다.
