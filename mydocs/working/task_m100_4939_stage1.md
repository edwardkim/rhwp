---
kind: working-note
status: completed
issue: 4939
stage: 1
last_verified: 2026-08-16
---

# Issue #4939 Stage 1 — schema와 source boundary RED

## 결론

Font Rule Ledger의 조사용 JSON schema와 source boundary를 실행 가능한 계약으로 고정했다.
실제 font mapping 수집, baseline 생성과 제품 동작 변경은 수행하지 않았다.

## 입력 기준

- 작업 브랜치: `local/task4939-font-rule-ledger`
- `upstream/devel`: `82f28ae8644110d4ccd1528447ab87ddf8ddce6f`
- 선행 계보 보고서 commit: `6b9f9d1957d2bce045a7c1c032162a99c2e3a03e`
- 승인된 수행계획: `mydocs/plans/task_m100_4939.md`

## RED

계약 test와 fixture를 먼저 추가한 뒤 다음 명령을 실행했다.

```bash
node --test scripts/tests/font_rule_ledger.test.mjs
```

validator가 아직 없는 상태에서 `ERR_MODULE_NOT_FOUND`로 실패했다. 따라서 기존 구현이 우연히
새 계약을 만족했다고 간주하지 않았고, Stage 1 구현 대상이 test로 먼저 드러났다.

## 구현

### 조사 원장 schema

`font_rule_ledger.schema.json`에 다음을 고정했다.

- `decisionPlane`, `relationType`, `evidenceStatus`, backend와 rule status enum
- 안정적인 의미 ID인 `ruleId`와 line number 대신 symbol·selector를 쓰는 `sourceLocation`
- `sourceFace`와 `targetFaceOrPolicy`, 조건, backend, order, evidence, test와 한계
- 빈 `evidenceStatus` 금지와 `unknown`의 명시적 허용
- 추가 필드를 조용히 수용하지 않는 `additionalProperties: false`

### source boundary

`font_rule_sources.json`은 12개 owner와 30개 literal selector를 선언한다.

| owner 구분 | owner 수 | selector 수 |
| --- | ---: | ---: |
| Rust layout·metric·paint·resource | 6 | 16 |
| Studio substitution·supply·detection·Canvas patch | 4 | 9 |
| asset authority | 1 | 3 |
| tests·history | 1 | 2 |
| 합계 | 12 | 30 |

Stage 1 validator는 모든 필수 owner가 존재하는지, repository-relative path가 checkout 밖으로
나가지 않는지, 파일이 존재하는지, 각 literal selector가 `minMatches` 이상 발견되는지 검사한다.
selector가 사라지면 candidate 0건으로 성공하지 않고 오류를 반환한다.

### 규칙 형태 fixture

fixture는 source 문법 전체를 흉내 내지 않고 후속 collector의 행 확장 계약만 고정한다.

- grouped mapping 1개를 source별 2행으로 확장
- ordered fallback chain 1개를 `order: 0`, `order: 1`의 2행으로 확장
- 무한 입력 algorithmic predicate 1개를 열거하지 않고 정책 1행으로 보존

총 5행은 schema를 통과하며 모두 `evidenceStatus: unknown`, `status: candidate`로 명시된다.
이는 실제 rhwp 규칙이나 evidence 판정을 대신하지 않는다.

## GREEN과 종료 게이트

```text
node --test scripts/tests/font_rule_ledger.test.mjs
tests 7, pass 7, fail 0

node scripts/font_rule_ledger.mjs boundary \
  --sources mydocs/tech/investigations/issue-4939/font_rule_sources.json
font rule source boundary: ok
```

검증된 실패 계약은 다음과 같다.

- 필수 owner 누락
- source symbol selector 0건
- 중복 `ruleId`
- 허용되지 않은 enum 값
- 빈 `evidenceStatus`

따라서 수행계획의 Stage 1 종료 게이트를 만족한다.

## 행동 불변과 제외 확인

- `src/`, `rhwp-studio/src/`, `web/` 제품 source 변경 없음
- metric 값, alias target, fallback order와 font asset 변경 없음
- private 10k corpus 접근·재계측·식별 정보 기록 없음
- candidate collector, canonical writer와 W0 baseline은 Stage 2 승인 전 미구현

## 다음 승인 지점

Stage 2는 이 계약 위에서 deterministic candidate collector의 공통 기반과 W0 baseline을 구현한다.
같은 HEAD의 반복 생성 byte 일치, source digest와 lookup projection 폐합, 기존 focused test 및 공개
native/WASM parity가 종료 게이트다. 메인테이너 승인 전에는 시작하지 않는다.
