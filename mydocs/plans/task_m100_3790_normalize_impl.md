# #3790 정상화 구현계획 — devel 전용 감사와 trusted reuse 계약

- Issue: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- 작성일: 2026-09-14 KST
- 상태: **N1 조사 완료, 구현 승인 대기**
- 근거: [승인된 수행계획](task_m100_3790_normalize.md), [N1 재현·검증](../working/task_m100_3790_normalize_n1.md)
- 기준 코드: `922946438fd89346a022df82e76648f88472f0cc`.

## 1. 설계 결정

새 Controller나 추가 workflow를 만들지 않는다. 기존 trusted base·advisory·선택 실행 구조를
유지하면서 **대상 식별 → 감사 → 게시 → 재사용 소비**의 계약을 일치시킨다.

### A. 대상 브랜치 판정

1. `pull_request_target.branches=[devel]`을 유지한다. job 조건에도 event PR의 base가 devel임을
   명시한다. main/devel push·tag·manual trigger를 새로 만들지 않는다.
2. `workflow_run`은 PR 실행 완료만 후보로 삼는다. 연결 PR 목록이 완전하게 대상 정보를 제공하고
   그 중 devel 대상이 없으면 감사 job을 시작하지 않는다. 목록에 devel이 있거나 정보가 부족하면
   live 식별로 넘긴다. fork의 빈 목록만으로 정상 감사를 모두 생략하지 않는다.
3. resolve는 연결 PR 번호가 있으면 그 PR을 live 재조회하고, 번호가 없으면 기존 repo/head 기반
   devel PR 검색을 사용한다. 명시된 main PR을 같은 branch의 다른 devel PR로 바꿔치기하지 않는다.
   후보가 모호하면 `.find()`로 첫 결과를 선택하지 않고 감사·게시하지 않으며 이유를 남긴다.
4. active 전의 공통 조건: open, base.ref=devel, base.repo=현재 저장소, event/run과 live head
   SHA·source repository·source branch 일치. 누락 필드는 추정하지 않는다.
5. PR 신원이 확정되지 않으면 checkout·증거 수집·감사·상태 게시를 수행하지 않는다.
   비대상, stale, 모호한 식별, API 오류는 로그에서 구별한다. 미확정을 success로 게시하지 않는다.

`workflow_run.branches`는 base 필터가 아니므로 추가하지 않는다. 이 설계는 **main 정책 감사·게시 0회**가
목표다. webhook 정보가 충분하면 job 자체를 skip하지만, 정보가 없으면 신원 확인 비용과 workflow
기록은 남을 수 있다. 모든 main 관련 workflow 기록까지 없애는 이벤트 구조 재설계는 포함하지 않는다.

### B. 게시 직전 재확인

publish 단계에 기존 계산 기준의 base ref/SHA, source repository/branch를 전달한다.
이미 수행하는 live `pulls.get` 응답으로 다음을 검증하므로 게시 전 조회 횟수를 추가하지 않는다.

- open 상태이며 base가 devel이고 base repository가 현재 저장소다.
- live head SHA·source repository/branch가 계산 입력과 같다.
- live base SHA가 계산 기준 BASE_SHA와 같다.

어느 하나라도 불일치하면 status API를 호출하지 않고 stale/비대상 이유를 남긴다. 이전 success를
지우거나 새 결과를 합성하지 않는다. 자동 재실행을 추가하지 않으며 최신 base 재검증은 기존 이벤트와
별도 승인된 운영 절차를 따른다. 기존 reporter가 이 무게시 결과를 내부 장애와 구분하는지도 검사한다.

### C. trusted reuse 발행·소비 계약

세 소비자의 v5 요구를 현재 발행 v6과 맞춘다. 상태 형식을 새로 바꾸지 않으므로 버전을 다시 올리지
않는다. 오래된 v5나 임의의 미래 버전을 함께 허용하여 우회하지 않는다.

