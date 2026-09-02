---
kind: working
status: completed
canonical: mydocs/plans/task_m100_4939.md
last_verified: 2026-08-16
---

# Task M100 #4939 Stage 4 — evidence 판정과 Font Rule Ledger

## 1. 종료 판정

Stage 4는 완료했다. Stage 3의 source candidate 1,352개는 1,507개 ledger rule에 전부 대응한다.
증가분 155개는 임의 중복이 아니라 다음 승인된 profile split이다.

| split 대상 | candidate | rule | 이유 |
| --- | ---: | ---: | --- |
| Studio `FONT_LIST` | 153 | 306 | Canvas2D CSS supply와 CanvasKit SFNT supply 분리 |
| 정부상징 oracle matrix | 1 | 3 | source exact, official successor, Hancom missing-font 분리 |
| 나머지 | 1,198 | 1,198 | candidate당 정확히 한 rule |
| **합계** | **1,352** | **1,507** |  |

validator 결과는 누락 0, 설명 없는 중복 0, order 중복 0, orphan evidence 0,
근거 없는 identity 승격 0이다. 제품 renderer·Studio source는 수정하지 않았다.

## 2. 입력과 기준

- candidate snapshot: `font_rule_candidates.json`, 1,352개
- snapshot source commit: `795e7b5fac24cfef79017e9120516570851a03b2`
- Stage 3 commit: `3c7e750df4631daff400424deab553d6b8568df1`
- 원인 계보: `mydocs/report/font_metrics_fallback_causal_lineage_20260816.md`
- 정부상징 bytes/oracle: #4739 successor matrix
- capability 분리 근거: #4741 Local Font Access 조사
- schema: Stage 1의 `font_rule_ledger.schema.json`

각 ledger 행은 candidate JSON anchor, #4939, source commit, 수행계획, 원인 계보를 공통 evidence로
가진다. 관계별로 관련 issue, 조사 문서, test와 허용된 font SHA-256을 추가했다. 외부 font bytes와
private corpus 정보는 원장에 넣지 않았다.

## 3. relation 판정

| relation | rule | 핵심 판정 |
| --- | ---: | --- |
| `metric-entry` | 600 | 현재 exact table entry와 lookup test |
| `metric-surrogate` | 24 | source 주석이 근사임을 명시하는 Source Han/HY각헤드라인 계열 |
| `style-fallback` | 181 | 이름·style·backend ordered selection |
| `paint-substitute` | 269 | layout metric을 바꾸지 않는 paint 해소 |
| `official-successor` | 11 | 정부상징 legacy 부재 조건의 ROKG successor |
| `document-substitution` | 1 | HWP/HWPX `substFont` 위치 |
| `generic-fallback` | 2 | terminal family classifier/chain |
| `measured-overlay` | 1 | face·size-gated Hancom regenerated space |
| `supply-source` | 361 | asset, webfont, CanvasKit supply |
| `capability-detection` | 9 | enumeration/probe/resource/backend capability |
| `oracle-observation` | 4 | test anchor와 정부상징 3 profile |
| `unknown` | 44 | SFNT/metric provenance 또는 heuristic 분해가 필요한 규칙 |

`identity-alias`는 0개다. 한국어 이름과 영문 DB 이름이 유사하거나 source와 target 문자열이 같다는
사실만으로 byte identity를 주장하지 않았다. 일반 metric name mapping 43개와 generic width estimator
1개는 질문을 원장과 요약에 남기고 `unknown`으로 보존했다.

## 4. profile과 backend 경계

### 4.1 정부상징 oracle

동일 evidence anchor를 다음 세 행으로 분리했다.

1. `source-exact`: 구형 face 설치 환경
2. `official-successor`: 구형 face 부재, ROKG 설치 환경
3. `hancom-missing-font`: 한컴바탕/Haansoft Batang PDF 관찰 환경

구형 TTF와 ROKG_R의 SHA-256을 exact/successor evidence에 연결했지만 동일 byte라고 판정하지 않았다.
missing-font PDF는 source exact의 정답지로 사용하지 않는다.

### 4.2 Canvas2D와 CanvasKit

