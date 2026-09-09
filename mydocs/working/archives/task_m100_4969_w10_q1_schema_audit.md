# Task M100 #4969 W10-Q1 — shaping schema 충분성 대사

## 결론

Q1의 성공 결과를 운반하기 위해 `ShapeKey`, `FontInstanceKey`, `LayerGlyphRunPaint`에 새 필드를 추가할 필요는
없다. exact source·face, direction·writing mode·script·language, ordered feature·variation, glyph ID·position·
advance·cluster와 backend replay 판정에 필요한 자리가 이미 있다. Q1에서 제품 schema를 변경하지 않고 이
경로를 `sufficient-with-producer-canonicalization`으로 판정한다.

반면 **shaping을 시도했지만 GlyphRun을 만들지 못한 실패 경로**에는 실제 계보 결손이 있다. standalone Q1
`ShapingOutputDecision`은 typed disposition과 reason을 보존하지만, 현재 PageLayerTree 공개 진단은 GlyphRun이
생성돼야 `GlyphRunDiagnostics`를 볼 수 있다. `TextShapeDiagnostic`은 자유 문자열 reason만 가지며 반환된
`TextShapeReport`도 PageLayerTree JSON에 남지 않는다. `TextV2Diagnostics` 역시 실제 GlyphRun/GlyphOutline
variant가 없는 TextRun-only 실패를 slot으로 만들지 않는다.

이 결손을 이유로 Q1에서 schema minor를 즉시 올리지는 않는다. 아직 shaping이 layout에 연결되지 않았으므로
공개 출력에 빈 필드를 먼저 추가하면 실제 계보가 아니라 추측 설계가 된다. 대신 Q2 진입 전 필수 조건으로
bounded rejected-attempt sidecar를 설계·승인하도록 동결한다.

기계 판독 정본은
[`w10_q1_schema_audit.json`](../../tech/investigations/issue-4969/w10_q1_schema_audit.json)이다.

## 필드별 대사

| Q1 계약 | 기존 owner | 판정 |
| --- | --- | --- |
| exact font source | `faceKey → FontFaceResource → FontBlobResource` | 충분 |
| face index | `FontFaceResource.face_index` | 충분 |
| direction·writing mode | `ShapeKey` 및 GlyphRun 명시 필드 | 충분 |
| script·language | `ShapeKey.script/language` | 충분 |
| ordered features | `ShapeKey.features: Vec` | 충분, producer 순서 보존 필요 |
| variation instance | `FontInstanceKey.variations: Vec` | 충분, producer tag 정렬 필요 |
| glyph ID·offset·advance | `glyph_ids`, `positions`, `advances`, `placement` | 충분 |
| many-to-many cluster | `GlyphCluster` source range·glyph range·flags | 충분 |
| shaper·fallback lineage | `shaping_engine`, `fallback_policy` | 충분 |
| applied replay quality | `GlyphRunDiagnostics` | 충분 |
| rejected attempt disposition | emitted GlyphRun 밖에는 typed owner 없음 | 결손 |

## 해석상 주의점

### digest algorithm

Q1 standalone identity는 SHA-256을 사용하지만 현행 portable resource key는 BLAKE3다. `FontDigest`가 algorithm과
value를 함께 가지므로 schema 결손은 아니다. Q2 adapter는 두 digest 문자열을 직접 비교하지 말고 선언된
algorithm을 확인하거나 exact bytes에서 Q1 SHA-256을 계산해야 한다.

### size·synthetic style

Q1 rustybuzz oracle은 font unit 결과라 font size를 settings hash에 넣지 않았다. 제품 position과 cache에는
`FontInstanceKey.size_px`가 반드시 남아야 한다. synthetic bold/italic은 Q1 요청에 없으므로 임의 shaping으로
흡수하지 않고 기존 `FontInstanceKey`와 capability gate에서 별도로 처리한다.

### vertical direction

`TextDirection`에 TTB 변형이 없어도 결손은 아니다. 이 enum은 bidi 방향을, `WritingMode`는 vertical inline
progression을 표현한다. rustybuzz의 `TopToBottom`은 두 필드를 결합해 adapter에서 도출해야 한다.

### cluster 변환

rustybuzz cluster는 UTF-8 시작 offset이고 GlyphRun schema는 source range와 glyph range를 가진다. LTR fixture만
보고 다음 cluster를 단순히 오른쪽 경계로 쓰면 RTL에서 깨질 수 있다. Q2 converter는 direction-aware boundary를
만들고 동일 cluster를 공유하는 glyph를 한 범위로 묶어야 한다.

## Q2 필수 인계 계약

실패도 성공과 같은 vocabulary로 관측하려면 shaping attempt가 `Option<GlyphRun>`만 반환해서는 안 된다. 공통
attempt 결과는 최소한 다음 정보를 glyph emission 전에 소유해야 한다.

- bounded run/source span 식별자
- `disposition`과 typed `reason`
- glyph count

canonicalization까지 도달한 attempt는 `settingsSha256`을 포함한다. source unavailable·font byte limit처럼
identity를 만들기 전에 닫힌 attempt에서는 이 값과 `fontSourceSha256`을 `None`으로 유지해야 하며, 빈 문자열이나
가짜 hash를 발급하지 않는다.

원문, font bytes의 진단 중복, 로컬 경로와 private corpus identity는 금지한다. portable replay에 필요한 font
blob은 기존 resource table의 별도 계약이며 diagnostics에 다시 복제하지 않는다. 공개 PageLayerTree JSON에 이
sidecar를 실제 연결하는 시점에만 schema minor 상승과 호환성 검증을 수행한다.

## Q1 종료 판정

- standalone request·capability·disposition: `qualified`
- canonical settings identity: `qualified`
- bounded glyph oracle: `qualified`
- applied GlyphRun transport schema: `qualified`
- rejected-attempt public trace: Q2 선행 조건으로 명시적 인계
- 제품 layout·paint·schema mutation: 0

따라서 W10-Q1은 위 결손을 숨기지 않고 종료할 수 있다. 다음 절편은 Q2 구현이 아니라, rejected-attempt sidecar와
horizontal GSUB/GPOS 공통 shaping owner의 수정 수행계획을 먼저 제시하는 단계다.
