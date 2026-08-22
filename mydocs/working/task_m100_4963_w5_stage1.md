---
kind: working-note
status: completed
issue: 4963
stage: W5-1
last_verified: 2026-08-22
---

# Task M100 #4963 W5 Stage 1 — Oracle Profile 계약·schema

- **이슈**: [#4963](https://github.com/edwardkim/rhwp/issues/4963)
- **계획**: [`task_m100_4963.md`](../plans/task_m100_4963.md)
- **선행 작업**: [#4962](https://github.com/edwardkim/rhwp/issues/4962) W4 완료
- **브랜치**: `task_m100_4963`
- **착수 기준**: `upstream/devel@4bd9c5d60efe85d80e935ff76ed5441c17d56699`
- **단계 상태**: W5-1 완료, W5-2 승인 대기

## 1. 결론

W4가 넘긴 17개 face와 다섯 exact/missing 질문을 byte/hash로 고정하고, 한컴 Oracle 실행 하나가
입력·환경·font state·관계·관찰 계보를 모두 보존하도록 JSON Schema와 실행 validator를 구현했다.

이 단계는 실제 font나 PDF 값을 수집하지 않았다. 공개 정상 fixture는
`synthetic-contract-fixture`/`contract-fixture`로 표시되어 Oracle evidence로 오인할 수 없다. 제품 font
metric DB, fallback, paint, HWP/HWPX renderer도 변경하지 않았다.

## 2. 동결한 입력

| 입력 | SHA-256·판정 |
| --- | --- |
| W4 공개 ranking file | `6947e9e8…0bb4ee3a`, exact |
| W4 canonical output | `95e7a41d…da111a68`, exact |
| W4 action queue | 17개 face와 action rank exact |
| W4 위험 문자 | 1,562,076, exact |
| W4 base risk mass | 7,015,182 / 810,374 ppm, exact |
| 한컴 2022 `EVIDENCE.md` | `975e8278…d0dd5e`, exact |
| 한컴 2022 HFT preflight TSV | `d8204207…f4abed`, exact |

따라서 Stage W5-1은 W3/W4 10k 계측이나 기존 한컴 2022 ASCII 계측을 다시 실행하지 않았다.

## 3. Profile 계약

### 3.1 증거 상태

| 상태 | 값 | 이유 |
| --- | --- | --- |
| `observed` | 반드시 존재 | `null` |
| `unavailable` | 반드시 `null` | 필수 |
| `not-applicable` | 반드시 `null` | 필수 |
| `blocked` | 반드시 `null` | 필수 |

필수 관찰 필드 자체를 `null`로 두거나 생략하면 실패한다. 이렇게 하면 관찰 실패를 0 또는 fallback
성공처럼 채우지 못한다.

### 3.2 advance 분리

`hmtxAdvance`는 다음 source SFNT identity를 요구한다.

- integer advance
- units-per-em
- face index
- source font SHA-256

`pdfObservedAdvance`는 PDF user-space advance와 glyph/CID identity를 별도로 기록한다. 테스트는 PDF
envelope를 `hmtxAdvance`에 복사한 경우와 반대 경우를 모두 거부한다. HFT·Type3·source-unavailable
상태에서는 `hmtx` 값을 추정하지 않고 명시적 status/reason을 사용한다.

### 3.3 관계와 권위

일곱 관계 enum을 각각 유지한다.

```text
identity-exact
identity-alias
official-successor
document-substitution
metric-surrogate
hancom-missing-font
unknown
```

`unknown` 외 확정 관계는 observed direct anchor가 없으면 실패한다. 특히 이름 유사성만 있는
`official-successor`는 통과하지 않는다.

증거 권위도 다음 세 종류로 분리했다.

| evidence class | authority |
| --- | --- |
| `synthetic-contract-fixture` | `contract-fixture` |
| `historical-import` | `secondary-historical` |
| `oracle-run` | `acceptance-primary` |

acceptance Oracle run은 reset된 한컴 process를 요구한다. 이 분리는 제품의 한컴 version 분기가 아니라
실험 결과의 증거 등급을 판정하는 절차다.

## 4. RED·GREEN과 검증

구현 전 요구사항을 9개 public negative mutation으로 고정했다.

1. 필수 `hmtx` evidence 누락
2. `hmtx` 필드 자체의 plain `null`
3. PDF width를 `hmtx`로 표기
4. direct anchor 없는 official successor
5. question/state 불일치
6. `observed`이지만 값 없음
7. `unavailable`이지만 이유 없음
8. W4 queue rank/face 불일치
9. font bytes 공개 flag

실행 결과:

```text
node --test scripts/tests/oracle_profile_contract.test.mjs
tests 12, pass 12, fail 0

node scripts/oracle_profile_contract.mjs check
ok true, frozenQueueFaces 17, negativeFixtures 9

Draft 2020-12 schema compile
schema OK, valid public profile OK
```

추가로 `git diff --check`와 신규 문서 link 검사를 통과시킨다. Cargo/Rust source를 변경하지 않았고
private corpus·font bytes를 읽거나 공개하지 않았다.

## 5. 보호 불변식 확인

- W4 public artifact와 기존 2022 evidence hash drift 시 fail-closed
- question ID와 exact/missing state exact 일치
- document face와 W4 action rank exact 일치
- `hmtx`와 PDF observed advance 분리
- relation direct anchor 없는 확정 관계 거부
- absolute local font path와 privacy flag 위반 거부
- 한컴 2010을 acceptance Oracle로 승격하지 않음
- host font 설치·제거·font cache 변경 없음
- 제품 metric·fallback·paint·renderer 변경 없음
- 원격 push·PR·GitHub 이슈 변경 없음

## 6. 다음 승인 지점

Stage W5-2는 deterministic HWPX fixture, SFNT/PDF analyzer와 readiness ledger를 구현한다. 이때
KoPubWorld 공식 공급 경로를 현재 시점에 다시 확인하고, 승인된 bytes는 저장소 밖 ttfs에만 보관한다.
외부 font 취득은 Stage W5-2 승인 범위에서만 수행하며, system font 설치·제거는 여전히 W5-4의 별도
강제 정지 게이트다.
