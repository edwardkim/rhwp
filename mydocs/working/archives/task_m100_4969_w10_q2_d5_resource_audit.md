# Task M100 #4969 W10-Q2-D5 — resource reuse·dedup 선행 감사

## 판정

Q2-D5 진입 전 resource 감사 결과는 **수정 수행계획 작성 가능**이다. D4-C가 남긴 성능 자료는 최초 strict
lane의 비용을 판단하기에 충분하므로 같은 단일 fixture 전체 계측이나 private corpus 전수 계측을 반복하지
않는다. 다만 현재 자료는 run 수와 page 수가 늘 때의 비용 곡선을 측정하지 않았으므로, D5 구현 전에 공개
fixture의 bounded `1/2/8 run × 1/2/8 page` matrix를 새로 계측해야 한다.

현재 구현은 “resource 재사용이 전혀 없음”이 아니다. 같은 `ResourceArena` 안에서는 동일 font blob을 한 번만
보존하고, exact source registry와 replay certificate도 `Arc<[u8]>`를 공유하며, CanvasKit은 검증된 blob과
typeface를 document renderer 수명 동안 cache한다. 반면 common shaping lowerer는 같은 source인 각 run에서
font 전체를 반복 hash·대사하고, page별 JSON은 같은 base64 font payload를 다시 싣는다. 따라서 D5 선행 정정은
기존 page-local dedup을 보존하면서 **unique source 단위 digest/face 준비**와 **document 전달 단위 payload
재사용**을 추가하는 것이어야 한다.

기계 판독 요약은
[`w10_q2_d5_resource_audit.json`](../../tech/investigations/issue-4969/w10_q2_d5_resource_audit.json)에 고정한다.

## 재사용한 기존 증거

### D4-C 성능 기준선

동일 release probe에서 D4-A dormant `520f14dcf`와 D4-B active `4992ccbf3`를 비교한 기존 결과를 그대로
재사용한다.

| 항목 | D4-A dormant | D4-B active | 차이 |
| --- | ---: | ---: | ---: |
| warm layer build | 0.843µs | 703.600µs | +702.757µs, 834.64배 |
| cold layer build | 1.608ms | 2.314ms | +0.706ms, +43.91% |
| layer JSON | 6,849B | 619,562B | +612,713B, 90.46배 |
| portable font payload | 0B | 456,688B | +456,688B |

이 수치는 exact portable replay의 최초 비용을 이미 증명한다. 같은 단일 fixture를 같은 방식으로 다시 돌려도
D5의 질문인 “여러 run·page에서 무엇이 한 번만 수행되는가”에는 답하지 못한다.

### 이번 focused 확인

- `cargo test --locked --lib interns_duplicate_resources_once`: 1 pass
- `cargo test --locked --test issue_4969_shaping_glyph_lowering`: 7 pass
- task worktree에서 generated suite를 준비하지 않았다. 사전 `manifest --check`는 prepare되지 않은 기존
  harness drift를 보고했으며 source·Cargo·generated 파일을 만들거나 수정하지 않았다.

첫 검사는 동일 font bytes가 한 `ResourceArena`에서 blob 1개가 됨을 확인한다. #4969 lowering 검사는 exact
certificate의 registry-owned `Arc` 공유, stale generation 거부, 성공 시 font blob/face 각 1개, 실패 시 resource
mutation 0을 확인한다.

## 현재 resource 계보

```text
ExactFontSourceRegistry
  └─ ExactFontSourceHandle -> Arc<[u8]> 한 번 보존
       └─ HorizontalShapingReplaySourceCertificate -> 같은 Arc clone
            └─ 각 GlyphRun lowering
                 ├─ BLAKE3 resource digest 계산 1
                 ├─ register_portable_face에서 BLAKE3 계산 1
                 ├─ ResourceArena interning에서 FNV 전수 scan
                 │    └─ 최초 blob이면 BLAKE3와 Vec copy 1
                 └─ page-local ResourceArena에는 동일 blob/face 1개

PageLayerTree JSON
  └─ 각 page의 resources.fontBlobs에 base64 payload inline
       └─ DocumentCore는 page·option별 완성 JSON을 cache
            └─ CanvasKit은 첫 검증 뒤 blob/typeface/font를 document renderer에서 cache
```

