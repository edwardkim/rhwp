# Task M100 #4968 — Stage W9-Q3-5R4 진입 감사와 수정 수행계획

- 작성일: 2026-08-26 KST
- 작업 브랜치: `task_m100_4968`
- 선행 커밋: `fbf507d2c` W9-Q3-5R3 portable exact kerning fixture
- 상태: R4 구현 전 진입 감사 완료, 수정 수행계획 승인 대기
- 제품 source 변경: 0

## 1. 결론

R4의 공통 run measurement를 현재 제품 호출 경로에 바로 연결하면 안 된다. R1/R2는 payload-free handle과
provider/session 계약을 만들었고 R3는 그 계약을 native/WASM에서 검증했지만, 실제 `LayoutEngine`에는
font selection이 확정한 source를 공급하는 owner가 아직 없다.

현 상태에서 가능한 지름길은 둘뿐이며 모두 배제해야 한다.

1. `TextStyle.font_family`로 tracked/system font를 다시 찾는다.
   - 이름이 같아도 embedded face, native selected face, browser webfont가 다를 수 있다.
   - 계획의 exact-source 불변식과 중단 조건을 위반한다.
2. 공개 1,236-byte fixture만 product registry에 정적으로 연결한다.
   - test는 통과하지만 실제 문서 selection과 무관한 이름 기반 특례가 된다.
   - 중단된 2.52 MiB Noto prototype과 크기만 다르고 책임 계층 오류는 같다.

따라서 R4 앞에 “font selection 결과를 layout session으로 전달하는 source binding” 절편을 추가해야 한다.
이는 R1/R2를 폐기하는 변경이 아니라 실제 제품 owner를 채우는 단계다.

## 2. 코드 근거

### 2.1 layout 측정에는 exact identity가 없다

- `resolved_to_text_style`은 family, size, ratio, spacing, kerning flag만 만든다.
- `estimate_text_width`와 `compute_char_positions`는 `text + TextStyle`만 입력받는다.
- `LayoutEngine`에는 exact source registry, slot binding, `KerningSourceSession`이 없다.
- composer의 run에는 `char_style_id`와 `lang_index`가 있어 selection slot은 알 수 있지만 그 slot에서 확정한
  source handle은 아직 없다.

### 2.2 embedded source는 layout 이후에 해소된다

`DocumentCore::build_page_layer_tree_with_profile`은 먼저 cached page render tree를 얻은 뒤, 그 tree의
`TextRunNode`를 순회해 사용 font slot을 수집한다. 그 다음에야 embedded BinData를 bounded load하고 TTC face
index를 해소해 `EmbeddedFontFace`를 layer builder에 전달한다.

즉 현재 embedded exact bytes는 paint/lowering에는 존재하지만 앞선 pagination·line measurement에는 존재하지
않는다. 이 순서를 유지한 채 paint에서만 kerning을 적용하면 layout width와 backend positions가 갈라진다.

### 2.3 native/WASM system source도 layout owner가 아니다

- native font path/fontdb는 SVG 또는 backend font 공급 단계에서 사용된다.
- browser가 실제 선택한 webfont bytes를 Rust layout에 등록하는 slot 기반 API는 없다.
- family fallback 결과를 Rust가 이름으로 다시 추측하면 browser/native 실제 face와 provenance가 달라질 수 있다.

## 3. 수정 R4 절편

### R4A — exact font slot registry와 layout-session binding

`(char_shape_id, language_index)`를 `ExactFontSourceHandle`에 연결하는 bounded registry를 둔다.

- registry value는 host/document가 소유한 immutable source bytes와 face index다.
- handle은 R1 계약대로 SHA-256, byte length, face index만 노출한다.
- registry 자체는 `LayoutEngine`이 소유하고 한 번의 layout/reflow 진입에서 R2
  `KerningSourceSession`이 이를 빌린다.
