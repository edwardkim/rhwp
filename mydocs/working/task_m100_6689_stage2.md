# #6689 Stage 2 — workflow inventory·증적 검증기 결과

- Issue: #6689
- Plan: `mydocs/plans/task_m100_6689.md`
- Branch: `task_m100_6689`
- Baseline: `upstream/main@e8800c8def63449808a4092798442652ed460552`
- Candidate: `upstream/devel@1c49df3d33a323d459c8e90517f4a0f5bd3c790b`
- Status: complete
- Date: 2026-09-05 KST

## 1. 완료 범위

Stage 1에서 RED로 고정한 Git tree inventory와 offline evidence verifier를 표준 라이브러리만으로
구현했다.

1. `.github/workflows/**`와 `.github/actions/**`의 add·modify·delete·rename을 Git blob으로 비교한다.
2. before/after Git blob, 파일 SHA-256, base·candidate·merge-base를 결정적 JSON에 기록한다.
3. YAML scalar 밖의 주석과 빈 줄만 제거하는 보수적 fingerprint를 사용한다. block scalar 본문,
   들여쓰기·인용·재정렬처럼 동등성을 증명할 수 없는 변화는 실행 변경으로 남긴다.
4. trigger·routing·permission·secret·matrix·action ref·cache·artifact·timeout·concurrency·job command 등
   실제로 달라진 축을 `riskAxes`에 기록한다.
5. 새로 추가되거나 횟수가 증가한 외부 action 참조만 full commit SHA 정책과 대조한다.
6. evidence verifier는 candidate SHA, inventory hash, workflow content hash, run ID·URL·event·actor,
   job pagination, run·필수 job conclusion, verdict artifact와 waiver를 실패-폐쇄로 검증한다.
7. CLI는 성공 verdict에 0, 누락·불일치·정책 위반 verdict에 1을 반환한다.

구현과 계약 위치:

- `scripts/workflow_promotion_preflight.py`
- `scripts/tests/test_workflow_promotion_preflight.py`

## 2. GREEN 계약

기존 10개 RED에 inventory 변조, 신규 action delta, 허용 event·actor, GitHub run URL, pagination,
잘못된 snapshot 형식과 CLI exit code 계약을 추가했다.

```text
python3 -m unittest scripts/tests/test_workflow_promotion_preflight.py
Ran 17 tests in 0.610s
OK
```

Python syntax compile과 `git diff --check`도 통과했다. 변경분 Markdown link 검사는 614개 문서 중
HEAD 대비 변경 5개를 검사해 오류 0건이었다. metadata 전수 검사의 기존 오류 16건은 그대로이며 신규
오류는 없다. 유효하지 않은 run ID가 정렬 단계에서 예외를 발생시키지 않고 `invalid-run-id` 판정으로
닫히는 경계도 포함한다.

Stage 4 대상인 CI 계약 배선은 아직 의도대로 RED다.

```text
python3 -m unittest scripts/tests/test_workflow_contract_wiring.py
Ran 3 tests
FAILED (failures=2)
```

두 실패는 신규 test가 `ci.yml`의 workflow 계약 step과 impact conditioning 이후 살아남는 job에 아직
배선되지 않았다는 동일 원인이다. Stage 2 실패나 회귀가 아니며 Stage 4 전까지 녹색으로 위장하지 않는다.

## 3. 실제 `main → devel` 반복 계측

같은 프로세스에서 inventory를 두 번 생성하고 canonical JSON byte를 비교했다.

| 항목 | 결과 |
| --- | --- |
| 반복 출력 | byte-identical |
| inventory SHA-256 | `bbba6e3ed13fe9a40445e03af01f87ce99c88456a0900edaef02359d9b5e1f5f` |
| 변경 entry | 8개, 전부 `modified`·`executable` |
| policy violation | `actions/checkout@v4` 1종 |

자동 검출한 변경 축은 다음과 같다.

