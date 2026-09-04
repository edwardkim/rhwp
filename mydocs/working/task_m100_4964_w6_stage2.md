# Task M100 #4964 — Stage W6-2 lineage manifest·계보 inventory

- **수행계획**: [`../plans/task_m100_4964.md`](../plans/archives/task_m100_4964.md)
- **선행 기준선**: [`task_m100_4964_w6_stage1.md`](task_m100_4964_w6_stage1.md)
- **기준 source**: `upstream/devel@d1ad0eb8784dbc55f0796e2ba8775f7363247b91`
- **단계 목적**: 600개 전부에 안정 ID·provenance 상태·W1/W5 evidence를 부여한다.
- **판정**: 통과

## 1. 결론

600개 항목은 모두 manifest row에 대응한다. 그러나 현재 증거로 **전체 metric이 source-exact라고
재현 가능한 항목은 0개**다. 이것은 데이터 600개가 모두 틀렸다는 뜻이 아니라, 과거 생성 당시의
font digest·face index·name record·compression error를 항목별로 보존하지 않았다는 뜻이다.

| 분류 | 항목 수 | 판정 |
| --- | ---: | --- |
| historical generated region, origin unknown | 595 | 생성 영역은 확인, source-exact 승격 금지 |
| #2430 measured overlay, provenance verified | 5 | 추적 TSV·generator로 ASCII 95자 재현 |
| fully source-exact | 0 | 전체 range/Hangul 재현 source manifest 없음 |
| tracked font partial verification | 1 | Noto Sans KR Regular printable ASCII만 검증 |
| W1 metric-entry 연결 | 600 | 각 index의 기존 rule ID와 1:1 |
| W5 Oracle Profile 연결 | 2 | 한양신명조·휴먼명조 face identity |

이 분류는 `generated = exact`를 의도적으로 거부한다. 후속 단계가 source bytes를 확보하고 전체
metric/compression을 재현하기 전까지 unknown을 유지한다.

## 2. 안정 ID와 순서

각 `entryId`는 `(name, bold, italic)`의 SHA-256 기반 20자리 semantic ID다. current index를 ID에
넣지 않았으며 600개 모두 충돌 없이 유일하다. 별도 `currentIndex: 0..599`가 물리 순서와 first-match
좌표를 보존한다.

manifest 생성기는 다음을 fail-closed한다.

- current index 누락·중복·순서 변경
- entry identity와 안정 ID 불일치
- W1 `metric-entry:<index>` rule 누락·오연결
- Rust source의 Latin/Hangul symbol 또는 per-entry semantic hash 불일치
- evidence path 누락·digest 변화·절대 경로
- unknown reason 누락 또는 legacy generated 항목의 무근거 `verified` 승격
- W5 profile의 document face 불일치

## 3. source·measurement·compression 상태

### 3.1 font source

599개 항목은 item-level source font가 저장소에 없어 digest, face index, naming record,
license/provenance를 각각 이유가 있는 `unknown`으로 기록했다.

`Noto Sans KR` regular(index 12)는 다음 항목을 실제 추적 TTF에서 읽었다.

- SHA-256 `6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74`
- single-face TTF, face index 0
- Unicode-platform SFNT naming record 8개
- SIL OFL 1.1과 Google Fonts → wght 400 instance → rhwp subset provenance

다만 #4442 test는 printable ASCII advance만 비교한다. name ID 3에도 과거 `NotoSansKR-Thin` 문자열이
남아 있어 이 파일을 현재 DB 전체의 exact source로 간주하지 않고
`verificationScope: printable-ascii-only`로 제한했다.

### 3.2 measured overlay

index 595..599의 다섯 항목은 각 ladder TSV, `gen_metrics.py`, preflight와 EVIDENCE digest를 연결했다.
각각 93자 직접 측정, 한컴 autocorrect 때문에 제외된 따옴표 2자 median-ratio 보간이라는 합성 방법도
manifest에 기록했다.

W5에는 이 가운데 한양신명조와 휴먼명조의 역사적 exact-installed profile만 존재한다. 두 profile의
`identity-exact`는 한컴 readback face에만 적용하며 `scope: face-identity-not-metric-source-exactness`로
metric exact 승격을 막았다. 한양중고딕·한양견명조·한양견고딕에는 존재하지 않는 W5 profile을 만들지
않았다.

### 3.3 Hangul compression

| 상태 | 항목 수 | 의미 |
| --- | ---: | --- |
| `unknown` | 178 | Hangul grid는 있으나 역사 max/average error가 보존되지 않음 |
| `not-applicable` | 422 | HangulMetric 자체가 없음 |

generator가 과거 계산한 compression error를 source schema에 내보내지 않았으므로 178개에 값을
역산하거나 0으로 채우지 않았다. 확인 가능한 신규 source에 대한 error 기록은 Stage W6-4의 생성기
보강 범위다.

## 4. Git 계보 anchor의 한계

manifest는 각 entry 선언 줄을 기준 commit에서 `git blame`해 `origin.declarationCommit`으로 기록한다.
결과는 0..594가 `ea564999e...`, 595..599가 #2430 commit `1727cfc20...`이다.

이 값은 선언 줄의 anchor일 뿐 참조 Latin/Hangul 배열이 최초 생성되거나 마지막 수정된 commit 전체를
뜻하지 않는다. 특히 legacy 595개가 한 commit으로 모이는 현상은 대규모 source rewrite의 결과이므로
이를 원본 provenance로 사용하지 않는다. 이 한계를 필드명과 문서에 명시했다.

## 5. deterministic 산출물

| 산출물 | SHA-256 |
| --- | --- |
| `font_metric_lineage_manifest.schema.json` | `7374ec8de5d52d59c0eb2acecd7ce2ff597cc98b23df37fb22e37e20e372816b` |
| `font_metric_lineage_manifest.json` | `4748fceb90758f8f2e060807f8f24cbfda77ebcbbb3aae68e4adc8e35e2bc904` |
| manifest 600행 canonical projection | `beeba4de52b4aa0c320f239ac29fb2b4e923d859dd8c4b6a1d86dba6f8983b0f` |

마지막 projection hash는 manifest의 `entriesSha256` 정본을 따른다. 파일은 wall-clock, 절대 경로,
사용자명 없이 canonical key order와 마지막 newline을 사용한다.

## 6. 검증

| 명령 | 결과 |
| --- | --- |
| `node scripts/font_metric_lineage.mjs --check` | W6-1 hash 통과 |
| `node scripts/font_metric_lineage.mjs --check-manifest` | 600행·evidence·source 정합 통과 |
| `node --test scripts/tests/font_metric_lineage.test.mjs` | 10/10 통과 |
| Python `jsonschema` Draft 2020-12 validation | 통과 |
| `node --check scripts/font_metric_lineage.mjs` | 통과 |

negative test는 unknown reason 제거, legacy exact 승격, W1 rule 오연결, evidence digest 훼손,
metric identity 변경과 per-entry width hash 변경을 모두 거부했다.

## 7. Stage 판정과 다음 게이트

600개 전부가 manifest에 대응하고 미확정 분모가 숨겨지지 않았으므로 Stage W6-3 물리 분리의 선행
조건을 충족했다. W6-3에서는 이 manifest와 W6-1 hash를 보호막으로 사용해 core lookup,
historical generated region 595개, measured overlay 5개를 물리적으로 나눈다.

아직 `src/renderer/font_metrics_data.rs`의 data나 runtime lookup은 변경하지 않았다. Stage W6-3은
메인테이너 승인 후 시작한다.
