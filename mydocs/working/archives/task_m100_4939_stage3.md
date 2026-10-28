---
kind: working-note
status: completed
issue: 4939
stage: 3
last_verified: 2026-08-16
---

# Issue #4939 Stage 3 — W1 rule candidate 전수 수집

## 결론

Stage 2의 30개 source boundary를 1,352개 실제 rule candidate로 확장했다. 모든 boundary는
양수 candidate와 추출 strategy를 가진 `extracted` disposition으로 닫혔고, 미인식 실행 mapping
block은 0개다. relation type과 evidence status는 Stage 4 전까지 판정하지 않는다.

## 입력과 generator 경계

- Stage 2 collector commit: `795e7b5fac24cfef79017e9120516570851a03b2`
- Stage 2 snapshot commit: `10aa08c13`
- W0 source boundary: 12개 owner, 30개 selector
- W1 generator: `scripts/font_rule_candidates.mjs`, version `3.0.0`

W0 generator인 `scripts/font_rule_ledger.mjs`와 그 test는 수정하지 않았다. W1 collector를 별도
모듈로 둔 이유는 Stage 2 baseline이 generator bytes를 input digest로 고정했기 때문이다. 이 경계를
깨면 Stage 5의 W0 byte 재현이 불가능해진다.

## RED→GREEN

Stage 3 test를 먼저 추가했을 때 다음 오류로 실패했다.

```text
ERR_MODULE_NOT_FOUND: scripts/font_rule_candidates.mjs
```

구현 뒤 Stage 1~3 계약을 함께 실행한 결과다.

```text
node --test \
  scripts/tests/font_rule_candidates.test.mjs \
  scripts/tests/font_rule_ledger.test.mjs

tests 17
pass 17
fail 0
```

## 추출 전략

| source shape | 처리 |
| --- | --- |
| Rust `"A" | "B" => Some("X")` | 좌변별 finite mapping 행으로 확장 |
| metric alias match | source name별 finite mapping 행으로 확장 |
| `FONT_METRICS` | 물리 순서의 600 metric-entry 행 보존 |
| Studio `SUBST_TABLES` | language·source type·target type별 265행 보존 |
| Studio `FONT_LIST` | family·file·format·CanvasKit file별 153 supply 행 보존 |
| installed alias와 선택 사다리 | edge별 ordered-chain 행과 `order` 보존 |
| classifier·measurement·capability 함수 | 무한 입력을 열거하지 않고 symbol 전체를 predicate 행으로 보존 |
| asset·license Markdown 표 | runtime fallback과 분리된 supply-source 행 |
| test·historical 문서 | 실행 규칙이 아닌 evidence anchor 행 |

구조 파서는 각 mapping block의 원시 개수와 해석한 block 수를 비교한다. 지원하지 않는 새 문법이
들어오면 조용히 누락하지 않고 해당 selector에서 실패한다.

## 전수 결과

### candidate kind

| kind | count |
| --- | ---: |
| `metric-entry` | 600 |
| `finite-mapping` | 506 |
| `supply-source` | 204 |
| `ordered-chain` | 27 |
| `predicate` | 13 |
| `evidence` | 2 |
| 합계 | 1,352 |

### decision plane

| plane | count |
| --- | ---: |
| `layout-metric` | 672 |
| `paint` | 290 |
| `supply` | 208 |
| `layout-name` | 171 |
| `detection` | 6 |
| `backend-resource` | 3 |
| `oracle` | 2 |

### owner

| owner | count |
| --- | ---: |
| `rust-metric` | 670 |
| `studio-substitution` | 270 |
| `rust-style-resolution` | 172 |
| `studio-supply` | 157 |
| `asset-authority` | 51 |
| `rust-paint-chain` | 14 |
| `native-skia` | 5 |
| `studio-detection` | 5 |
| `paint-resource` | 2 |
| `rust-measurement` | 2 |
| `studio-canvas-patch` | 2 |
| `tests-history` | 2 |

핵심 population은 `FONT_METRICS` 600, `SUBST_TABLES` 265, `FONT_LIST` 153으로 source 선언과
정확히 일치한다.

## 결정성과 W0 보호

```bash
node scripts/font_rule_candidates.mjs collect \
  --in mydocs/tech/investigations/issue-4939/font_rule_candidates.json \
  --out mydocs/tech/investigations/issue-4939/font_rule_candidates.json
```

같은 파일에 두 번 생성한 SHA-256은 모두
`0c5316fcb0bad11e7af17586062486fcf6a26206a478da1fd9bb641c1aa9474a`였다.

확장된 candidate를 `buildBaseline`에 입력해 메모리에서 재생성한 W0 SHA-256은
`a0fac05c3138471eb3e7404fc701f0053caa6c01a923afae60fd4da64064b466`이며 기존 baseline과
byte 동일하다. Stage 3 정보는 W0가 소비하는 boundary projection을 바꾸지 않는다.

## fail-closed 계약

- 30개 boundary 모두 disposition 존재
- `extracted` disposition의 candidate count는 반드시 양수
- disposition count와 실제 행 수 일치
- 모든 candidate ID 고유
- 모든 candidate가 알려진 boundary를 참조
- candidate source digest와 boundary/current file digest 일치
- owner별 candidate 또는 명시적 `not-applicable` 필수
- unrecognized mapping block 0
- supply·detection·runtime fallback decision plane 혼합 금지 test

## 한계와 다음 판정

- candidate는 source 사실의 inventory이며 관계 의미가 옳다는 판정이 아니다.
- metric alias, Studio substitution과 webfont supply가 같은 face를 가리켜도 아직 하나의 규칙으로
  합치지 않는다.
- Markdown 표의 51행은 supply·license authority 후보이지 runtime fallback이 아니다.
- predicate 행은 무한 입력의 대표 단위이며 finite 입력 수처럼 해석하지 않는다.
- `unknown`, historical, test, byte/oracle evidence 판정은 Stage 4에서 수행한다.

## 작업 경계

- `src/`, `rhwp-studio/src/`, `web/` 제품 source 변경 없음
- font metric 값, alias target, fallback order와 asset 변경 없음
- private corpus 접근·재측정·식별 정보 기록 없음
- 원격 push 없음

## 다음 승인 지점

Stage 4는 1,352개 candidate에 relation type과 evidence status를 판정하고, 같은 source·condition의
충돌, 순환 chain, order 중복과 orphan evidence를 검사한다. 설명 없는 누락·중복·identity 승격 0개가
종료 조건이다. 메인테이너 승인 전에는 시작하지 않는다.
