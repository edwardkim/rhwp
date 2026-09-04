---
kind: investigation
status: active
canonical: mydocs/plans/archives/task_m100_4962_w4.md
last_verified: 2026-08-22
---

# Issue #4962 W3 font metric coverage·W4 조판 위험 계약

이 디렉터리는 기존 10k 폰트 편집 습관 POC를 재사용하면서, 그 집계에서 복원할 수 없는 실제 renderer
문자 결정만 delta로 측정하기 위한 계약을 보존한다. 이 JSON은 조사·계측 계약이며 제품 runtime의
폰트 registry나 fallback 정책이 아니다.

## 산출물

| 파일 | 역할 |
| --- | --- |
| `font_metric_coverage_contract.schema.json` | W3 분류·분모·privacy·기존 자산 기준선 schema |
| `font_metric_coverage_contract.json` | 7개 분류 우선순위, W2 입력 inventory와 POC/W1 동결값 |
| `font_metric_coverage_checkpoint_policy.json` | crash-consistent journal·state·identity·storage 정책 |
| `font_metric_coverage_finalizer_policy.json` | format 보존 usage key와 corpus 병합·hash 정책 |
| `font_metric_coverage_full_manifest_policy.json` | local-only 10k 발견·BLAKE3·저장공간 preflight 정책 |
| [`tools/diagnostics/font_metric_coverage_worker.rs`](../../../../tools/diagnostics/font_metric_coverage_worker.rs) | W3 격리 계측 worker source; 반복 실행 중이므로 명시적 Cargo target으로 유지 |
| `scripts/font_metric_coverage_contract.mjs` | 분류·대사·hash·privacy·POC/W1 drift 검사 |
| `scripts/tests/font_metric_coverage_contract.test.mjs` | 정상·변이·누락을 다루는 Stage 1 계약 test |
| `font_typesetting_risk_contract.schema.json` | W4 입력·identity·proxy·lane·risk mass·privacy schema |
| `font_typesetting_risk_contract.json` | W3 동결 입력과 조판 위험 순위 규칙의 실행 전 계약 |
| `scripts/font_typesetting_risk_rank.mjs` | W3 `decisionUsage` 전용 streaming W4 ranker |
| `scripts/tests/font_typesetting_risk_rank.test.mjs` | W4 ranker의 계약·결정성·privacy test |

## 권위와 경계

- [수행계획](../../../plans/archives/task_m100_4962.md)이 범위와 승인 게이트의 정본이다.
- W1 원장과 candidate는 `../issue-4939/`, W2 trace 계약은 `../issue-4961/`에서 읽는다.
- 기존 `output/poc/font-layout-habits/`는 gitignored 로컬 입력이다. 이 디렉터리로 복사하지 않는다.
- 기존 POC의 장평·자간·커닝·문맥·font usage는 재측정하지 않는다.
- private corpus의 원문, 파일명·경로, 문서별 hash와 raw 문자 trace는 기록하지 않는다.
- 현재 단계는 계약만 고정하며 metric 값, lookup 순서, fallback, paint face와 renderer output을 바꾸지
  않는다.

## 분류 순서

문자 하나는 다음 순서에서 처음 만족한 분류 하나에만 들어간다.

1. `measured-overlay`
2. `identity-alias-hit`
3. `metric-surrogate`
4. `exact-hit`
5. `char-miss`
6. `face-miss`
7. `heuristic`

cluster continuation, inline object placeholder, HWP PUA filler, figure space와 tab advance는 font metric
coverage 대상이 아니므로 `not-applicable`로 분리한다. 새 `widthSource`나 모순된
`characterMatch`·`metricEntry` 조합은 임의 분류하지 않고 실패한다.

## 분모

- layout: 실제로 관찰한 모든 문자 결정
- coverage: 7개 분류의 합
- non-applicable·excluded: coverage와 분리
- source join: `joined`·`layoutOnly`·`excluded`의 독립 합
- document parse: 성공과 이유별 실패의 독립 합
- backend: `complete`·`unsupported`·`notObserved`·`failed`의 별도 요청 분모

긴 page의 `truncatedCharacters`는 0이어야 한다. parser·join·backend 실패가 발생하는 것 자체를 다른
분모의 metric miss로 해석하지 않지만, 상태 key나 count를 생략하면 계약 검사가 실패한다.

## 검증

공개 계약과 현재 W1 의미 drift를 검사한다. 이 명령은 private corpus를 읽지 않는다.

```bash
node --test scripts/tests/font_metric_coverage_contract.test.mjs
node scripts/font_metric_coverage_contract.mjs check
```

이미 보존된 로컬 POC aggregate를 재생성 없이 검사할 때만 세 파일을 명시한다.

```bash
node scripts/font_metric_coverage_contract.mjs check \
  --poc output/poc/font-layout-habits/summary-10k-v2.json \
  --poc-hwp output/poc/font-layout-habits/summary-hwp-v2.json \
  --poc-hwpx output/poc/font-layout-habits/summary-hwpx-v2.json
```

