---
kind: working
status: active
canonical: mydocs/working/task_m100_6628_stage1.md
issue: 6628
last_verified: 2026-09-02
---

# #6628 Stage 1 — 정답 권위 분류 모델과 전수 원장

## 1. 결론

21개 pack의 task/reference 1,035쌍을 한 번씩 분류하는 결정론적 원장을 만들었다.
현행 Gym에는 독립 제품 정답으로 부를 수 있는 외부 오라클이 없다. 가장 많은 987개
task는 채점 대상과 같은 현재 rhwp를 다시 실행하는 `self-live`이고, 이는 Gym 내부
일관성과 에이전트의 CLI/API 사용 능력을 측정할 뿐 한컴 결과와의 동등성을 증명하지
않는다.

원장 규약과 사용법의 정본은
[`../../gym/docs/authority_ledger.md`](../../gym/docs/authority_ledger.md), 생성기는
[`../../gym/tools/authority_ledger.py`](../../gym/tools/authority_ledger.py)다.

## 2. 왜 채점 권위와 기준풀이를 분리했는가

초기 계보 조사는 reference가 현재 rhwp를 얼마나 호출하는지 세었다. 하지만 reference는
기준 제출물을 만드는 경로이고 task의 `checks`는 제출물을 합격시키는 판정 경로다.
둘을 같은 ‘정답지’로 부르면 다음 경우를 구별하지 못한다.

- 현재 rhwp로 산출물을 만들고 현재 rhwp로 다시 읽어 판정
- 현재 rhwp로 산출물을 만들지만 공개 입력과 달라졌는지만 판정
- 작성자가 넣은 상수 answer를 같은 작성자 상수와 비교

따라서 원장은 `authority`와 `baselineSource`를 독립 축으로 기록한다. primary
authority는 task당 하나이며, 보조 신호는 `authoritySignals`에 별도로 보존한다.

## 3. 보수적 분류 규칙

판정 의존성을 숨기지 않도록 다음 우선순위를 적용했다.

1. scoring check 하나라도 현재 rhwp CLI를 요구하면 `self-live`
2. live check가 없을 때만 저장소 내부의 명시된 외부·독립 증적 적용
3. 명시 증적이 없고 공개 input과 `differs_from_input` 관계를 검사하면
   `independent-fixture`
4. 그 밖의 task 작성자 고정값·형식·제출물 관계는 `contract-constant`

`external-oracle` 또는 명시적 `independent-fixture`는 저장소 안에 실제 존재하는 증적
경로를 요구한다. live check가 있는 task는 메타데이터만 붙여 더 독립적인 권위로
승격할 수 없다. 공개 input과 다르다는 판정도 그 관계만 독립적일 뿐 HWP 의미·조판
전체가 옳다는 증거가 아니다.

reference는 `run`/answer `cmd`를 `self-live`, answer `const`를
`contract-constant`로 분류한다. 한 task에서 두 출처가 섞이면 실패한다.

## 4. 전수 계측 결과

### 채점의 primary authority

| class | task | 비율 |
|---|---:|---:|
| `self-live` | 987 | 95.36% |
| `contract-constant` | 28 | 2.71% |
| `independent-fixture` | 20 | 1.93% |
| `external-oracle` | 0 | 0.00% |
| 합계 | 1,035 | 100.00% |

### 기준풀이 출처

| source | task | 비율 |
|---|---:|---:|
| `self-live` | 1,031 | 99.61% |
| `contract-constant` | 4 | 0.39% |
| 합계 | 1,035 | 100.00% |

명시적인 `authority` 메타데이터가 있는 task는 현재 0개다. 즉 현 분류는 모든 task의
실제 check 연산자와 reference를 재계산한 결과다.

### pack별 primary authority

| pack | self-live | contract-constant | independent-fixture | external-oracle |
|---|---:|---:|---:|---:|
| automation | 66 | 4 | 0 | 0 |
| batch-ops | 17 | 3 | 0 | 0 |
| casual-rides | 44 | 0 | 0 | 0 |
| core-cli | 54 | 0 | 0 | 0 |
| corpus-diagnostics | 48 | 0 | 0 | 0 |
| expert-challenges | 55 | 0 | 0 | 0 |
| extraction | 28 | 0 | 0 | 0 |
| form-journeys | 72 | 0 | 0 | 0 |
| layout-rendering | 42 | 0 | 6 | 0 |
| objects-media | 45 | 0 | 0 | 0 |
| oracle-probe | 44 | 0 | 0 | 0 |
| render-tree | 37 | 3 | 0 | 0 |
| security | 80 | 0 | 0 | 0 |
| self-description | 69 | 0 | 5 | 0 |
| serialization | 55 | 0 | 1 | 0 |
| showcase | 6 | 0 | 0 | 0 |
| studio-e2e | 36 | 4 | 0 | 0 |
| table-csv | 19 | 0 | 6 | 0 |
| table-editing | 38 | 0 | 2 | 0 |
| text-editing | 90 | 0 | 0 | 0 |
| work-receipt | 42 | 14 | 0 | 0 |