## 이미 만족하는 불변식

1. exact source registry는 동일 handle의 bytes를 `Arc<[u8]>` 한 개로 보존한다.
2. replay certificate는 font bytes를 복사하지 않고 registry `Arc`를 공유한다.
3. 한 page의 `ResourceArena`는 같은 bytes를 blob 1개로 intern한다.
4. 같은 page의 font blob metadata와 face metadata는 ID 기준으로 중복 등록되지 않는다.
5. 모든 lowering 검증은 resource mutation보다 먼저 끝나며 reject가 partial blob을 남기지 않는다.
6. CanvasKit은 digest·length·resource key를 검증한 뒤 같은 blob/typeface/font를 재사용한다.
7. TextRun fallback은 GlyphRun과 함께 남아 cache miss·backend 미지원 시 내용을 잃지 않는다.

이 불변식을 다시 설계하거나 page-local blob을 run마다 분리해서는 안 된다.

## 남은 결손

### R1 — run별 전수 digest와 face parse

`lower_horizontal_shaping_source_shadow()`는 같은 exact source라도 각 run에서 `resource_digest_hex()`를 호출하고,
`register_portable_face()`가 같은 BLAKE3를 다시 계산한다. `ResourceArena::intern_font_blob_bytes()`도 매 호출마다
font 전체의 FNV hash와 candidate bytes 대사를 수행하며 최초 등록에서는 BLAKE3와 `Vec` copy를 추가한다.
`certified_replay_face()`의 `ttf_parser::Face` 준비도 run별이다.

따라서 page-local payload 개수는 1이어도 CPU 비용은 run 수에 따라 font byte 길이와 함께 증가할 수 있다.

### R2 — page 간 inline JSON 중복

`PageLayerTree`는 page마다 새 `ResourceArena`를 만들고 `write_visual_resources()`는 모든 `fontBlobs`를 base64로
직렬화한다. `layer_tree_json_cache`는 같은 page·option의 재직렬화를 피하지만, 첫 직렬화와 cache 무효화 뒤에는
payload를 다시 만들며 다른 page 사이에는 blob을 공유하지 않는다.

Studio의 CanvasKit cache는 같은 digest를 다시 typeface로 만들지 않지만, 현재 `getPageLayerTreeWithProfile()`은
font payload를 inline으로 받은 뒤에야 그 cache에 도달한다. 그림의 `omit_image_bytes`·`getSourceImageBytes()`에
대응하는 font by-key 전달 경계는 아직 없다.

### R3 — 기존 D4 측정의 범위

D4 수치는 단일 run의 최초 strict lane 비용을 정확히 보여주지만 다음을 분리하지 않는다.

- unique source 준비 비용과 run별 geometry lowering 비용
- 같은 page의 2/8 run 비용
- 다른 page의 같은 source payload 전송량
- JSON cache hit와 cache invalidation 뒤 rebuild
- cache miss·stale generation·잘못된 digest에서의 전체 TextRun rollback

## D5 선행 측정 matrix

새 계측은 private corpus가 아니라 저장소의 공개 Source Han subset fixture만 사용한다. 임의 wall-time 상한을 먼저
정하지 않고, 아래 구조적 목표와 절대 수치·증감률을 함께 기록한다.

