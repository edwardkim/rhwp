# #6689 Stage 1 — 기준선·RED 계약 결과

- Issue: #6689
- Plan: `mydocs/plans/task_m100_6689.md`
- Branch: `task_m100_6689`
- Baseline: `upstream/devel@1c49df3d33a323d459c8e90517f4a0f5bd3c790b`
- Status: complete
- Date: 2026-09-05 KST

## 1. 완료 범위

Stage 1은 실제 promotion 판정기를 구현하지 않고, 과거 누락을 재현하는 기준선과 구현이 만족해야 할
실패 계약을 먼저 고정했다.

1. `main`, `devel`, merge-base와 양쪽 고유 commit 수를 고정했다.
2. `main..devel`에서 수정된 workflow 8개의 before/after Git blob과 byte SHA-256을 보존했다.
3. candidate exact head의 Actions run과 미실행 workflow를 분리했다.
4. Fuzz smoke의 기존 세 run에서 6개 matrix 중 `parse_wmf`만 실패한 사실과 devel run 0건을 보존했다.
5. PR #5366의 check 0건과 통합 PR #5425의 일반 workflow 녹색이 `Fuzz smoke` 증적을 포함하지 않는다는
   원인 계보를 고정했다.
6. inventory 4건, evidence 6건의 RED test와 Stage 2용 공개 함수 interface를 추가했다.

증적 위치:

- `mydocs/tech/investigations/issue-6689/README.md`
- `mydocs/tech/investigations/issue-6689/baseline.json`

## 2. 기준선 재현 결과

`baseline.json`을 Git live tree와 다시 대조한 결과는 다음과 같다.

```text
baseline verified: 8 workflow changes, refs/counts/hashes exact
```

검사한 항목은 다음과 같다.

- `upstream/main`, `upstream/devel`, merge-base SHA
- `git rev-list --left-right --count`의 `2 238`
- 8개 workflow의 before/after Git blob
- 8개 workflow의 before/after SHA-256

JSON parse와 production/test Python syntax compile도 통과했다.

## 3. RED 계약

### 3.1 inventory 4건

1. YAML scalar 밖의 주석·빈 줄·inline comment만 바뀌면 `comment-only`
2. block scalar 내부 내용 변경은 `executable`과 `job-command`
3. trigger·permission·secret·matrix·action ref·cache·artifact·timeout·concurrency 검출
4. add/delete/rename의 정렬과 반복 실행 결과 결정론

### 3.2 evidence 6건

1. candidate exact SHA·workflow hash와 Fuzz 6-job 전건 성공
2. 일반 CI 녹색으로 누락된 Fuzz run을 대신할 수 없음
3. stale head 또는 workflow hash 불일치는 fail-closed
4. 필수 job의 missing/skipped/failed 상태는 성공 아님
5. `continue-on-error` workflow는 별도 verdict artifact 필요
6. waiver는 maintainer·exact scope·future expiry를 모두 충족해야 함

실행 결과:

```text
python3 -m unittest scripts/tests/test_workflow_promotion_preflight.py
Ran 10 tests
FAILED (errors=10)
```

10건 모두 `build_inventory()` 또는 `verify_evidence()`의 Stage 2 `NotImplementedError`에서 실패했다.
assertion 이전의 예상하지 않은 parser·fixture·Git 오류는 없다.

CI 계약 배선 검사 결과:

```text
python3 -m unittest scripts/tests/test_workflow_contract_wiring.py
Ran 3 tests
FAILED (failures=2)
```

- 신규 `test_workflow_promotion_preflight.py`가 CI Lint job에 아직 호출되지 않음
- 같은 테스트가 impact conditioning 이후에도 살아남는 job에 아직 없음

이는 Stage 4에서 CI에 배선하기 전까지 유지해야 하는 의도된 RED다. 나머지 discovery pattern test 1건은
통과해 새 test 자체는 자동 발견되고 있다.

## 4. 테스트 자체 결함 정정

첫 실행에서는 evidence fixture helper를 `run()`으로 명명해 `unittest.TestCase.run()`을 덮어썼다.
이는 제품 계약 실패가 아니라 테스트 하니스 결함이므로 `candidate_run()`으로 바꾸고 즉시 재실행했다.
정정 뒤에는 위 10건이 모두 의도한 미구현 함수에서만 실패한다.

## 5. 보호 불변식 점검

- secret 값은 조회·기록하지 않았다. repository secret은 이름조차 baseline JSON에 넣지 않았다.
- GitHub mutation, workflow dispatch, push, PR, comment는 수행하지 않았다.
- private corpus·개인 폰트·artifact를 사용하거나 기록하지 않았다.
- 일반 CI 성공과 개별 workflow 실행 성공을 분리했다.
- Fuzz 실패를 `continue-on-error`나 허용 conclusion으로 바꾸지 않았다.
- 다른 worktree와 사용자 WIP를 건드리지 않았다.

## 6. Stage 2 진입 조건

Stage 2에서는 두 sentinel 함수를 실제 구현해 기능 계약 10건을 GREEN으로 전환한다. CI 배선 2건은
Stage 4까지 RED로 유지한다. Stage 2 구현이 수작업 기준선의 8개 위험 축과 다르게 분류하면 구현을
통과시키지 않고 classifier를 정정한다.
