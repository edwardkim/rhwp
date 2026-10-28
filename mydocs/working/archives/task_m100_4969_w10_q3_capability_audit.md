# Task M100 #4969 W10-Q3 — variable font instance capability 감사

## 판정

W10-Q3는 **수정 수행계획 작성 가능, 제품 activation 불가** 상태다. 공식 exact variable TTF와 Q1 standalone
shaping oracle은 준비돼 있다. 반면 실제 Q2 transaction·cache·lowering·backend replay에는 variation instance가
연결되지 않았고, 현재 제품에는 HWP/HWPX 문서 의도에서 axis 값을 얻는 권위 경로도 없다.

따라서 Q3는 “variation reject를 제거”하는 작업으로 시작하지 않는다. 먼저 effective instance canonicalization,
mutable face 격리, explicit opt-in request owner와 backend별 replay capability를 red 계약으로 고정해야 한다.

기계 판독 감사는
[`w10_q3_capability_audit.json`](../../tech/investigations/issue-4969/w10_q3_capability_audit.json)에 고정한다.

## exact source 재확인

| 항목 | 값 |
| --- | --- |
| source | `ttfs/redistributable/happiness-sans/HappinessSansVF.ttf` |
| bytes | 1,503,064 |
| SHA-256 | `3bbd254dcc5780f7524f9d07af4aa981ba5e3e84cf32d7d4e04301b3943e8694` |
| face index | 0 |
| glyphs | 3,889 |
| variable tables | `fvar`, `gvar` |
| shaping tables | `GDEF`, `GPOS`, `GSUB` |
| axes | `wght` 400/400/900, `opsz` 400/400/900 |
| named instances | Regular 400/400, Bold 900/400, Title 900/900 |

실제 파일 hash·byte length와 `fonttools ttx`의 `fvar`·table 목록을 Q0/Q1 기록과 다시 대사해 mismatch 0을
확인했다. 공식 archive member를 byte 그대로 추적하며 WOFF2를 변환하거나 TTF를 수정하지 않는다. Q3에서도
static instance font나 수정된 font binary를 생성·추적하지 않는다.

## 이미 준비된 capability

Q1 `renderer/shaping.rs`에는 다음 기반이 있다.

- axis 최대 16개, tag·중복·finite·지원 범위 검증
- axis tag 순서 canonicalization과 float bit identity
- exact source SHA-256·face index·axis vector를 포함한 settings identity
- `rustybuzz::Face::set_variations()` 적용
- default 400/400과 Title 900/900에서 glyph ID는 유지되고 advance가 달라지는 공개 oracle
- malformed·unknown·out-of-range·axis explosion의 structured reject

paint schema도 `FontInstanceKey.variations`와 JSON `shapeKey.fontInstance.variations` 자리를 이미 갖는다. 따라서
Q3의 주된 문제는 schema 필드 추가가 아니라 실제 transaction·replay owner 연결이다.

## 확인된 계보 단절

### 1. effective default identity가 아직 canonical하지 않음

Q1은 axis tag를 정렬하지만 explicit `wght=400, opsz=400`을 빈 vector와 동일한 effective default instance로
접지 않는다. 두 요청은 font 결과가 같아도 variation vector 길이와 settings hash가 다르다. 그대로 cache key와
resource identity에 연결하면 같은 instance가 중복되고 “동일 instance는 같은 identity” 불변식이 깨진다.

Q3는 각 `fvar` default와 같은 coordinate를 canonical vector에서 제거해야 한다. source digest·face index가 axis
model을 고정하므로 빈 vector는 해당 exact face의 default instance를 뜻한다. axis 순서, `-0.0`, partial vector와
explicit default가 같은 effective instance로 정규화돼야 한다.

### 2. mutable face instance 누출 위험

Q2 `HorizontalShapingTransaction`은 source handle당 mutable `rustybuzz::Face` 하나를 cache한다. Rustybuzz
`set_variations()`는 전달된 축만 설정하고 생략된 축을 default로 되돌리지 않는다. 그러므로 한 face를 900/900으로
사용한 뒤 빈 vector 또는 partial vector로 재사용하면 이전 coordinate가 남을 수 있다.

variation을 현재 cache에 단순 추가해서는 안 된다. parsed face의 owner를 `(exact source handle, canonical effective
instance)`로 분리하거나 매 요청마다 모든 축을 default 포함 완전 설정해야 한다. Q3 권고는 instance별 immutable
face cache다. source bytes는 공유하되 서로 다른 instance가 같은 mutable face를 공유하지 않는다.

### 3. Q2 transaction과 lowerer가 axis를 운반하지 않음

- `HorizontalShapingRequest`에 variation field가 없다.
- `HorizontalShapingCacheKey`에 instance vector가 없다.
- Q2는 standalone shaper에 항상 `variations: &[]`를 전달한다.
- common GlyphRun lowerer는 measurement identity의 variation이 비어 있지 않으면 거부한다.
- current prepared source는 source당 한 번 준비되지만 instance별 measurement·outline identity는 없다.