Studio `FONT_LIST`의 각 항목을 Canvas2D CSS source와 CanvasKit SFNT source로 분리했다. 153개 중
CanvasKit source가 없는 항목도 행을 삭제하지 않고 `unavailable` 정책과 fail-closed 한계를 기록했다.
따라서 CSS 이름으로 paint 가능한 상태가 CanvasKit Typeface 조달 성공으로 승격되지 않는다.

## 5. 상충 target과 cycle 감사

동일 source/condition에서 target이 둘 이상인 group은 14개다.

- 6개는 metric lookup, native replay, installed successor, Studio display/detection처럼 source가 이미
  ordered chain으로 정의한다.
- 8개는 Studio `SUBST_TABLES`의 동일 key 다중 target이다. runtime은 `Map`을 만들 때 첫 entry만
  보존하므로 candidate의 물리 배열 순서를 ledger `order` 0, 1로 복원했다.

동일 group의 order 중복은 없다. graph audit에서 self-loop 5개를 찾았다.

- Rust metric: `D2Coding`, `Gowun Batang`, `Gowun Dodum`, `Pretendard`
- Studio substitution: `휴먼명조`

Rust의 네 행은 single-pass canonical spelling 반환이고 Studio 행은 visited-set과 15단계 상한으로
종료한다. 다단 순환은 0개다. 모든 self-loop는 `knownLimitations`에 비-identity 의미와 종료 조건을
기록했다.

## 6. 구현 산출물

- `scripts/font_rule_ledger_evidence.mjs`
  - candidate별 relation/evidence 판정
  - 허용 profile split
  - source physical order 복원
  - candidate coverage, conflict/order, cycle, identity, orphan evidence validator
  - canonical ledger와 Markdown summary 생성
- `scripts/tests/font_rule_ledger_evidence.test.mjs`
  - missing coverage, 불법 identity 승격, order 누락, orphan evidence/test, 미설명 cycle RED
- `font_rule_ledger.json`
  - 1,507개 조사 원장 행
- `font_rule_ledger_summary.md`
  - owner/relation/profile 수량, 44개 unknown 질문, 충돌·순환 disposition

Stage 1/2의 `scripts/font_rule_ledger.mjs`와 W0 baseline 수집 입력은 수정하지 않았다.

## 7. Stage 4 검증

다음 명령을 실행했다.

```bash
node --test scripts/tests/font_rule_ledger_evidence.test.mjs
node scripts/font_rule_ledger_evidence.mjs build \
  --candidates mydocs/tech/investigations/issue-4939/font_rule_candidates.json \
  --ledger mydocs/tech/investigations/issue-4939/font_rule_ledger.json \
  --summary mydocs/tech/investigations/issue-4939/font_rule_ledger_summary.md
node scripts/font_rule_ledger_evidence.mjs check \
  --candidates mydocs/tech/investigations/issue-4939/font_rule_candidates.json \
  --ledger mydocs/tech/investigations/issue-4939/font_rule_ledger.json
```

현재 결과:

- Stage 4 RED/positive test: 8/8 PASS
- Stage 1~4 Node test: 25/25 PASS
- Markdown link check: 5개 문서, 이상 없음
- candidate: 1,352
- ledger rule: 1,507
- validator error: 0
- documented conflict group: 14
- documented cycle: 5 self-loop, multi-node cycle 0
- canonical ledger SHA-256:
  `284afd72259eb0e8465ff6f10da4e6285d792d73dcde5cfa90daa8e4520b8c23`

전체 Stage 1~4 test와 Markdown link 검사는 commit 전에 다시 실행한다.

## 8. 잔여 위험과 다음 단계

- 44개 unknown은 누락이 아니라 의도적으로 보존한 후속 조사 queue다.
- ledger evidence는 현재 source와 test를 설명하지만 font rule runtime registry가 아니다.
- W0 baseline byte 동일성과 제품 source 0-delta는 Stage 5에서 다시 감사한다.
- native/WASM public fixture parity와 focused Rust/Studio/asset test는 Stage 5 최종 게이트다.

Stage 4 변경을 별도 commit으로 고정한 뒤 Stage 5 진행은 메인테이너의 다음 승인을 기다린다.
