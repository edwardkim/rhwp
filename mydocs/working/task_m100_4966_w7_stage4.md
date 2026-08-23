---
kind: report
status: completed
canonical: mydocs/plans/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 — Stage W7-4 Rust layout-name·layout-metric 전환

## 1. 판정

Stage W7-4는 통과했다. Rust runtime의 유한 font name mapping 171행과 metric alias 67행은 canonical
registry에서 생성한 projection을 소비한다. language·alt-type 분기와 legacy-latin → HFT → TTF
우선순위, metric entry 600개의 물리 순서와 exact → bold-only → name-first lookup 사다리는 기존
hand-written owner에 남았다.

전환 전 match 표는 삭제하지 않고 `#[cfg(test)]` 기준 오라클로 보존했다. runtime build에는 중복 표가
들어가지 않으며, 전환 전후 결과를 향후 registry 변경 때도 전건 비교할 수 있다. 신규 JSON parse,
schema 해석, heap allocation과 W1 ledger 검색은 runtime 경로에 추가되지 않았다.

## 2. generated lookup 경계

Rust projection 행에는 W1 `ruleId`와 함께 정확히 하나의 `sourceBoundaryId`를 보존한다. 이 필드가 필요한
이유는 `HCI Poppy` 같은 동일 source face가 legacy-latin과 HFT에 동시에 존재하기 때문이다. face만으로
조회하면 기존 우선순위를 재현할 수 없다.

generator는 다음 정적 함수를 함께 만든다.

- `find_font_rule_layout_name(source_boundary_id, source_face, language_slot)`
- `find_font_rule_layout_metric(source_face)`

두 함수는 generated 배열을 선형 순회하지 않고 Rust `match`를 사용한다. 따라서 runtime lookup에
초기화용 `HashMap`, `OnceLock` 또는 per-call allocation이 생기지 않는다. `#[rustfmt::skip]`은 생성
함수에만 적용해 `cargo fmt --all` 뒤에도 generator의 canonical bytes와 `--check`가 충돌하지 않게 한다.
TypeScript 세 projection의 semantic row와 source bytes는 이 단계에서 바뀌지 않았다.

## 3. layout-name 동등성

133개 고유 source face와 등록되지 않은 sentinel에 대해 alt type 4종과 language slot 7종을 전수
실행했다. 총 3,752개 입력 조합에서 generated resolver와 전환 전 hand-written resolver의 다음 결과가
모두 같았다.

- match 유무와 최종 face
- legacy-latin → HFT → TTF boundary 선택
- HFT common과 English-only language condition
- alt type 2에서만 허용되는 HFT 분기
- generated rule의 `sourceBoundaryId`와 `ruleId`

문서 `substFont`, embedded 판정, CSS family chain과 generic paint fallback은 이 표의 소비 뒤에 있는 기존
알고리즘이므로 수정하지 않았다.

## 4. layout-metric 동등성

67개 alias와 미등록 sentinel을 projection과 전환 전 표로 비교했다. 각 alias는 bold·italic 4조합을
전수 실행해 다음 항목을 비교했다.

- alias target과 generated `ruleId`
- 선택된 `FontMetric` 객체와 기존 entry index
- `exact`, `boldOnly`, `nameFirst` match kind
- faux-bold용 `boldFallback`

W6 lineage 검사는 600개 metric entry, historical generated 595개 + measured overlay 5개의 composition과
모든 저장 width를 다시 계산했다. 다음 pre-migration hash는 그대로다.

| 보호 대상 | SHA-256 |
| --- | --- |
| composition | `d4cdac86b3c6ee55d8b1aa921d662f1fc1241c2809cb9c8ffe991d56a045e69a` |
| lookup projection | `bb3008f9dc379bd580119a6a658388732e94358db2039dbb02d78c28ec992fdf` |
| metric data | `025812eac4bad179c5b87e23b15fdf08a4e4fb3f19a6e453738e03110a140bcf` |
| width projection | `2cd1389a14401f6488041af3c54ff0ba5e982d944acd0b5bb56147056e3a7d1b` |

## 5. W1 기준선과 W2 trace

W7-1 pre-migration snapshot은 재생성하지 않았다. runtime source가 projection 소비로 이동하면 Rust 파일
digest와 source commit은 당연히 달라지므로, 기준선 비교는 역사 provenance의 byte equality가 아니라
보호 대상 semantic view를 비교하도록 바꿨다. source path 목록은 유지하고 다음 값은 계속 fail-closed로
검사한다.

- W1 30개 boundary·1,352개 candidate의 폐합과 `currentMatchesW1`
- Rust name 171행·metric alias 67행의 tuple, 조건, 순서와 projection hash
- active unknown 44개 중 metric legacy-preservation 43개와 hand-written predicate 1개
- W6 600개 metric anchor와 Studio runtime 기준선

W2 trace는 generated `ruleId`를 `FontNameDecision`과 `MetricLookupDecision`으로 운반한다. trace가 기존
candidate identity로 계산한 ID와 generated ID가 다르면 오류로 종료하며 조용히 다른 ID를 기록하지
않는다. 공개 HWP/HWPX native integration 12건과 fresh WASM trace E2E 3건이 기존 ledger join,
requested/resolved face, metric entry, layout hash와 resource-limit 계약을 통과했다.

## 6. projection hash

Rust semantic row에는 새로 `sourceBoundaryId`가 포함되므로 W7-3의 projection hash와 content hash는
의도적으로 갱신됐다. registry 입력 자체와 TypeScript output bytes는 바뀌지 않았다.

| 항목 | SHA-256 |
| --- | --- |
| projection bundle | `4f6a4915575c4476c19dc0b81582cf8ba1fff69a4513b732148702742c116320` |
| content bundle | `9a00372b45bb53e4759c1b4103b156f6040d3604ebb3c60bb64455e977e91fe9` |
| Rust layout-name projection | `595cdcc1c8d81441c9e4585acb393e734f52e6da3e822babf0f722df2c791cee` |
| Rust layout-metric projection | `c4659fc40246c5d4ad903578a61807c646681638cb4c8f9b7c802fb3f0c37cc2` |

## 7. 검증

- W1·W2·W6·W7 Node contract: 61/61
- projection generator focused contract: 11/11
- Rust source-side test tier: 4,225개, 총량 불변
- Rust style/metric 전건 오라클 비교: 통과
- 공개 native W2/W4 integration: 12/12
- Docker 표준 WASM build: 통과, 5분 52초
- fresh WASM font decision trace E2E: 3/3
- registry, projection, pre-migration semantic baseline과 W6 lineage check: 통과

## 8. Stage W7-5 인계

다음 단계는 TypeScript consumer만 전환한다. Canvas2D paint, webfont supply와 CanvasKit SFNT plan의
generated projection을 각각 연결하되 document substitution, local probe, offline filter, glyph coverage와
SFNT capability 판정은 기존 hand-written 알고리즘에 남긴다. Rust projection과 이번 전건 오라클은
W7-5에서 수정하지 않는다.
