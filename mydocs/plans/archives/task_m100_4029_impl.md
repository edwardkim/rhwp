# 구현계획서 — task_m100_4029

- **Issue**: #4029
- **기준 계획**: [task_m100_4029.md](task_m100_4029.md)
- **브랜치**: `issue-4029-cold-release-ci`
- **승인 상태**: 2026-08-11 전체 보정 방향·진행 승인 완료

## 1. 파일별 변경

### `.github/workflows/ci.yml`

1. `workflow_dispatch.inputs.release_grade` boolean을 기본값 `false`로 추가한다.
2. `preflight`에 `test_profile`, `test_archive_timeout_minutes` output을 추가한다.
3. `Select test profile policy` step이 event/ref/input 진리표를 한 번만 계산한다.
4. 세 archive builder와 Native Skia test는 같은 `test_profile`을 소비한다.
5. archive timeout은 세 builder에만 전달하고 Native Skia의 기존 30분 상한은 유지한다.
6. 새 workflow 계약 테스트를 Lint job의 기존 계약 테스트 묶음에 배선한다.

### `.github/workflows/build-nextest-archives.yml`

1. `cargo_profile` string과 `timeout_minutes` number를 reusable input으로 받는다.
2. job timeout을 `inputs.timeout_minutes`로 설정한다.
3. `release-test:30`, `release:60` 외 조합을 build 전에 실패시킨다.
4. workflow 안의 중복 event/ref profile selector를 제거하고 전달받은 값을 nextest에 사용한다.
5. Rust cache step에 id를 주고 exact-hit output을 요약한다.
6. `if: always()` 요약에 event/ref/profile/timeout/cache exact-hit/save eligibility를 기록한다.

### `scripts/tests/test_nextest_archive_workflow.py`

장기 운영 계약 이름으로 다음을 고정한다.

- 수동 입력의 type/default
- event/ref/input별 profile/timeout 진리표
- 세 builder가 동일한 preflight output을 전달하는 배선
- Native Skia test가 같은 profile output을 소비하는 배선
- reusable workflow 입력과 동적 timeout
- 허용 조합 검증 및 중복 selector 부재
- cache exact-hit·운영 summary
- `Build & Test` check identity
- release binary와 WASM artifact의 release 명령 불변

### 문서

- `mydocs/orders/20260811.md`: #4029 진행 상태
- `mydocs/working/task_m100_4029_stage1.md`: RED/GREEN focused 검증과 변경 결과

## 2. TDD 순서

1. 새 Python 계약 테스트와 CI 배선을 먼저 추가한다.
2. 테스트가 profile 라우터/inputs/summary 부재로 실패하는 RED를 기록한다.
3. workflow 두 파일을 구현한다.
4. 새 테스트와 전체 workflow 계약 테스트를 재실행한다.
5. `actionlint`와 `git diff --check`를 실행한다.

## 3. focused 검증

```bash
python3 -m unittest scripts/tests/test_nextest_archive_workflow.py
python3 -m unittest scripts/tests/test_workflow_contract_wiring.py
python3 -m unittest discover -s scripts/tests -p 'test_*workflow*.py'
actionlint .github/workflows/ci.yml .github/workflows/build-nextest-archives.yml
git diff --check
```

CI workflow만 바꾸므로 Rust/Cargo 전체 회귀는 기본 focused 범위가 아니다. 긴 전체 CI와 두 수동
canary는 focused 결과를 공유한 뒤 원격 단계에서 수행한다.

## 4. 원격 판정 항목

| 실행 | 기대 profile/timeout | 판정 |
| --- | --- | --- |
| PR | `release-test/30` | archive·Native Skia profile과 기존 check identity 유지 |
| 같은 SHA 일반 dispatch | `release-test/30` | cold여도 세 archive·네 worker·Native Skia 완주 |
| 같은 SHA `release_grade=true` | `release/60` | cold release builder와 Native Skia 완주 시간·여유 측정 |

세 실행에서 `Build & Test` 결과, builder별 duration, cache exact-hit, archive upload, worker 실행 수를
기록한다. 60분 안에도 완주하지 못하면 단순 상향을 반복하지 않고 target scope·LTO 분리를 다시 설계한다.