- `hasTrustedReviewReuse(pr)` 진입 시 devel 대상·same-repository PR인지 확인한다.
  main PR에서 Controller 상태를 기다리며 polling하지 않도록 즉시 false를 반환한다.
- 기존 creator, 정확한 base SHA, rfp=1, 상태 success, run 이름·path·event·repository·실행 상태
  검증은 유지한다. in-progress Controller가 정상 상태를 먼저 게시하는 기존 순서는 깨뜨리지 않는다.
- 필드가 중복되거나 형식이 잘린 상태를 Map 생성으로 정상화해 수용하지 않도록 확인한다.
- 현재 세 inline 소비자를 유지하고 Node 연결 테스트에서 실제 발행 출력을 각각 읽게 한다.
  새로운 helper를 PR head에서 읽거나 별도 checkout/API 의존을 만드는 리팩토링은 하지 않는다.
  버전은 명시적인 호환 계약으로 pin하되 앞으로 producer만 올리면 연결 테스트가 실패해야 한다.
- 일반 code PR의 review-only fast-pass, main의 기존 검증, Classifier 영향축 자체는 바꾸지 않는다.

### D. 기존 개선 보존

#6819의 `cancel-in-progress: event == pull_request_target`와 #7069의 수렴·attempt 검증을 유지한다.
취소 충돌에 대해 새로운 polling/timeout 확대를 구현하지 않는다. #6899 상세 실패 보고와 #7070
병합 후 중복 CI 제거도 그대로 보호한다.

## 2. 파일별 변경

| 파일 | 변경 |
| --- | --- |
| `.github/workflows/ci-impact-policy.yml` | job 진입·resolve의 devel/live identity 검사, publish 직전 base/head 확인·입력 전달·사유 |
| `.github/workflows/ci.yml` | trusted reuse 계약 일치, 새 Node 계약 테스트 실행 명령 추가 |
| `.github/workflows/codeql.yml`, `render-diff.yml` | 동일 trusted reuse 계약 수정. 실제 검사 job/matrix는 불변 |
| `scripts/tests/ci-impact-controller-contract.test.cjs` (신규) | 실제 YAML inline script·소비 함수를 읽어 모의 API로 실행하는 연결 테스트 |
| `scripts/tests/test_ci_impact_policy_workflow.py` | v5 고정 기대 제거, 이벤트/권한/새 테스트 CI 배선·게시 입력 계약 검증 |
| 관련 기존 workflow tests | 의미가 바뀐 계약만 수정. 실패를 통과시키려고 영향 없는 기대값은 변경하지 않음 |
| `mydocs/manual/github_operations.md`, 필요 시 `pr_review/review_only_fast_pass.md` | devel 전용 advisory 경계와 등록/운영 활성화 차이 명시 |

현재 `scripts/ci-impact-policy.cjs`는 올바른 v6을 발행하므로 이 파일을 고칠 필요는 없다.
`scripts/ci-workflow-evidence.cjs`, `ci-impact-report.cjs`도 새 이유 표현에 꼭 필요한 경우를 제외하면
변경하지 않는다. 추가 실행 정책 파일·권한·릴리즈 계약 변경이 필요하면 먼저 계획을 보완한다.

## 3. 실행형 테스트

테스트는 별도로 복제한 판정 함수를 검사하지 않고 실제 workflow inline 함수를 추출한다.
해당 블록을 찾지 못하면 테스트를 실패시킨다. API·시간 대기만 모의 처리하고 실제 GitHub mutation은
절대 실행하지 않는다. YAML 자체의 job 조건은 구조/표현식 계약과 이벤트 매트릭스를 함께 확인한다.

