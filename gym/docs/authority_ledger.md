---
kind: guide
status: active
canonical: gym/docs/authority_ledger.md
last_verified: 2026-09-02
---

# Gym 정답 권위 원장

Gym 과제가 무엇을 정답으로 믿는지 감추지 않기 위한 전수 감사 규약이다.
`gym/tools/authority_ledger.py`는 모든 task와 짝 reference를 읽고, 채점의 주된
권위와 기준 제출물의 출처를 서로 다른 축으로 분류한다.

```bash
python3 gym/tools/authority_ledger.py
python3 gym/tools/authority_ledger.py --json > authority-ledger.json
python3 -m unittest scripts.tests.test_gym_authority_ledger
```

위반이 없으면 종료 0, 누락·모순·증적 부재가 하나라도 있으면 종료 1이다. JSON은
절대 경로나 실행 시각을 넣지 않으므로 같은 tree에서 결정론적으로 재생성된다.

## 1. 두 축을 분리하는 이유

채점 권위와 기준풀이 출처는 같은 질문이 아니다.

- `authority`: 제출물을 무엇과 비교해 합격시키는가.
- `baselineSource`: 저장소의 기준 제출물을 무엇으로 생성하는가.

예를 들어 기준풀이는 현재 rhwp 명령으로 산출물을 만들지만, task는 그 파일이 공개
입력 fixture와 달라졌는지만 검사할 수 있다. 이때 기준풀이 출처는 `self-live`, 채점
권위는 `independent-fixture`다. 두 값을 하나로 합치면 기준풀이가 실행됐다는 사실을
독립 정답이 존재한다는 주장으로 오해하게 된다.

## 2. 주 권위 분류

각 task는 다음 네 클래스 중 정확히 하나를 갖는다.

| class | 판정 근거 | 주장할 수 없는 것 |
|---|---|---|
| `self-live` | scoring check가 현재 rhwp를 실행해 제출물·입력을 다시 읽음 | 한컴·명세·독립 구현과의 제품 동등성 |
| `contract-constant` | task 작성자가 고정한 값·형식·제출물 사이 관계 | 외부 제품 정답 |
| `independent-fixture` | 현재 rhwp를 호출하지 않고 공개 입력 fixture와 직접 관계를 검사 | 인용 관계 밖의 문서 의미·조판 정확성 |
| `external-oracle` | 한컴 또는 독립 구현에서 얻은 공개 증적을 명시적으로 인용 | 인용 범위 밖의 정확성 |

분류는 의존성이 가장 강한 근거를 우선하는 보수적 순서를 따른다.

1. scoring check 하나라도 현재 rhwp를 호출하면 `self-live`다.
2. 그렇지 않을 때만 명시된 외부·독립 증적을 적용할 수 있다.
3. 명시 증적이 없고 `differs_from_input`이 공개 입력과 직접 비교하면
   `independent-fixture`다.
4. 나머지는 작성자 내부 계약인 `contract-constant`다.

따라서 live check와 fixture 관계가 함께 있는 task를 더 강한 독립 권위로 승격하지
않는다. 한 task에서 관찰된 모든 근거 종류는 `authoritySignals`에 남기되, primary
`authority`는 하나만 둔다.

## 3. 기준풀이 출처

reference의 `steps`를 읽어 다음 두 값 중 하나로 분류한다.

- `self-live`: `run` 또는 answer의 `cmd`가 현재 rhwp를 실행한다.
- `contract-constant`: answer가 `const`로 작성자 값을 사용한다.

한 task의 기준풀이가 두 출처를 섞으면 `multiple-baseline-source`로 실패한다. 출처를
분리하지 않은 혼합 baseline은 집계 의미가 모호하기 때문이다.

## 4. 명시적 외부 증적

실제로 독립 증적이 추가되면 task JSON에 다음처럼 선언한다.

```json
{
  "authority": {
    "class": "external-oracle",
    "evidence": ["gym/evidence/example/oracle.json"]
  }
}
```

`independent-fixture`도 같은 형식을 쓸 수 있다. 증적은 저장소 안에 추적된 실재
파일이어야 한다. 빈 목록, 존재하지 않는 경로, 저장소 밖 경로는 실패다. 현재 rhwp를
호출하는 check가 있으면 위 선언으로 `external-oracle`이나 `independent-fixture`로
승격할 수 없다.

증적 파일이 있다는 사실만으로 권위가 생기는 것은 아니다. 파일에는 생성 주체,
대상 fixture, 도구·버전, 비교 좌표와 재현 절차가 있어야 하며, 리뷰는 인용 범위가
task의 판정과 실제로 연결되는지 확인해야 한다.

## 5. 현행 전수 결과

2026-09-02 기준 21개 pack, 1,035개 task/reference를 분류한 결과다. 집계는 JSON의
`entries`에서 매번 재계산하며 단위 시험이 수치와 중복·누락을 고정한다.

| 채점 권위 | task 수 |
|---|---:|
| `self-live` | 987 |
| `contract-constant` | 28 |
| `independent-fixture` | 20 |
| `external-oracle` | 0 |
| 합계 | 1,035 |

| 기준풀이 출처 | task 수 |
|---|---:|
| `self-live` | 1,031 |
| `contract-constant` | 4 |
| 합계 | 1,035 |

`external-oracle = 0`은 누락을 숨긴 결과가 아니라 현재 Gym의 권위 한계를 드러내는
계측 결과다. Gym은 현행 rhwp의 CLI/API 활용과 벤치마크 내부 판별력을 측정할 수
있지만, 이 원장만으로 한컴 조판 동등성이나 제품 릴리스 적합성을 증명하지 않는다.

## 6. JSON 봉투와 실패 조건

봉투의 핵심 필드는 다음과 같다.

- `kind: gymAuthorityLedger`, `schemaVersion: 1.0`
- `taskCount`, `referenceCount`, `entryCount`, `entries`
- `summary.byAuthority`, `summary.byBaselineSource`
- `issueCount`, `issues`, `ok`, `exit`

각 entry는 pack/task key, task/reference 상대 경로, primary authority, 관찰 신호,
근거 locator, 기준풀이 출처와 caveat를 가진다. locator는 JSON pointer까지 기록하므로
수치를 만든 source 위치를 다시 찾을 수 있다.

다음은 fail-closed다.

- task 또는 짝 reference 누락·고아 reference·읽기 실패·ID 불일치
- 빈 checks, 등록되지 않은 check 연산자, 유효하지 않은 live command
- 미분류·다중 authority, live check와 명시 권위의 충돌
- 독립·외부 증적의 빈 목록·부재·저장소 이탈
- 기준풀이 출처의 미분류·혼합
- task 수와 entry 수 불일치, 중복 key, entry에서 재계산한 summary 불일치

수동 `Gym Benchmark Validation`은 JSON 원문과 stderr, 종료 코드를 artifact로
30일 보존한다. 이 증적은 Gym 자체의 감사 자료이며 제품 release/main/devel 승격
조건이 아니다.
