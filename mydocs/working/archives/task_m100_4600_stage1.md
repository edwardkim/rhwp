# [#4600] gym 채점기 오검출 수리 — 처리 결과 (stage 1)

> 이슈: [#4600](https://github.com/edwardkim/rhwp/issues/4600) (메인테이너 제기·재현 완료)
> 브랜치: `task_m100_4600` (base: `origin/devel`)

## 1. 무엇을 고쳤나

`gym` 채점기가 **지시한 대상이 아닌 것을 고친 제출**과 **아무것도 고치지 않은
제출**을 통과시키던 결함을 막았다. 이슈가 재현한 잘못된 제출 묶음은 T07·T08·T10
각 3점씩 **9점을 부당 취득**했다 — 이 PR 이후 **0점**이다. 올바른 기존 제출의
32/32 는 그대로 유지된다.

원인은 세 검사가 모두 "무엇을 고쳤는지"가 아니라 "값이 어딘가 있는지"만 봤다는
것이다. 검사는 대상을 **지목**해야 한다.

## 2. 과제별 수리

| 과제 | 통과해선 안 되던 제출 | 전 | 후 |
|---|---|---|---|
| T07 서식 채움 | 첫 필드(회사명)는 비우고 두 번째(작성자)만 채움 | 봉투 전역 `deep_contains("홍길동")` | `value_eq` @ `fields[0].value` — 실측 `""` 로 차단 |
| T08 표 셀 교정 | (0,0) 은 두고 (1,0) 만 교체 | 봉투 전역 `deep_contains("짐검증")` | 신규 `cell_text_eq` (table 0, row 0, col 0) — 실측 원문 그대로라 차단 |
| T10 결정론 실증 | 입력 HWP 를 o1·o2 로 단순 복사 | `same_hash` 뿐 | `same_hash` + 신규 `differs_from_input` + **계획 재현**(replay) |

### T10 — 결정론을 "증명"으로 바꾼 방법

복사본 두 벌은 서로 같으므로 `same_hash` 만으로는 영원히 막을 수 없다. 그래서
제출 규약에 **계획서(`plan.json`)** 를 추가하고, 채점기가 그 계획을 rhwp 로
되돌려 실행해 산출물을 실제로 재현하는지 본다:

```
replay {file:plan.json} --expect-output-sha256 {sha256:o1.hwp} --json  →  reproduced: true
```

기대 해시를 과제 파일에 박제하지 않고 **채점 시점에 제출물에서 계산해 rhwp 에게
판정을 시킨다** — 살아있는 오라클 원칙 그대로다. `expect_exits:[0,3]` 으로
미재현(exit 3)도 봉투로 읽어 판정 데이터로 다룬다.

## 3. 채점기 변경 (`gym/score.py`)

- 신규 op `cell_text_eq` — `find_cell()` 로 (row, col) 을 찾는다. `cells[0]`
  같은 **순서 가정을 쓰지 않는다**(순서 가정은 내보내기 구현이 바뀌면 조용히
  엉뚱한 셀을 검사하게 되며, 그것이 이 이슈와 같은 부류의 결함이다).
- 신규 op `differs_from_input` — 제출물이 과제 입력과 바이트가 같으면 실패.
- 신규 자리표 `{sha256:이름}` — 제출물의 채점 시점 해시를 인자로 넘기는 통로.

## 4. 부수 발견 — T13 이 없는 명령을 부르고 있었다

검증 중 `gym/tasks/T13.json` 이 **존재하지 않는 `harness-status`** 를 호출해
exit 2 로 영구 실패하는 것을 발견했다(devel 실측). 실제 CLI 는 우산 명령
`harness status` 다. 같은 계열(과제 파일 결함)이라 이 PR 에서 함께 교정하고,
재발 방지로 **과제가 부르는 명령이 CLI 에 실재하는지 검사하는 가드**를 넣었다.

## 5. 음성 회귀 (CI 상시 가드)

`scripts/tests/test_gym_score.py` (CI `Validate gym scorer contracts` 가 실행)에
**반드시 실패해야 하는 제출**을 고정했다 — 통과 제출만 검사하면 채점기는
"무엇이든 통과시키는" 방향으로 조용히 썩는다.

| 테스트군 | 고정하는 것 |
|---|---|
| `WrongTargetRegressionTests` | 엉뚱한 필드/셀 제출 실패 · 올바른 제출 통과 · 좌표 부재 시 조용한 통과 금지 · 무편집 복사 실패 · 실제 편집물 통과 · `{sha256:}` 자리표 해석 |
| `WeakCheckLockTests` | T07·T08 이 `deep_contains` 로 되돌아가지 못하게 잠금 · T10 이 3중 검사(동일성·무편집거부·재현)를 유지하는지 |
| `TaskCommandExistenceTests` | 과제가 부르는 명령 머리 토큰 실재성(§4 재발 방지) |

## 6. 검증 실측

| 게이트 | 결과 |
|---|---|
| 잘못된 제출(이슈 재현 묶음) | **9/9 통과 → 0/9 차단** (변경 전 채점기를 `origin/devel` 에서 꺼내 같은 제출에 돌린 대조 측정) |
| 올바른 제출 baseline | **32/32 유지** (T07·T08·T10 새 검사 전부 통과) |
| `python -m unittest scripts/tests/test_gym_score.py` | **17/17** (기존 7 + 신규 10) |
| 과제 명령 실재성 감사 | 미존재 명령 **0건** (교정 전 1건) |
| Markdown 링크 검사 | 이상 없음 |

시각 증거: `mydocs/report/edit_demo_4600/01_false_positive_blocked.png`

## 7. 부딪힌 함정 (정직 기록)

1. **T12 로컬 제출물이 낡아 있었다** — #4586 수정 전에 `convert` 로 만든
   `conv.hwpx` 는 실제로 HWP5 였다. `export-hwpx` 로 재생성하고 답안을 실측
   (`identical: false`)에 맞추니 통과. 저장소 결함이 아니라 제출물 노후화라
   커밋 대상이 아니다(제출물은 `.gitignore`).
2. **상대 경로 바이너리 = WinError 2** — 감사 스크립트에서 재현. `score.py`
   `find_bin()` 주석이 경고하는 그 함정으로, 절대 경로로 해결.
3. **계획서의 `output` 이 제출자 절대 경로였다** — 베이스라인 계획서를 상대
   이름으로 정리했고, 재현 판정이 그대로 통과함을 실측(출력 파일명이 산출
   바이트에 영향을 주지 않음을 부수 확인).

## 8. 남긴 것

메인테이너가 [PR #4465](https://github.com/edwardkim/rhwp/pull/4465) 에서 제시한
**pack 단위 대형 운동장**(core/packs/profiles 구조) 은 이 PR 범위 밖의 후속
확장이다. 채점기의 구멍을 먼저 막는 순서가 맞다 — pack 을 8개로 늘리면 오검출도
같이 늘어나고, 이번에 만든 `cell_text_eq`·`differs_from_input` 은 그 구조에서
공통 check registry 의 첫 입주자가 된다.