| workflow | 변경 축 |
| --- | --- |
| `adapter-diff.yml` | routing, job-command |
| `ci.yml` | trigger, routing, timeout, job-command |
| `codeql.yml` | trigger, routing, job-command, security |
| `deploy-pages.yml` | trigger, routing |
| `gym-release-gate.yml` | trigger, routing, action-ref, artifact, timeout, concurrency, job-command |
| `oracle-public-advisory.yml` | trigger, job-command |
| `proptest-roundtrip.yml` | routing, job-command |
| `render-diff.yml` | trigger, routing, job-command |

정책 위반이 있는 현재 inventory를 evidence verifier에 넣으면 run이 녹색이어도
`policy-violation:actions/checkout@v4`로 실패한다. Stage 3에서 참조를 pin하기 전에는 승격 증적을 만들 수
없다는 의도한 판정이다.

## 4. Stage 1 가정의 정정 필요점

### 4.1 새로 증가한 action 참조

Stage 1 계획은 Gym 변경에서 `actions/checkout@v4`와 `dtolnay/rust-toolchain@stable`이 모두 새로
사용됐다고 기술했다. Counter 기반 before/after 대조 결과는 다음과 같다.

- `actions/checkout@v4`: main 1회 → devel 2회. 새 호출 1회이므로 delta policy violation이다.
- `dtolnay/rust-toolchain@stable`: main 1회 → devel 1회. 신규 delta는 아니지만 현재 변경 파일에 남은
  비고정 참조다.

따라서 원인 계보에서는 둘을 “모두 신규”라고 쓰지 않는다. Stage 3 정정은 범위를 줄이지 않고 Gym의 두
checkout과 toolchain 참조를 저장소에서 이미 사용하는 full commit SHA로 함께 고정한다.

### 4.2 변경 축과 민감 표면

`riskAxes`는 before/after가 달라진 축이다. `deploy-pages.yml`의 Pages write permission·deploy action과
`oracle-public-advisory.yml`의 advisory artifact는 후보에도 존재하지만 이번 diff에서 그 자체가 바뀌지는
않았다. 이를 변경 축이라고 표시하면 계측이 사실과 달라진다.

Stage 3에서는 별도 workflow 정책 map에 다음을 분리한다.

- `changedAxes`: inventory가 계산한 실제 변화
- `sensitiveSurfaces`: 후보에 존재하는 deployment·write permission·secret·advisory artifact 등
- `executionMode`: direct, contracts-only, verify-only adapter 중 하나
- `requiredJobs`, `requiredVerdictArtifact`, `allowedEvents`, `allowedActors`

이 정책 map을 inventory hash에 포함한 뒤 실제 증적 요구를 계산한다. 자동 diff와 운영상 보호 대상이
서로를 가장하지 않게 하는 정정이다.

## 5. 보호 불변식 점검

- GitHub mutation, workflow dispatch, push, PR, comment는 수행하지 않았다.
- run URL은 GitHub Actions run 형식과 run ID가 일치해야 하며 임의 URL을 증적으로 받지 않는다.
- `schedule`은 기본 pre-main event로 인정하지 않고, workflow별 명시 allowlist만 확장할 수 있다.
- incomplete pagination, missing·skipped·failed job과 verdict artifact 누락을 녹색으로 바꾸지 않는다.
- inventory가 변조됐거나 신규 action pin 정책을 위반하면 valid run이 있어도 실패한다.
- waiver는 trusted maintainer, exact candidate·workflow hash, 제한 scope, reason, GitHub URL과 미래 만료를
  모두 요구한다.
- private corpus·개인 폰트·secret 값은 조회하거나 기록하지 않았다.

## 6. Stage 3 진입 조건

계획서의 Stage 2 수정안을 승인받은 뒤 다음 순서로 진행한다.

1. workflow별 정책 map과 실행 mode를 계약 test로 먼저 고정한다.
2. Gym의 비고정 checkout 두 곳과 toolchain 한 곳을 full SHA로 고정한다.
3. Pages verify-only, Gym contracts-only, Oracle verdict artifact의 실제 실행 경계를 구현한다.
4. task PR exact head에서 안전한 bootstrap run을 실증한다.

원격 push·PR 생성·workflow 실행은 이 Stage 2 완료 승인과 별도로 각각 승인을 받는다.