1. producer `statusDescription()`의 실제 v6 출력 → 세 소비자 모두 true.
2. v5/미지원 버전·중복/누락 필드·wrong creator/base/repo/workflow/event·실패 run → false.
3. main 대상·fork trusted reuse 요청 → false이며 status polling 호출 0회.
4. devel 이벤트가 resolve 전 main으로 retarget → active=false, 권한 조회/checkout 이전 종료.
5. resolve 뒤 main retarget·base SHA 전진·head 변경·close → publish 호출 0회.
6. 정상 open devel·정확한 head/base → 기존 publish/audit의 pending/success/failure 유지.
7. workflow_run의 devel/main/혼합/빈 목록·서로 다른 repo·오래된 head·모호한 복수 후보.
8. main의 `workflow_run`, push/tag/manual 완료 → 감사 비대상. 정보 없는 경우 식별만 수행.
9. 기존 concurrency, bounded collector, 실제 실패·누락·부적절 skip, reporter 및 post-merge
   duration 계약은 기존 테스트를 재사용한다.

1·4·5를 구현 전에 현재 코드에서 실패하도록 고정한다. 테스트가 모두 녹색인데 별도 수동 재현만
실패하는 상태를 남기지 않는다. 신규 테스트 명령을 기존 CI의 정책 검증 step에 배선한다.

## 4. 검증 명령과 비용

```bash
node --test scripts/tests/ci-impact-controller-contract.test.cjs \
  scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs \
  scripts/tests/ci-workflow-evidence.test.cjs scripts/tests/ci-impact-report.test.cjs \
  scripts/tests/collect-postmerge-duration-data.test.mjs
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  scripts.tests.test_ci_impact_policy_workflow scripts.tests.test_ci_impact_workflow \
  scripts.tests.test_codeql_workflow scripts.tests.test_render_diff_workflow \
  scripts.tests.test_review_only_fast_pass_workflows \
  scripts.tests.test_postmerge_duration_workflow \
  scripts.tests.test_trusted_postmerge_ci_reuse_workflow
git diff --check
```

변경 YAML 네 파일은 PyYAML BaseLoader 등 `on` 키를 보존하는 방식으로 파싱한다. inline JavaScript는
실행형 테스트와 syntax check로 검사한다. actionlint는 현재 PATH에 없어 미검증이며, 사용 가능한
기존 로컬 설치가 있는지 확인한 뒤 없으면 최종 보고에 명시한다. 무단 설치하지 않는다.

기존 Node 136개는 약 0.2초, Python 99개는 약 9.8초의 로컬 기준선이다. 새 테스트 비용은 구현 후
실측하며 이 값을 GitHub runner 비용으로 해석하지 않는다. 새 heavy job·상시 polling은 추가하지 않는다.
Rust·WASM 제품 빌드 및 의례적인 전체 조판 회귀는 이 변경의 로컬 필수 검증이 아니다.

## 5. 적용·완료 경계

1. N2 코드와 테스트 완료 후 N3에서 정확한 head의 검증·증적과 PR 초안을 준비한다.
2. push/PR/원격 CI·merge는 승인 후 진행한다. 이번 PR도 CI 실행 정책 변경이므로 정확한 Full
   candidate 검증이 선행되어야 한다. PR에서 소비자를 고쳐도 trusted controller의 main 배선은
   즉시 바뀌지 않는다. 재사용을 무조건 보장하거나 main을 우회 수정하지 않는다.
3. devel 병합 뒤 새로운 devel 대상 PR에서 v6 소비·선택 실행을 확인한다.
4. Controller YAML은 main에 정상 승격되기 전까지 운영 배선이 구버전이다. #6819/#7069를 포함한
   main/devel 차이와 이번 변경을 승격 검증에 전달하며 정규 릴리즈 일정 자체는 별도 승인 사항이다.
5. main 등록 후 실제 devel PR 감사·재사용 및 main 대상의 skip/무게시를 확인해야 운영 완료다.
   main run 기록의 생성과 감사 실행을 혼동하지 않는다. 실측 불가 항목은 미확인으로 남긴다.
6. 부작용은 이번 수정의 최소 revert PR로 복구한다. required check 해제·무조건 fast-pass·무한
   재시도는 복구 수단이 아니다.

구현 승인 요청 범위는 A~D 및 연결 테스트·운영 문서다. N2 진행 전에 본 계획 승인을 받는다.
