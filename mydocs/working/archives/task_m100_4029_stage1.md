# #4029 Stage 1 완료보고 — cold-cache CI test profile 정책 구현

- **Issue**: [#4029](https://github.com/edwardkim/rhwp/issues/4029)
- **브랜치**: `issue-4029-cold-release-ci`
- **기준**: `upstream/devel` `b66e3d79a93c048478c4737443084f9e7149bbb2`
- **결정 기록**: [issue comment 5251618440](https://github.com/edwardkim/rhwp/issues/4029#issuecomment-5251618440)
- **상태**: 로컬 구현·focused 검증과 동일 SHA 원격 canary 완료, 최신 `devel` 재검증 중

## 1. 구현 결과

`ci.yml`의 preflight가 event/ref와 수동 `release_grade` 입력을 한 번 해석해 다음 두 값을 출력한다.

- `test_profile`: `release-test` 또는 `release`
- `test_archive_timeout_minutes`: `30` 또는 `60`

세 nextest archive builder와 Native Skia test는 같은 `test_profile`을 소비한다. archive builder만
profile과 timeout의 허용 조합을 다시 검증한다.

| 실행 | profile | archive timeout |
| --- | --- | ---: |
| PR, devel push, 일반 수동 full | `release-test` | 30분 |
| main push, `v*` tag, `release_grade=true` 수동 full | `release` | 60분 |
| 알 수 없는 event/ref 또는 잘못된 수동 입력 | `release` | 60분 |

실제 release binary와 WASM artifact 명령, builder/worker topology, `Build & Test` 이름은 바꾸지 않았다.
cache 저장 범위도 기존 `devel/main push`를 유지한다.

## 2. 구현 중 설계 보정

첫 GREEN 직후 인접 경로를 다시 확인해, archive만 공통 policy를 소비하면 일반 수동 full의 Native Skia가
기존 event 분기 때문에 계속 `release`로 남는 불일치를 발견했다. Native Skia는 배포 artifact가 아니라
회귀 테스트이므로 같은 `test_profile` output을 사용하도록 보정했다. main/tag/명시적 release-grade는
여전히 `release`이며 Native Skia job의 30분 상한은 그대로다.

## 3. TDD 근거

### 최초 RED

```text
python3 -m unittest scripts/tests/test_nextest_archive_workflow.py
Ran 8 tests
FAILED (failures=9)
```

기존 release artifact 불변 계약만 통과했고, 수동 입력·profile 라우터·reusable inputs·동적 timeout·
cache summary가 없어 예상대로 실패했다. `test_workflow_contract_wiring.py`는 새 파일 배선을 3/3으로
확인했다.

### Native Skia 공통 profile RED

archive 구현의 첫 GREEN 뒤 공통 profile 계약을 추가하자 9개 테스트에서 6개 failure가 발생했다.
기존 `GITHUB_EVENT_NAME` 분기와 archive 전용 output 이름이 원인이었고, `test_profile` 단일 output으로
바꾼 뒤 GREEN으로 전환했다.

## 4. focused 검증

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest scripts/tests/test_nextest_archive_workflow.py` | **9/9 PASS** |
| `python3 -m unittest discover -s scripts/tests -p 'test_*workflow*.py'` | **98/98 PASS** |
| `actionlint .github/workflows/ci.yml .github/workflows/build-nextest-archives.yml` | **PASS** |
| `git diff --check` | **PASS** |

CI workflow와 계약 테스트만 변경했으므로 Cargo 전체 회귀는 focused 범위에서 제외했다. GitHub Actions의
boolean dispatch input, reusable workflow typed input과 caller `with` 전달 규칙은 2026-08-11 현재
[GitHub workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)와
[reusable workflow 문서](https://docs.github.com/en/actions/reference/workflows-and-actions/reusing-workflow-configurations)로
교차 확인했다.

## 5. 안전성 판정

- required check `Build & Test`와 aggregate 진리표는 불변이다.
- PR/devel의 timeout은 늘지 않는다.
- release-grade 경로만 60분 상한을 사용하며 정상 실행 시간 자체는 늘리지 않는다.
- main cold run이 완주하면 기존 main push `save-if`가 cache를 다시 만들 수 있다.
- tag별 cache를 새로 저장하지 않아 #4080의 ref별 cache 증가를 악화시키지 않는다.
- `cache_exact_hit`는 exact key hit만 뜻하며 false를 부분 복원과 완전 miss로 과해석하지 않는다.

## 6. 남은 원격 검증

1. Draft PR의 기본 CI가 `release-test/30`과 기존 check identity로 통과하는지 확인한다.
2. 같은 SHA 일반 `workflow_dispatch`에서 archive와 Native Skia가 `release-test`로 완주하는지 측정한다.
3. 같은 SHA `release_grade=true`에서 cold `release/60` builder 세 개와 worker 네 개가 완주하는지 측정한다.
4. cache exact-hit, builder별 duration, archive upload, worker run count를 #4029에 후속 기록한다.

60분에도 cold release가 완주하지 못하면 timeout 추가 상향으로 덮지 않고 test target scope 또는 LTO
검증 구조를 다시 설계한다.

## 7. 동일 SHA 원격 canary 결과

PR head `9dbc0d91f7f663421af36ed6acfee7a39499a33c`에서 다음 두 수동 full run을 순서대로 실행했다.

| 실행 | 정책·cache | 결과 |
| --- | --- | --- |
| [일반 dispatch 31483270200](https://github.com/edwardkim/rhwp/actions/runs/31483270200) | `release-test/30`, 세 builder exact hit `false` | `Build & Test` 성공, wall clock 17분 44초 |
| [release-grade dispatch 31483281790](https://github.com/edwardkim/rhwp/actions/runs/31483281790) | `release/60`, 세 builder `No cache found`·exact hit `false` | `Build & Test` 성공, 실행 구간 약 60분 34초 |

release-grade archive job은 A 52분 01초, slow builder 48분 03초, B 49분 27초에 완주했다. 가장 느린
A의 Cargo release build는 50분 07초였고 job 상한까지 약 7분 59초가 남았다. Native Skia는 같은
`release` profile과 기존 30분 상한에서 23분 15초에 성공했다. 네 worker는 slow 1, archive 1 3,938,
archive 2 866, archive 3 952로 총 5,757개를 실행했고 aggregate의 기대 수와 일치했다.

따라서 30분 상한이 cold release archive를 반복 취소하던 #4029의 직접 실패 조건은 해소됐다. 다만
수동 dispatch는 cache 저장 대상이 아니므로 실제 main push의 cache 재생성과 약 8분인 cold builder
여유 폭은 메인테이너 운영 단계에서 후속 관찰한다. canary 뒤 `upstream/devel` 전진으로 PR이 충돌 상태가
되어 최신 기준선을 merge하고 새 head의 CI를 다시 확인한 뒤 리뷰를 요청한다.
