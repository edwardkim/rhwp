# [#4662] CI 릴리스 게이트 — 처리 결과 (stage 1)

> 이슈: [#4662](https://github.com/edwardkim/rhwp/issues/4662) · 브랜치: `task_m100_4653` (운동장 확장 스택)

## 1. 한 문장

운동장 회귀 도구(차등 오라클 #4658·리더보드 #4659·릴리스 차등 #4661)를 하나의
**릴리스 판정**으로 묶어 CI 파이프라인에 물렸다. 도구가 도구로만 있으면 사람이
기억해서 돌려야 하지만, 파이프라인에 물리면 잊어도 돈다.

## 2. 최소 침습 원칙

릴리스 워크플로 본체(`release-binary.yml`)는 메인테이너 소유라 건드리지 않았다.
대신 두 갈래로 최소 침습했다:

- **독립 워크플로 신설** `.github/workflows/gym-release-gate.yml` —
  workflow_dispatch(수동) + 태그 push 관찰. 릴리스 본체와 무관하게 돈다.
- **기존 CI 가드 확장** — `ci.yml` 의 "Validate gym scorer contracts" 스텝에
  이번 스택의 가드 3종(pack·leaderboard·release_diff)을 배선. 전부 바이너리
  불요라 CI 시간 영향 미미.

## 3. 판정 계약 — regression 만 차단

`gym/tools/release_gate.py` 가 세 도구를 하나의 판정으로 묶는다.

| 판정 | exit | 조건 |
|---|---|---|
| `pass` | 0 | 릴리스 차등 stable(또는 대상 없음) + 리더보드 체인 무결 |
| `review` | 2 | surface-changed — 표면 변경, 사람 판정 |
| `block` | 3 | regression 또는 리더보드 체인 파손 |

**regression 만 자동 차단**하는 것이 급소다. 도구는 "무엇이 바뀌었나" 를
가리키지 "어느 쪽이 옳은가" 를 판정하지 않으므로(#4661 정직 조항), 표면 변경은
리뷰 신호이지 자동 차단 대상이 아니다. 명령이 추가된 릴리스를 회귀로 오차단하지
않는다.

## 4. 부재는 실패가 아니다

- **old 바이너리 없음**(직전 태그 미빌드) → 차등 skipped, 리더보드 검증만.
- **커밋된 리더보드 없음** → 리더보드 검증 skipped.

둘 다 pass 로 처리한다 — 부재를 실패로 위장하지 않는 결(#4653 unavailable 과
같은 문장).

## 5. 실측

### 판정 분기 (목 주입 — 네 갈래 전부)

| 입력 | 판정 | exit |
|---|---|---|
| 차등 stable | pass | 0 |
| 차등 surface-changed | review | 2 |
| 차등 regression | block | 3 |
| 리더보드 체인 파손 | block | 3 |

### 실행 시나리오 (실제 러너)

- **old 없음** → 차등 skipped · 리더보드 무결 · pass(0)
- **자기-대조**(old=new) → stable · pass(0)

## 6. CI 배선 규약 준수 (#4080)

이 저장소는 `test_*workflow*.py` 계약 테스트를 만들면
`test_workflow_contract_wiring.py` 가 ci.yml 배선을 강제한다 — 파일만 추가하고
배선을 잊으면 한 번도 안 도는 회귀를 막는 장치다. 이 규약 그대로
`test_gym_release_gate_workflow.py` 를 만들었고, 그 파일명이 wiring 테스트에
잡혀 ci.yml 배선을 함께 넣게 됐다(실측: wiring 테스트가 배선 후 통과).

## 7. 검증

| 게이트 | 결과 |
|---|---|
| 판정 분기 | 4/4 일치(목) |
| 실행 시나리오 | 2/2 pass(실측) |
| `test_gym_release_gate_workflow.py` (신규 12) | 워크플로 계약 7 + 러너 판정 계약 5 |
| gym 가드 합계 | **55/55** (score 17 + pack 10 + leaderboard 7 + release-diff 9 + release-gate 12) |
| YAML 문법 | gym-release-gate.yml · ci.yml 유효 |
| 릴리스 본체 무침습 | 계약 테스트가 확인(release-binary.yml 에 게이트 침습 없음) |

시각 증거: `mydocs/report/edit_demo_4662/01_release_gate.png`

## 8. 남긴 것

- 실제 CI 주행 관찰 — 태그 push 시 old 바이너리 빌드(전체 빌드 2회) 부하 측정 후
  push 트리거 유지 여부 판단(현재 workflow_dispatch 로도 충분).
- 게이트 판정을 PR 코멘트/체크로 노출(현재 job summary + 아티팩트).
