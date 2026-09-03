# PR 초안 — Gym 오라클·전수 계약 핵심 정상화

## Summary

- 1,035개 Gym task의 정답 권위와 기준풀이 출처를 결정론적 원장으로 분류합니다.
- 알려진 19 task·28 false-pass를 포함한 약한 answer/artifact oracle, stale reference와
  Python 계약을 fail-closed 방식으로 정상화합니다.
- positive, discrimination, trajectory를 서로 독립된 감사 축으로 고정합니다.
- Gym을 제품 릴리스 게이트에서 분리하고 Gym 관련 PR의 빠른 계약과 메인테이너 수동 전수
  실행으로 운영 경계를 정리합니다.
- 인간 개발자용 수동 운영 매뉴얼과 `gym/**` 범위 AI 에이전트 지침을 추가합니다.

Refs #6628

> 이 PR은 #6628의 핵심 구현을 devel에 통합하지만 부모 이슈를 닫지 않습니다. JSON 증적
> 시각화 sub-issue #6669를 후속 독립 PR로 완료한 뒤 #6628의 최종 종료 게이트를 정산합니다.

## Why

기존 Gym은 기준풀이의 녹색 수치가 같은 rhwp가 만든 self-live 결과인지, 독립 fixture 또는
외부 oracle인지 구분하지 못했습니다. 일부 채점은 파일 존재·크기·동일 해시만 확인해 실제로
과제를 수행하지 않은 제출도 통과시켰고, trajectory step의 실질 필요성과 실행 오류도 전수
계약으로 고정되지 않았습니다.

또한 Gym 전건 평가가 제품 태그와 릴리스 맥락에 연결돼 벤치마크의 목적과 제품 배포 게이트가
혼동됐습니다. 이 PR은 Gym을 AI 에이전트의 rhwp CLI/API 활용 능력 벤치마크로 한정하며,
한컴 조판 동등성 또는 제품 릴리스 적합성의 독립 증거로 승격하지 않습니다.

## Changes

### Oracle and evidence contracts

- `authority_ledger.py`와 1,035개 task 전수 분류 계약
- full-text/digest/상한 기반 answer·artifact 검사와 오류 봉투 불통과
- current CLI와 어긋난 reference 및 pack-health 계약 정상화
- positive/discrimination/trajectory 봉투 자체 검산과 회귀 시험

### Workflow boundary

- 기존 workflow 파일 경로를 유지하고 표시 이름을 `Gym Benchmark Validation`으로 변경
- Gym 관련 PR에서 빠른 구조·단위 계약 실행
- 전수 benchmark는 `workflow_dispatch`에서만 실행하고 60분 timeout·30일 artifact 보존
- devel/main push와 v* tag, Release Binary, npm·extension 게시에서 Gym artifact 비생산·비소비
- CI impact classifier가 Gym-only 변경을 제품 worker와 CodeQL에 과잉 배정하지 않도록 계약화

### Operations and documentation

- `mydocs/manual/gym_benchmark_operations.md`: 인간 메인테이너의 격리 실행·판정·증적·정리 정본
- `gym/AGENTS.md`: 참가자와 메인테이너 감사 역할, reference 비노출, fail-closed 불변식
- 각 Gym 도구 규약과 문서 진입점 현행화

## Validation

현재 최신 devel 동기화 tree에서:

- `python3 -m unittest discover -s scripts/tests -p 'test_gym_*.py'`
  - 3,149 passed, 1 policy skip
- workflow의 exact Gym contract command
  - 2,123 passed, 1 policy skip
- `python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'`
  - 148 passed
- CI impact classifier/policy Node contracts
  - 78 passed
- `python3 gym/tools/audit.py --json`
  - 21 packs, 1,035 tasks, issue 0
- `python3 gym/tools/oracle_probe.py --json`
  - structural issue 0
- `python3 gym/tools/oracle_probe.py --selftest --json`
  - 14/14 passed
- `python3 gym/tools/authority_ledger.py --json`
  - task/reference/entry 1,035/1,035/1,035, issue 0
- workflow YAML parse, `git diff --check`, changed-document link/redirect check passed

#6641의 exact product candidate와 이 branch의 Gym runner로 실행한 전수 결과:

- positive: 21 packs, 1,035/1,035, failure/skip 0
- discrimination: 1,035 tasks, 1,511 controls, false-pass 0
- trajectory: 239/239 load-bearing, theater/exception/tool error 0

전수 runner는 `374c7416a6c2d9abe7c2701969de5f377b71183f`, product candidate는
`7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`, binary SHA-256은
`4334e35e3bfb7e892416e663c7a52dd055a7d82040d2a89447f32d25ccd02f34`입니다.
현재 devel까지 관련 실행 source diff가 0임을 확인해 같은 38분 전수 실행을 중복하지 않았습니다.

## Authority boundary

- self-live 987
- contract-constant 28
- independent-fixture 20
- external-oracle 0

external oracle은 아직 0개입니다. 이 PR의 전수 통과는 공개 Gym 내부 정합성과 판별력의
증거이며, 한컴 조판 동등성이나 제품 릴리스 적합성의 증명은 아닙니다.

## Operational impact and rollback

- 변경 등급: O3 — workflow trigger, job command, artifact와 CI routing 계약 변경
- devel required context `Build & Test`의 이름과 생성 주체는 변경하지 않음
- Gym workflow file identity와 최소 `contents: read` permission 유지
- 예상 PR 동작: Gym contracts 실행, 수동 전수 job은 skip
- 예상 devel/main/tag 동작: Gym workflow 자동 실행 없음
- rollback: 이 PR의 workflow·classifier·계약 커밋을 함께 revert하고 required context 생성 여부를 재관찰

## Evidence

- `mydocs/report/task_m100_6628_report.md`
- `mydocs/working/task_m100_6628_stage4.md`
- `mydocs/working/task_m100_6628_stage6.md`
- `mydocs/report/task_m100_6641_report.md`

## Checklist

- [x] 최신 `upstream/devel` 병합 및 충돌 없음
- [x] task/reference/authority 1,035건 누락·중복 없음
- [x] positive/discrimination/trajectory 종료 게이트 충족
- [x] Gym의 제품 CI·릴리스·게시 비의존 계약 확인
- [x] 사설 코퍼스·비밀값·생성 제출물 미포함
- [x] 로컬 workflow·문서 검증 완료
- [ ] 원격 PR 최신 head의 GitHub Actions 확인
- [ ] self-review와 merge 승인
- [ ] #6669 완료 뒤 부모 #6628 최종 종료 감사