따라서 현재 Q1 variation oracle은 실제 line selection·bbox·next origin·GlyphRun owner에 도달하지 않는다.

### 4. backend capability가 서로 다름

| backend | 현재 상태 | Q3에서 필요한 증거 |
| --- | --- | --- |
| CanvasKit | variation vector가 있으면 `variationUnsupported`; 현재 JS Typeface API에 clone/coordinate binding 없음 | exact runtime GlyphOutline 또는 실제 instance typeface API proof |
| Native Skia | selector가 `FontVariationUnsupported`; skia-safe에는 `FontArguments`와 `clone_with_arguments()` 존재 | exact blob typeface construction과 coordinate round-trip proof |
| SVG·Canvas2D | glyph-ID typeface instance를 만들지 않음 | exact variable outline을 공통 owner가 게시해 backend reshaping 없이 replay |
| text-v2 | variation을 strict variant에서 거부 | instance-qualified GlyphRun/Outline 선택 규칙 |

CanvasKit font cache key도 현재 size·synthetic style만 포함하고 axis vector를 포함하지 않는다. selector를 먼저
열면 서로 다른 instance가 같은 `Font`를 재사용한다. capability가 증명되지 않은 backend는 기존 TextRun fallback을
유지해야 한다.

## 권위 입력 부재

현행 HWP/HWPX parser·model·serializer에는 `fvar`, `wght`, `opsz` 또는 variation axis 의도를 표현하는 필드가
없다. 글자 굵기·장평·글꼴 이름을 axis 값으로 추측하면 문서에 없던 의도를 생성하고 기존 조판을 바꾼다.

최초 Q3 product lane은 명시적 opt-in command/API가 `(charShapeId, languageIndex)` exact source slot에 canonical
instance를 결합하는 경우만 허용해야 한다. API 이름과 CQRS owner는 구현 전 감사에서 확정한다. axis 요청이 없으면
Q2 default instance·출력을 byte/canonical identity 수준으로 보존한다.

## POC matrix와 측정 범위

private corpus·Hyper-V·한컴 Oracle은 사용하지 않는다. HWP/HWPX가 axis intent를 제공하지 않으므로 한컴 출력은
Q3의 source-of-truth가 될 수 없다. 공식 Happiness Sans TTF와 공개 문구를 사용해 다음 effective instance를
분리한다.

| ID | wght | opsz | 목적 |
| --- | ---: | ---: | --- |
| default | 400 | 400 | 빈 vector·explicit default 동일성, Q2 무회귀 |
| weight-interior | 650 | 400 | `wght` 단독 interior |
| bold | 900 | 400 | named Bold boundary |
| optical-interior | 400 | 650 | `opsz` 단독 interior |
| title | 900 | 900 | named Title, 두 축 max |

fixture는 pure Hangul과 pure Latin non-ligature를 분리한다. Q1의 mixed `가변 Typography`는 historical oracle로
보존하되 첫 product activation의 homogeneous script 근거로 사용하지 않는다. `1/2/8 instance × 1/2/8 run`에서
face parse·shape result·outline/typeface·font cache 수와 payload bytes를 측정한다.

## 보호 불변식

1. axis 요청이 없으면 Q2 `bounded-subset`의 glyph·cluster·geometry·fallback과 output이 변하지 않는다.
2. HWP/HWPX version·font name·bold·장평에서 variation axis를 추측하지 않는다.
3. exact source digest·face index·canonical effective vector가 instance identity와 cache key에서 빠지지 않는다.
4. 빈 vector, explicit defaults, 축 순서만 다른 요청은 같은 effective instance다.
5. 서로 다른 instance가 mutable rustybuzz/ttf-parser/Skia face 상태를 공유하지 않는다.
6. shaping measurement·line selection·bbox·next origin·replay payload가 같은 instance를 소비한다.
7. backend는 published glyph/outline을 재생하며 독자 shaping을 하지 않는다.
8. backend instance proof가 없으면 variation GlyphRun을 선택하지 않고 TextRun 또는 qualified outline으로 닫는다.
9. axis 16개, glyph·cluster 4,096, cache·font resource 상한을 유지한다.
10. modified/static instance font를 생성·추적하지 않고 공식 TTF bytes를 그대로 사용한다.
11. raw text·font bytes·host path·private corpus identity를 trace에 기록하지 않는다.
12. Q3는 W8 face correction이나 font fallback registry 규칙을 함께 바꾸지 않는다.

## 결론과 다음 단계

Q3는 exact source와 standalone oracle이 준비됐으므로 blocked가 아니다. 그러나 현행 transaction에 variation vector만
추가하는 구현은 default identity 중복, mutable face 누출, backend cache collision을 만들 수 있어 위험하다.

다음은
[`task_m100_4969_w10_q3.md`](../../plans/archives/task_m100_4969_w10_q3.md)의 Q3-A canonical instance red 계약이다.
감사·수정 수행계획 승인 전에는 제품 source를 변경하지 않는다. commit·push·PR·GitHub comment도 각각 기존 승인
경계를 유지한다.
