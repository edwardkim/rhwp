---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_4939.md
last_verified: 2026-08-16
---

# #4939 Font Rule Ledger 요약

## 판정 결과

- source candidate: **1,352개**
- ledger rule: **1,507개**
- 허용된 profile split candidate: **154개**
  - Studio `FONT_LIST` 153개는 Canvas2D CSS supply와 CanvasKit SFNT supply를 분리했다.
  - 정부상징 oracle 1개는 source exact, official successor, Hancom missing-font 3개 profile로 분리했다.
- 미확정 relation 또는 evidence: **44개**
- 설명된 상충 target group: **14개**
- 설명된 cycle: **5개**
- validator error: **0개**
- ledger canonical SHA-256: `284afd72259eb0e8465ff6f10da4e6285d792d73dcde5cfa90daa8e4520b8c23`

원장은 현재 source를 설명하는 investigation snapshot이다. runtime registry가 아니며 제품 코드가 이
JSON을 import하지 않는다. `identity-alias`는 SFNT/byte 근거 없이는 한 건도 승격하지 않았다.

## relation별 수량

| relation | rule |
| --- | ---: |
| `capability-detection` | 9 |
| `document-substitution` | 1 |
| `generic-fallback` | 2 |
| `measured-overlay` | 1 |
| `metric-entry` | 600 |
| `metric-surrogate` | 24 |
| `official-successor` | 11 |
| `oracle-observation` | 4 |
| `paint-substitute` | 269 |
| `style-fallback` | 181 |
| `supply-source` | 361 |
| `unknown` | 44 |

## owner별 수량

| owner | rule |
| --- | ---: |
| `asset-authority` | 51 |
| `native-skia` | 5 |
| `paint-resource` | 2 |
| `rust-measurement` | 2 |
| `rust-metric` | 670 |
| `rust-paint-chain` | 14 |
| `rust-style-resolution` | 172 |
| `studio-canvas-patch` | 2 |
| `studio-detection` | 5 |
| `studio-substitution` | 270 |
| `studio-supply` | 310 |
| `tests-history` | 4 |

## evidence status별 수량

| evidence status | rule |
| --- | ---: |
| `historical` | 340 |
| `inferred` | 1 |
| `unknown` | 43 |
| `verified-by-bytes` | 12 |
| `verified-by-oracle` | 1 |
| `verified-by-test` | 1110 |

## backend/profile 분리

| profile | rule |
| --- | ---: |
| `canvas2d` | 272 |
| `canvas2d-css-opentype` | 7 |
| `canvas2d-css-truetype` | 9 |
| `canvas2d-css-unknown` | 74 |
| `canvas2d-css-woff` | 63 |
| `canvaskit` | 5 |
| `canvaskit-sfnt` | 153 |
| `hancom-missing-font` | 1 |
| `official-successor` | 1 |
| `shared-or-not-applicable` | 921 |
| `source-exact` | 1 |

Canvas2D의 CSS family 사용 가능성과 CanvasKit의 SFNT byte 조달을 같은 행에 두지 않았다. CanvasKit
source가 없는 `FONT_LIST` entry도 누락시키지 않고 `unavailable` 정책으로 보존했다. 정부상징
missing-font PDF 관찰은 source exact 또는 ROKG successor의 정답지로 사용하지 않는다.

## 충돌·순환 감사

상충 target 14개 group은 두 부류다.

- 6개는 source에 이미 `order`가 있는 lookup/fallback chain이다.
- 8개는 Studio `SUBST_TABLES`의 동일 source·language·altType 다중 target이다. runtime의
  `Map` 구축이 첫 entry를 보존하므로 물리 배열 순서를 원장 `order` 0, 1로 복원했다.

동일 decision key의 order 중복은 0개다. 탐지한 cycle은 모두 self-loop이며, 다단 순환은 0개다.

| owner | plane | member | rule |
| --- | --- | --- | --- |
| rust-metric | layout-metric | D2Coding | rule.rust-metric.ee88ac9b7256e14b1034 |
| rust-metric | layout-metric | Gowun Batang | rule.rust-metric.0be3736c8eb2a5dd33fb |
| rust-metric | layout-metric | Gowun Dodum | rule.rust-metric.b4f149e0beb1732c1da7 |
| rust-metric | layout-metric | Pretendard | rule.rust-metric.851b868ea4a0e030660b |
| studio-substitution | paint | 휴먼명조 | rule.studio-substitution.31018655d8b0949f8169 |

Rust metric self-loop는 단일 match의 canonical spelling 반환이다. Studio self-loop는 visited-set과
15단계 상한으로 종료된다. 어느 쪽도 byte identity 증거로 사용하지 않는다.

## 미확정 규칙과 후속 질문

