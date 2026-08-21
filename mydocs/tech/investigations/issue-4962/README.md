---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_4962.md
last_verified: 2026-08-21
---

# Issue #4962 W3 font metric coverage 계약

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
| `scripts/font_metric_coverage_contract.mjs` | 분류·대사·hash·privacy·POC/W1 drift 검사 |
| `scripts/tests/font_metric_coverage_contract.test.mjs` | 정상·변이·누락을 다루는 Stage 1 계약 test |

## 권위와 경계

- [수행계획](../../../plans/task_m100_4962.md)이 범위와 승인 게이트의 정본이다.
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
  --manifest output/poc/font-metric-coverage/full-manifest-stage4-a-v1.json \
  --preflight output/poc/font-metric-coverage/full-manifest-preflight-stage4-a-v1.json \
  --source-head <full-commit>

node scripts/font_metric_coverage_checkpoint_runner.mjs \
  --manifest output/poc/font-metric-coverage/full-manifest-stage4-a-v1.json \
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
