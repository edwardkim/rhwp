# #6899 Stage 1 — 실패 보고 원인 계보와 실행 경계 조사

- 일자: 2026-09-08
- 상태: 조사 완료, 구현계획 승인 대기. 제품·workflow 구현 변경 없음.
- 승인 계획 commit: `e50e32b88`.

## 1. 결론

이번 문제는 최근 보고 기능의 회귀가 아니라 최초 Controller 도입 때부터 있던 진단 기능의 부족이다.
기존 fail-closed 판정의 잘못을 뜻하지 않는다. 기존 policy 결과를 수정하지 않고 별도 진단 보고를 덧붙인다.

| 근거 | 확인 결과 |
| --- | --- |
| `git log -S 'workflow-not-success:' -- scripts/ci-impact-policy.cjs` | 최초 `86b966ac5b`, 2026-08-15, #3790/#4682 |
| `git log -S 'Summarize trusted policy' -- .github/workflows/ci-impact-policy.yml` | 동일 도입 commit |
| 해당 commit의 policy 원문 | 실패 workflow를 일반 reason으로 반환하고 job 감사 전 종료 |
| 해당 commit의 workflow 원문 | run/job의 이름·상태·결론 중심 projection, ID/attempt 없는 상태 요약 |
| 현재 코드 | 위 구조 유지. fast-pass 등 정책 필드는 추가됐지만 상세 오류 연결은 없음 |

## 2. 실제 실패 계보

- [Controller 34221894722](https://github.com/edwardkim/rhwp/actions/runs/34221894722):
  main `e8800c8def63449808a4092798442652ed460552`, workflow_run, attempt 1.
- 대상 PR #6898, head `5f2ea958f14a3b1ce13525ebc482c21a7c7e5b6d`.
- [CI 34220658247](https://github.com/edwardkim/rhwp/actions/runs/34220658247):
  pull_request, 같은 head, attempt 1.
- [job 102045013116](https://github.com/edwardkim/rhwp/actions/runs/34220658247/job/102045013116):
  `test-archive-b-shard-1 / Default-feature tests (Archive B)`, 실패 step 6 `Run Archive B`.
- `text_overlaps_do_not_grow_partition_13`: 신규 issue6879 샘플에서 겹침 1건, baseline 없음.
- worker exit 100 → Build & Test 실패 → Controller가 failure status를 게시하고 의도적으로 job을 실패 처리.
- Controller API 게시 실패가 아니다. 해당 샘플은 PR에서 추가됐으므로 기존 미해결/신규 회귀 분류는 별도 비교가 필요하다.
- check annotation은 `Process completed with exit code 100.`만 제공한다. 테스트명과 겹침 이유는 job 로그에 있다.
  따라서 annotation만 연결하면 이번 사례의 핵심 오류 보고가 충족되지 않는다.

## 3. 데이터가 사라지는 지점

1. `collectRun`은 GitHub run을 선택하고 `listJobsForWorkflowRun(filter: latest)`로 job을 수집한다.
2. JSON projection에 run ID/attempt, job ID/attempt, step number가 빠진다.
3. policy가 workflow failure를 먼저 반환한다. 이후 `auditCi` 등은 성공 workflow 내부 계약 검사용이다.
4. summary는 policy reason만 출력한다. publish는 그 reason으로 `core.setFailed`를 호출한다.

현재 수집 구조에는 job 이름·결론·step 이름·결론이 이미 있으므로 진단용 ID를 보존하는 데 추가 API는 필요 없다.
다만 `latest` job 조회와 run 선택 사이에 재실행이 시작될 가능성은 코드상 존재한다.
이번 attempt 1 사례에서 실제 혼합을 관찰한 것은 아니다. 보고는 attempt를 고정하고 불일치 시 미확인 처리해야 한다.

## 4. Controller 자체 장애 보고 경계

- resolve 단계 실패 시 `active`가 없어 기존 요약이 실행되지 않는다.
- checkout/collect/policy는 continue-on-error 경계가 있으며 기존 fallback 판정이 존재한다.
- publish가 마지막 단계이고 기존 summary보다 뒤이므로 게시 API 오류는 기존 summary에 포함할 수 없다.
- 상세 진단은 publish 뒤 별도 best-effort 단계로 둔다. 취소된 실행에서는 수행하지 않는다.
- checkout 실패로 helper가 없는 경우에는 workflow 자체의 고정 문자열 fallback으로 단계 outcome만 기록한다.
- runner 강제 종료·job 전체 timeout은 후속 단계 실행을 보장할 수 없으므로 완전 보고를 약속하지 않는다.

## 5. 배포와 원격 변화

live main SHA는 `e8800c8def...`, 조사 후 upstream/devel SHA는 `54f4a0237e5aa8c9270dd767d5d9a21b9df56bd8`이다.
현재 main/devel Controller YAML 차이는 #6819의 concurrency 주석과 조건이다.
main은 `cancel-in-progress: true`, devel은 pull_request_target만 취소하도록 되어 있다.
이를 이번 작업에서 다시 변경하거나 우회하지 않는다.

실행 YAML은 main에서 등록되고, executable helper는 live PR base SHA를 sparse checkout하여 실행한다.
새 helper는 기존 sparse 목록에 없으므로 helper를 devel에 추가하는 것만으로 live 상세 보고가 활성화되지 않는다.
main 배선 반영은 별도 승인 및 배포 절차가 필요하다. devel 구현 완료와 live 적용 완료를 구분한다.

조사 중 추가된 #6863 merge는 renderer/test/리뷰 증적이며 Controller 파일과 겹치지 않는다.
Stage 2 진입 전 최신 devel을 다시 fetch하고 병합·충돌 여부를 확인한다. 현재 작업 브랜치에 임의 병합하지 않았다.

## 6. 기준 검증

- `node --test scripts/tests/ci-impact-policy.test.cjs`: 37/37 PASS.
- `python3 -m unittest discover -s scripts/tests -p 'test_ci_impact_policy_workflow.py'`: 13/13 PASS.
- `actionlint`: 현재 PATH에서 찾지 못함. Stage 3에서 승인 범위의 검증 도구로 준비해야 한다.
- 기존 Python 검사는 summary/publish 단계 순서와 guard 문자열에도 의존한다.
  보고 단계 추가 시 기존 취소/보안 의미를 검증하는 테스트를 유지하고 단순 개수 고정만 적절히 갱신한다.

## 7. 다음 gate

[구현계획](../plans/task_m100_6899_impl.md) 승인 후 Stage 2로 진입한다.
이 조사에서는 원격 comment/push/PR/재실행·workflow 변경을 수행하지 않았다.