## 5. 이전 계측 문구 정정

원인 계보의 수치 자체는 맞지만 `reference의 run step 993`을 reference 개수로 읽으면
안 된다. 정확한 단위는 다음과 같다.

| 항목 | 수치 |
|---|---:|
| reference 파일 | 1,035 |
| `run`을 하나 이상 가진 reference | 448 |
| reference 안의 `run` step | 993 |
| answer spec | 686 |
| answer `cmd` | 682 |
| answer `const` | 4 |

원장은 step 수를 task 권위 수로 바꾸지 않는다. 여러 `run` step이 있는 한 task도
원장에서는 정확히 한 entry이고 baseline source도 하나다.

## 6. 보호 불변식과 실패 계약

- task 파일 1개당 entry 1개. 누락 reference나 중복 pack/task key는 실패한다.
- primary authority는 네 클래스 중 하나. 미분류·다중 분류는 실패한다.
- summary는 entries에서 재계산한다. 사람이 별도 합계를 고치지 않는다.
- external/independent 승격에는 repo-contained 증적이 필요하다.
- 현재 rhwp를 호출하는 check는 항상 `self-live` 의존성을 노출한다.
- 결과에는 실행 시각과 절대 경로를 넣지 않아 같은 tree에서 동일 JSON이 나온다.
- 모든 entry는 source 파일과 JSON pointer 또는 증적 파일 locator를 가진다.
- `self-live`와 `independent-fixture`의 caveat를 entry마다 보존한다.

실패 계약 단위 시험은 다중 권위, 외부 증적 누락·부재·repo 이탈, live 권위의 외부
승격, input fixture 부재, baseline 출처 혼합, reference 누락·고아를 각각 음성 사례로
검증한다.

## 7. CI 증적 경계

Gym 관련 PR의 빠른 contract job에 원장 단위 시험을 추가했다. 수동
`Full Gym benchmark validation`은 전건 실행 시작 시 `authority-ledger.json`, stderr와
종료 코드를 `gym-benchmark-evidence` artifact에 넣는다. 원장이 실패하면 수동 Gym
감사는 fail-closed다.

이 workflow는 여전히 `workflow_dispatch`에서만 전건 실행하며 devel/main push, tag,
Release Binary, npm·extension 게시가 원장을 소비하지 않는다. 권위의 정직성을 Gym
안에서 지키되 제품 릴리스 게이트로 되돌리지 않는다.

## 8. 검증 결과

```text
python3 -m unittest scripts.tests.test_gym_authority_ledger
Ran 18 tests — OK

python3 gym/tools/authority_ledger.py
Gym 정답 권위 원장: 1035/1035 task 분류
  authority: self-live 987 · contract-constant 28 · independent-fixture 20 · external-oracle 0
  baseline: self-live 1031 · contract-constant 4
  판정: 위반 0 — self-live는 독립 제품 정답이 아님

python3 gym/tools/audit.py --json
21 pack · task 1035 · reference 1035 · issue 0 — OK
```

추가 검증 결과:

- 권위 원장 + workflow 구조 계약: 26/26 통과
- 같은 tree의 원장 JSON 2회 생성 바이트 대조: 일치
- workflow YAML BaseLoader 파싱: 통과
- 변경 문서 5개 내부 상대 링크: 이상 없음
- Markdown 링크 검사 자체 시험: 5/5 통과
- `git diff --check`: 통과
- committed/uncommitted Rust source·Rust test diff: 없음

GitHub의 빠른 Gym contract job과 동일한 모듈 목록은 1,617건을 실행해 기존 tutorial
날짜 정문 snapshot 1건 실패, 1건 skip을 보고했다. Stage 0 전수 기준선의 다섯 번째
실패군과 동일하며 Stage 1이 추가한 `test_gym_authority_ledger` 18건은 모두 통과했다.
기존 실패를 통과로 위장하거나 Stage 1 범위를 넘어 기대값을 바꾸지 않았고, 승인된
Stage 2에서 날짜 문자열 대신 front matter 형식·정본 관계를 검증하도록 정상화한다.

## 9. Stage 1 판정과 다음 단계

Stage 1 종료 게이트를 충족했다.

- 1,035 task의 중복·누락 없는 전수 분류: 충족
- 분류 근거와 caveat의 기계 판독 보존: 충족
- 집계의 entry 재계산: 충족
- `self-live`의 독립 제품 정답 승격 방지: 충족
- 현행 외부 오라클 공백의 명시: 충족

다음 Stage 2는 전수 Python 계약의 기존 5 failure·4 error를 원인별로 정상화한다.
Stage 1 결과만으로 pack-health 예외를 승인하거나 테스트 기대값을 바꾸지 않는다.
