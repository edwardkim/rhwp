---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_4963.md
last_verified: 2026-08-22
---

# Issue #4963 W5 Oracle Profile·controlled ladder

이 디렉터리는 W4 조판 위험 상위 17개 face를 한컴 exact/missing 상태에서 비교하기 위한 기계 검증
계약을 보존한다. 현재 Stage W5-1은 profile 형식과 fail-closed 경계만 고정했다. 실제 한컴 출력이나
제품 font metric·fallback·paint 결과를 포함하지 않는다.

## 산출물

| 파일 | 역할 |
| --- | --- |
| `oracle_profile_contract.json` | W4 입력 hash·17개 queue·ladder·관계·환경·privacy 계약 |
| `oracle_profile.schema.json` | 개별 Oracle Profile JSON Schema Draft 2020-12 |
| `oracle_profile_public_fixtures.json` | Oracle 결과가 아닌 공개 synthetic 정상 fixture와 9개 negative mutation |
| `scripts/oracle_profile_contract.mjs` | 계약·schema·W4 handoff·profile·negative fixture 실행 검사 |
| `scripts/tests/oracle_profile_contract.test.mjs` | 상태·advance·관계·권위·privacy 회귀 test |

## 핵심 경계

- `observed`는 실제 값이 있어야 한다.
- `unavailable`, `not-applicable`, `blocked`는 값이 `null`이고 이유가 있어야 한다.
- PDF에서 관찰한 CID/glyph advance와 SFNT `hmtx` advance는 서로 다른 envelope다.
- exact, alias, official successor, document substitution, metric surrogate, Hancom missing-font를
  하나의 fallback 관계로 합치지 않는다.
- `official-successor`를 포함한 모든 확정 relation은 직접 관찰한 anchor가 필요하다.
- synthetic fixture, historical import, acceptance Oracle run을 동일한 증거 등급으로 다루지 않는다.
- 현재 한컴 2010은 secondary/historical뿐이며 acceptance Oracle은 통제된 한컴 2020/2022 환경이다.
- mutable font state는 disposable snapshot과 메인테이너 승인 없이는 실행하지 않는다.

## 검증

다음 검사는 private corpus, font bytes, 한컴 실행과 원격 GitHub 상태를 변경하지 않는다.

```bash
node --test scripts/tests/oracle_profile_contract.test.mjs
node scripts/oracle_profile_contract.mjs check
```

개별 profile은 다음처럼 같은 계약으로 검사한다.

```bash
node scripts/oracle_profile_contract.mjs check --profile <oracle-profile.json>
```

JSON Schema 자체는 Draft 2020-12 validator로 compile한 뒤 public `validProfile`을 검증한다. status별
조건, W4 queue 결합, relation direct anchor와 `hmtx`/PDF advance 분리는 실행 validator가 추가로
검사한다.

## 다음 단계

Stage W5-2는 이 계약 위에서 deterministic HWPX fixture, SFNT/PDF analyzer와 17개 readiness ledger를
구현한다. KoPubWorld bytes의 외부 취득은 Stage W5-2 승인 뒤 저장소 밖
`/home/edward/mygithub/ttfs/kopubworld`에서만 수행한다.