- 최대 face 수와 누적 source bytes 상한을 둔다. 초과·중복 충돌·digest mismatch는 구조화해 fail-closed한다.
- embedded font는 기존 BinData id와 face-index 해소 결과를 layout 이전에 같은 slot으로 등록한다.
- native/WASM 외부 font는 family 이름이 아니라 확정 slot과 bytes를 함께 넘기는 공통 core API로 등록한다.
- registry 변경은 layout cache와 pagination을 무효화하며 다음 layout session에서만 반영한다.

R4A에서는 positions를 바꾸지 않는다. slot→handle→provider→session 왕복과 K0 무변화만 검증한다.

### R4B — 공통 `KerningRunMeasurement`

기존 base char positions와 exact pair candidate를 하나의 owned 결과로 합친다.

- base positions
- pair-adjusted positions
- per-character advance delta
- total width
- capability/disposition/fallback trace
- source handle과 bounded segment 회계

적용 순서는 다음으로 고정한다.

1. 기존 single-glyph base advance
2. script font scale
3. 장평 ratio
4. 기존 glyph-relative 자간과 extra spacing
5. pair design units × effective font size × ratio ÷ unitsPerEm

pair adjustment에는 자간을 다시 곱하지 않는다. K0 또는 exact source 부재는 기존 positions를 그대로 반환한다.

### R4C — token·긴 단어·line boundary 단일 소비

- token total과 `compute_char_positions`가 같은 `KerningRunMeasurement`를 소비한다.
- GSUB 때문에 whole-run nominal gate가 닫히는 문자열은 공백 경계의 최대 256 segment로 제한한다.
- 긴 단어 char fallback은 개별 글자 폭 합계가 아니라 현재 line 후보 segment를 다시 측정한다.
- pair의 두 glyph가 다른 줄로 갈라지면 앞줄·뒷줄을 각각 재측정해 crossing adjustment를 제거한다.
- segment 256, code point/glyph 4,096, pair 4,095, trace 4,096 상한을 동시에 적용한다.

### R4D — layout 결정 positions의 backend 재생

- `TextRunNode`에 optional owned measurement/positions를 둔다.
- K0와 fail-closed에서는 필드를 생략해 기존 JSON/SVG byte baseline을 유지한다.
- SVG, Canvas2D, CanvasKit, native Skia, portable GlyphRun은 저장된 positions만 재생한다.
- backend는 source를 다시 찾거나 shaping하지 않는다.

### R4E — 재진입 게이트

- 공개 small face: native/WASM measurement·line break·positions 동일
- K0 공개 fixture: position/paint JSON/SVG byte identity
- K1 ratio 100/90/80, spacing 0/-5/-10 적용 순서
- pair line crossing 제거와 long-word fallback
- source unavailable·unsupported·malformed·상한 초과 fail-closed
- Docker WASM code-only 증가량과 build time
- full font payload와 private identity 부재

## 4. 단계별 중단선

- R4A에서 slot 기반 exact source를 native/WASM 공통 core API로 표현할 수 없으면 중단한다.
- self-referential face cache, leaked bytes, global/thread-local registry는 사용하지 않는다.
- embedded font를 layout 전에 bounded load할 수 없거나 현행 lazy-load 보안 상한을 약화해야 하면 중단한다.
- K0가 optional field 도입만으로 byte drift하면 R4B 전에 중단한다.
- product line boundary에 같은 measurement를 공급하지 못하고 paint-only 조정이 필요하면 중단한다.
- 유의미한 WASM size/time 회귀나 실제 stored-valid overflow가 나오면 R4E에서 재계획한다.

## 5. 권고안

R4를 한 번에 구현하지 않고 R4A→R4B→R4C→R4D→R4E로 커밋·승인 경계를 둔다. 먼저 R4A에서 exact source
owner와 수명을 해결해야 이후 수치가 실제 제품 source의 결과가 된다. 이 선행 없이 measurement 함수만 만들면
R3 portable test의 확장일 뿐 #4968의 end-to-end 구현이 아니다.

메인테이너가 이 수정 수행계획을 승인하면 R4A부터 시작한다.