| 축 | 입력 | 반드시 증명할 결과 |
| --- | --- | --- |
| page-local dedup | 같은 source `1/2/8` run | blob·face·payload bytes는 각각 `1/1/456,688B`; GlyphRun만 N개 |
| source 준비 | 같은 source `1/2/8` run | digest identity와 face resource 준비는 unique source당 1회 |
| document 전달 | 같은 source `1/2/8` page | 최초 fetch 뒤 font bytes 전송 합계가 page 수에 비례하지 않음 |
| JSON | inline과 by-key | by-key page JSON에 base64 font payload 0, resource key·metadata는 유지 |
| cache invalidation | 동일 generation / 새 generation | 동일 generation은 재사용, source·generation 변경은 이전 blob을 재사용하지 않음 |
| fail-closed | missing/wrong/oversized key | GlyphRun replay 거부, TextRun 유지, partial cache·resource 0 |
| 성능 | D4와 같은 release protocol | warm/cold 절대값과 `1→2→8` 증가분을 분리 기록 |

구조적 gate는 “8 run이 1 run보다 몇 µs 이하여야 한다”가 아니다. font 전체 digest·parse·전송이 **run/page 수가
아니라 unique exact source 수**에 귀속되는지를 계측 counter와 payload byte 회계로 증명하는 것이다.

## 수정 수행계획에 반영할 권고 구조

### A. unique source 준비 cache

- exact source handle과 registry generation을 key로 bounded portable resource identity를 준비한다.
- BLAKE3 digest, resource key, face index와 immutable face metadata를 unique source당 한 번 계산한다.
- page lowerer는 준비된 identity를 받아 `ResourceArena`에 한 번 등록하고 각 run은 그 face key만 참조한다.
- source bytes는 기존 certificate의 `Arc<[u8]>`를 공유하며 trace·JSON metadata에 raw bytes나 host path를 넣지
  않는다.

### B. font by-key 전달

- 기존 inline JSON을 호환 기본값으로 유지하고 선택적 font-byte omission을 별도 option으로 연다.
- metadata의 `dataRef.id`와 digest는 유지하고 binary payload만 생략한다.
- WASM은 bounded `getSourceFontBytes(key)` 또는 동등한 exact-source resolver를 제공한다.
- Studio는 document-generation별 verified blob cache를 먼저 조회하고 miss일 때만 bytes를 받아 digest·length를
  다시 검증한다.
- 오래된 generation, 없는 key, digest mismatch와 상한 초과는 GlyphRun을 거부하고 기존 TextRun으로 닫는다.

그림 by-key 경로는 설계 선례지만 font identity와 생명주기의 정답으로 복사하지 않는다. 그림은 BinData ID·epoch가
신원이고 font replay는 exact source digest·face index·registry generation이 신원이므로 별도 capability와 cache
key를 사용해야 한다.

## D5와의 순서

1. resource reuse probe와 계측 counter를 먼저 추가한다.
2. unique source 준비 cache를 구현하고 page-local `1/2/8 run` 결과를 판정한다.
3. font by-key 전달과 Studio cache miss/fallback을 구현하고 `1/2/8 page` 결과를 판정한다.
4. 위 결과가 qualified일 때만 NO_LS boundary-changing transaction을 연다.
5. D5 기능 검증은 line selection·bbox·next origin·GlyphRun이 같은 `Arc`를 소비하고 하나라도 실패하면 네 값을
   함께 rollback하는 기존 종료 gate를 그대로 적용한다.

## 제외 범위

- D5 boundary activation 제품 구현
- multi-line·multi-run 일반화 자체
- RTL, vertical, variation, nonzero GPOS y positioning
- native Skia blob typeface replay
- private corpus·Hyper-V·한컴 Oracle 재실행
- 기존 D4 전체 회귀의 무근거 반복

## 결론

2단계의 결론은 “resource system을 새로 만든다”가 아니다. page-local blob dedup과 exact-source `Arc`, CanvasKit
document cache는 이미 존재한다. D5 전에 필요한 최소 정정은 common shaping의 **unique source 준비 cache**와
WASM/Studio의 **font by-key 전달**이며, 두 효과를 공개 `1/2/8` matrix로 증명하는 것이다. 이 경계를 반영한 수정
수행계획을 별도 승인받기 전에는 source 구현과 D5 activation을 시작하지 않는다.