| rule | source | target/policy | 후속 질문 |
| --- | --- | --- | --- |
| `rule.rust-metric.cf9164c17319ba49ddd8` | 함초롬돋움 | HCR Dotum | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.50e8b4fe44e2633d21f9` | 한컴돋움 | Haansoft Dotum | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.1378b1b9d9506f86a014` | 돋움 | Dotum | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.1de1fcb9b17d66d599b5` | 함초롬바탕 | HCR Batang | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.7a2702136cc80f952133` | 한컴바탕 | Haansoft Batang | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.699fcfcdb4b25070dc37` | 바탕 | Batang | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.e69131207a16d7535d80` | 맑은 고딕 | Malgun Gothic | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.5ff808f5409744c0a1b0` | 나눔고딕 | NanumGothic | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.1464fe2bf651f25c3060` | 나눔명조 | NanumMyeongjo | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.980ac0a95716c7a62c4a` | 바탕체 | BatangChe | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.34c100f7862575a46540` | 굴림 | Gulim | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.8cdee6ea2e6a2099d9dc` | 궁서 | Gungsuh | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.b2b4b3bf73bbdcae40b5` | 굴림체 | GulimChe | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.9bdad5ff6a5c5fe55bf7` | 돋움체 | DotumChe | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.7e74e1181bd29a93343b` | 궁서체 | GungsuhChe | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.ee88ac9b7256e14b1034` | D2Coding | D2Coding | Self-loop is intentional canonicalization in a single-pass Rust match; it is not byte identity. |
| `rule.rust-metric.a8941ceca1bf044e21bd` | D2 Coding | D2Coding | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.c90981a0f93928779824` | 고운바탕 | Gowun Batang | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.0be3736c8eb2a5dd33fb` | Gowun Batang | Gowun Batang | Self-loop is intentional canonicalization in a single-pass Rust match; it is not byte identity. |
| `rule.rust-metric.a2509235152a1ac5c875` | 고운돋움 | Gowun Dodum | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.b4f149e0beb1732c1da7` | Gowun Dodum | Gowun Dodum | Self-loop is intentional canonicalization in a single-pass Rust match; it is not byte identity. |
| `rule.rust-metric.851b868ea4a0e030660b` | Pretendard | Pretendard | Self-loop is intentional canonicalization in a single-pass Rust match; it is not byte identity. |
| `rule.rust-metric.f21ebe3ed64dcd2ad18e` | 프리텐다드 | Pretendard | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.bedc15cfc3048b3367e7` | HY중고딕 | HYGothic-Medium | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.04b2e35585c329e276eb` | HY견고딕 | HYGothic-Extra | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.1f9388631aac491d3380` | HY헤드라인M | HYHeadLine-Medium | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.f1450144ca329b7be879` | HY견명조 | HYMyeongJo-Extra | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.1444c98cb745d946782a` | HY신명조 | HYSinMyeongJo-Medium | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.b5c0f0d1efa629d562f2` | HY그래픽 | HYGraphic-Medium | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.e648f19ca68dd7c3cebe` | HY궁서 | HYGungSo-Bold | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.e3b10e6770cafa77ff71` | 한양신명조 | HanyangSinMyeongJo | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.f87d4d432ef2820f9c8f` | 한양중고딕 | HanyangJungGothic | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.d51ea95a4f72143e4b51` | 한양견명조 | HanyangKyunMyeongJo | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.c28b0b6b6dfd7f3adf86` | 한양견고딕 | HanyangKyunGothic | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.4f65d46263459fd6e3eb` | 휴먼명조 | HumanMyeongJo | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.a1d58eacfc8e7abdaf9b` | 신명조 | HanyangSinMyeongJo | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.d1edee136a66e4703a19` | HY수평선B | HYsupB | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.7867c1d6f35f35f56fee` | HY수평선M | HYsupM | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.2dc7a889b1fdf42b4e88` | HY울릉도B | HYwulB | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.c38460dc5362fdf16080` | HY울릉도M | HYwulM | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.48163357a589b288e4f3` | HY태백B | HYtbrB | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.e5beb2bc96916f59b69c` | HY동녘B | HYdnkB | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-metric.9bbf1f249a62f66d4ebf` | HY동녘M | HYdnkM | Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance. |
| `rule.rust-measurement.28970f1984ed0e4ce06d` | (predicate) | estimate cluster and character advances using embedded metrics and guarded heuristics | Separate exact metric lookup, heuristic estimation, and synthetic styling in W2 trace output. |

미확정 규칙은 삭제하거나 임의로 `identity-alias`로 바꾸지 않았다. W2/W8에서 다음 순서로
판정한다.

1. source/target SFNT name table과 허용된 byte digest로 multilingual name alias인지 확인한다.
2. name alias가 아니면 실제 advance·coverage 차이를 측정해 `metric-surrogate` 여부를 판정한다.
3. generic width estimator는 exact metric miss, heuristic, faux styling provenance를 W2 trace에서
   분리한 뒤 더 구체적인 relation으로 승격한다.

## 재생성과 검사

```bash
node scripts/font_rule_ledger_evidence.mjs build \
  --candidates mydocs/tech/investigations/issue-4939/font_rule_candidates.json \
  --ledger mydocs/tech/investigations/issue-4939/font_rule_ledger.json \
  --summary mydocs/tech/investigations/issue-4939/font_rule_ledger_summary.md
node scripts/font_rule_ledger_evidence.mjs check \
  --candidates mydocs/tech/investigations/issue-4939/font_rule_candidates.json \
  --ledger mydocs/tech/investigations/issue-4939/font_rule_ledger.json
node --test scripts/tests/font_rule_ledger_evidence.test.mjs
```