이 검사는 원문이나 risk document를 출력하지 않는다. corpus·totals·schema key·문자 합과 비식별
projection hash, HWP+HWPX 가산성만 판정한다.

Stage 4 실행 준비에는 developer-only checkpoint runner·finalizer와 full manifest builder를 사용한다.
manifest와 checkpoint는 반드시 gitignored `output/` 아래에 두며 Issue·PR·CI artifact로 게시하지 않는다.

```bash
node scripts/font_metric_coverage_full_manifest.mjs \
  --corpus-root <private-corpus-root> \
  --manifest output/poc/font-metric-coverage/full-manifest-stage4-c-v2.json \
  --preflight output/poc/font-metric-coverage/full-manifest-preflight-stage4-c-v2.json \
  --source-head <full-commit>

node scripts/font_metric_coverage_checkpoint_runner.mjs \
  --manifest output/poc/font-metric-coverage/full-manifest-stage4-c-v2.json \
  --checkpoint-dir output/poc/font-metric-coverage/checkpoint-stage4-r1 \
  --worker target/debug/examples/font_metric_coverage_worker \
  --source-head <full-commit>

node scripts/font_metric_coverage_checkpoint_finalizer.mjs \
  --checkpoint-dir output/poc/font-metric-coverage/checkpoint-stage4-r1 \
  --output output/poc/font-metric-coverage/final-stage4-r1.json
```

full manifest는 content 중복을 제거하지 않는다. 같은 source의 중복만 오류이며 동일 bytes가 여러 문서로
존재하는 경우 기존 corpus 빈도를 보존한다. finalizer는 문서별 usage row에 format을 주입해 HWP/HWPX
축을 유지하고, path·filename·개별 BLAKE3 없이 최종 aggregate와 canonical SHA-256을 만든다.

`inputFormat`은 동결된 10k inventory의 확장자 분모이고 `format`은 HWP OLE/HWPX ZIP 컨테이너
시그니처로 판정한 실행 형식이다. 지원 컨테이너 시그니처가 없으면 실행 형식은 입력 분류를 유지하되
worker가 HWP3·HML·unknown을 `unsupported` 문서 실패로 기록한다. 지원 형식의 확장자와 컨테이너가
엇갈린 문서는 corpus를 이름 변경하지 않고 컨테이너 형식으로 실행하며, worker aggregate와 다르면
checkpoint runner가 계속 fail-closed 한다. preflight에는 경로·파일명 없이 지원 형식 수,
미인식 수, 입력 분류 불일치 수만 남긴다.

## W4 조판 위험 순위 경계

W4는 W3가 보존한 `decisionUsage`를 읽기 전용으로 사용한다. document face는 `font` 문자열을 exact
identity로 유지하고 `metricRequestedFace`는 별도 원인 cluster로만 집계한다. 위험 category는
`face-miss`·`char-miss`·`heuristic`뿐이며 장평·자간과 문맥은 같은 usage row에서 관찰된 경우에만 risk
mass에 반영한다.

`fixedFrameContextProxy`는 historical generator의 context bit 집합을 재사용하는 proxy일 뿐 geometry나
overflow 판정이 아니다. `storedLineSeg`도 유효성 판정이 아니므로 `stored-line-lane`과
`fresh-candidate-lane`으로만 나누고 점수 배수에는 사용하지 않는다. 계약과 schema는 다음 파일에 있다.

```text
mydocs/tech/investigations/issue-4962/font_typesetting_risk_contract.json
mydocs/tech/investigations/issue-4962/font_typesetting_risk_contract.schema.json
```

Stage W4-1에서 RED로 고정한 요구사항은 W4-2 ranker 구현 뒤 GREEN이다. 공개 fixture 계약 test는 private
10k aggregate를 열지 않는다.

```bash
node --test scripts/tests/font_typesetting_risk_rank.test.mjs
```

실제 W3 r2를 읽을 때는 입력 경로·mode·bytes·파일 hash·aggregate hash·source를 동결값과 먼저 대사한다.
ranker는 큰 `legacyUsage`를 materialize하지 않고 `decisionUsage` 행만 streaming 처리하며, 결과는 새 파일로만
기록한다.

```bash
node scripts/font_typesetting_risk_rank.mjs \
  --input output/poc/font-metric-coverage/final-stage4-c-10k-r2.json \
  --output output/poc/font-typesetting-risk/rank-stage-w4-2-r1.json
```

실제 110 MB 입력과 local ranking 중간물은 `output/poc/font-typesetting-risk/` 아래 mode `0600`으로만
보존하며 저장소에 추가하지 않는다. `metricRequestedFace=null`은 비슷한 이름으로 추정하지 않고
`preserve-unavailable-cluster` 정책에 따라 별도 unavailable cluster로 보존한다.
