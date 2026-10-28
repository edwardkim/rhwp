# Task M100 #4968 — Stage W9-Q3-5R4E 재진입 감사와 종료 수행계획

- 작성일: 2026-08-27 KST
- 작업 브랜치: `task_m100_4968`
- 기준 커밋: `beeba689a` R4D bounded replay·최종 검증
- 최신 통합 기준: `upstream/devel@9be8b0562`
- 동기화 상태: `upstream/devel` 대비 0 behind / 27 ahead
- 상태: **R4E 진입 감사 완료, 수행계획 승인 대기**
- 이번 감사의 제품 source 변경: 0
- remote push·PR·comment: 수행하지 않음

## 1. 결론

R4D까지 #4968의 제품 경로는 style request → exact slot source → bounded measurement → fresh line boundary →
최종 positions → visual backend replay로 연결됐다. K0 identity, native 제품 K1, backend 공통 replay와 자원 상한도
통과했다.

그러나 R4E의 원래 재진입 게이트 중 한 항목은 아직 실제 실행 증거가 부족하다. Docker로 만든 배포 WASM에는
`HwpDocument.registerExactFontSource(charShapeId, languageIndex, fontBytes, faceIndex)`가 공개돼 있지만, 이
API로 exact source를 등록한 뒤 **K1 measurement·줄 경계·positions·Canvas replay를 native 결과와 교차한
runtime 하니스가 없다.** 지금까지의 WASM 증거는 다음 두 종류다.

- 작은 공개 face의 provider/session/pair candidate를 직접 실행한 target-neutral test
- exact source를 등록하지 않은 K0 문서의 native/WASM layer-tree·SVG identity

따라서 R4E는 새 shaping 기능을 추가하는 단계가 아니다. 별도 공개 runtime fixture와 Docker WASM Node
하니스로 남은 K1 교차 증거를 닫고, 이슈 본문의 완료 조건을 전항 대사한 뒤 #4968의 PR 준비 여부를 판정하는
종료 단계다.

## 2. 현재 완료 조건 감사

| #4968 완료 조건 | 현재 근거 | R4E 판정 |
|---|---|---:|
| kerning flag가 style resolution부터 최종 positioning까지 전달 | Q3-1, R4C, R4D | 충족 |
| controlled on/off fixture와 비대상 무변화 | Q2 fixture, 공개 small face native integration | native 충족 |
| stored `LineSeg`와 fresh layout 분리 | Q0 cohort, R4C stored/fresh·container tests | 충족 |
| 실제 사용 aggregate cohort 보고 | Q0 157문서·175,466자 비식별 집계 | 충족, 재계측 불필요 |
| GPOS·legacy kern capability와 glyph/cluster gate | Q2, Q3-2~4 | 충족 |
| source 부재·malformed·상한 초과 구조화 fail-closed | Q3·R4D boundary tests | 충족 |
| native·WASM·Canvas2D·CanvasKit pair positioning parity | K0 parity와 공통 replay는 충족 | **K1 Docker runtime 증거 부족** |

이 표에서 마지막 한 행만 R4E의 제품 종료를 막는다. 10k corpus를 다시 전수 계측하거나 fallback metric face를
다시 바꾸는 일은 정당화되지 않는다.

## 3. 코드·런타임 근거

### 3.1 WASM 제품 API는 이미 있다

- Rust: `src/wasm_api.rs::register_exact_font_source`
- 생성 JS: `pkg/rhwp.js::HwpDocument.registerExactFontSource`
- 생성 type: `pkg/rhwp.d.ts`
- core owner: `DocumentCore::register_exact_font_source_native`

이 API는 family 이름을 재탐색하지 않고 `(char_shape_id, language_index)` slot에 bytes와 face index를 직접
결합한다. 등록 성공 시 measurement context와 pagination·page-tree cache를 무효화한다. 따라서 R4E는 새
제품 API를 발명할 이유가 없다.

### 3.2 Studio 자동 연결은 현재 별도 경계다

`rhwp-studio`의 font loader와 WASM bridge에는 `registerExactFontSource` 호출이 없다. Studio가 local/webfont
bytes를 어떤 HWP char-shape slot의 최종 선택 face로 확정하는지는 fallback·selection owner 문제다.

R4E에서 이를 family 이름 추측으로 급히 연결하면 #4968이 금지한 metric/fallback 혼합과 provenance 분리가
재발한다. R4E는 공개 runtime 하니스가 명시적인 exact slot 등록 API를 호출해 core와 backend parity를
검증한다. Studio 자동 slot binding이 필요하다는 최종 판정이 나오면 별도 이슈로 분리하고 #4968의 pair
positioning 수학과 섞지 않는다.

### 3.3 과거 Q2 정본은 변형하지 않는다

`kerning_pair_fixture.hwpx`는 Noto Sans KR과 한컴 2020 판정의 역사적 정본이다. 현재 K1 runtime 검증을 위해
그 manifest의 font identity를 1,236-byte synthetic face로 바꾸면 Q2 계보가 깨진다.

R4E는 `RHWPExactKerningSmoke.ttf`의 family·SHA·pair truth를 명시한 별도 generated HWPX와 manifest를 둔다.
기존 fixture 생성 helper를 재사용하되 Q2 파일과 baseline JSON은 byte-for-byte 유지한다.

## 4. R4E 절편

### R4E-0 — 공개 runtime fixture와 canonical projection 계약

별도 deterministic fixture는 다음을 포함한다.

- 추적 face: `tests/fixtures/fonts/RHWPExactKerningSmoke.ttf`
- 크기 1,236 bytes, SHA-256
  `775667d1980cd734e331f01e9390e02191bc35d669325291c842968cb0a4a9fc`
- GPOS truth: `AV=-80`, `To=-40`, `WA=0`, `HH=0`, unitsPerEm 1,000
- ratio 100/90/80 × spacing 0/-5/-10 × K0/K1
- 같은 on/off pair의 stored/fresh lane 일치
- body와 최소 table-cell·text-box context
- K1 exact slot 목록과 `languageIndex=1`, `faceIndex=0`을 manifest에 명시
- font bytes는 HWPX·manifest·WASM binary에 내장하지 않음

canonical projection은 target pointer-width sentinel만 기존 규칙대로 정규화한다. 원문·source path·private
identity를 결과에 넣지 않고 다음 값만 비교한다.

- request/capability/disposition/fallback reason
- ratio·spacing·lane·context
- measurement total과 line start/boundary
- 최종 `layout_positions`와 bbox width
- Canvas command positions와 CanvasKit replay positions
- SVG digest와 page count

### R4E-1 — native·Docker WASM K1 실제 교차 실행

같은 fixture bytes와 exact face bytes를 사용한다.

1. native는 `DocumentCore::register_exact_font_source_native`로 manifest의 slot을 등록한다.
2. Docker WASM Node runner는 `HwpDocument.registerExactFontSource`로 같은 slot·bytes·face index를 등록한다.
3. 양쪽에서 등록 전 K0와 등록 후 K1을 각각 실행한다.
4. shared projection을 canonical 비교한다.
5. K1은 적어도 한 adjusted pair와 한 line/positions 차이를 보여야 하고, K0는 기존 byte identity를 유지해야
   한다.

저장소 전체 `wasm-pack test --tests` 토폴로지를 고치는 우회는 R4E에 넣지 않는다. 실제 배포 산출물인 Docker
`pkg`를 Node에서 실행해 public WASM API 경계를 직접 검증한다. native 결과를 얻기 위해 제품 CLI에 임시
family-name 특례를 추가하지 않는다. 기존 integration source와 검증용 projection helper를 우선 재사용하며,
새 integration source·generated suite를 source PR에 넣지 않는다.

### R4E-2 — matrix·실패 경계와 backend disposition

교차 하니스와 기존 integration을 함께 사용해 다음을 닫는다.

- ratio 100/90/80에서 pair delta가 한 번만 scale됨
- spacing 0/-5/-10은 glyph별 spacing으로 남고 pair delta에 다시 곱해지지 않음
- line crossing pair 제거와 long-word fallback의 native/WASM 동일성
- stored-valid boundary 유지와 fresh boundary 재계산 분리
- source unavailable, wrong digest/face, unsupported, malformed, 32MiB 초과
- 4,096 scalar / 4,097 positions 상한과 CanvasKit bounded-work
- SVG, Canvas command, CanvasKit plan, paint JSON이 layout owner positions만 재생

Canvas2D/CanvasKit이 source를 다시 찾거나 자체 kerning을 켜는 방식으로 결과를 맞추면 실패다.

### R4E-3 — 최종 gate·이슈 disposition

- 공개 runtime fixture·manifest 재생성 byte identity
- focused native integration과 Node unit/runtime 하니스
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`
- `cargo check --locked --all-targets`, clippy, wasm32 lib check
- Docker WASM build·code-only size/time 참고 계측
- WASM blob에서 full Noto font·smoke fixture name/SHA·private identity 부재 probe
- K0 native/WASM canonical identity 재확인

R4E가 production Rust를 바꾸지 않고 검증 source·fixture·문서만 추가한다면 R4D의 동일 product head 전체
nextest·native-skia 결과를 재사용하고 변경 영향 gate를 실행한다. production Rust를 한 줄이라도 바꾸거나
runtime 하니스가 새 결함을 드러내면 전체 nextest·native-skia·Docker WASM을 다시 실행한다.

최종 보고서는 이슈 본문의 범위·비범위·보호 불변식·완료 조건을 전항 대사한다. 충족하면 PR 준비 단계로
넘어가고, 미충족이면 구체적 후속 이슈와 #4968 차단 여부를 분리한다. GitHub issue close는 PR merge와
후속 검증 뒤 별도 승인을 받아 수행한다.

## 5. 보호 불변식

1. Q0~Q2 공개 정본과 한컴 판정 JSON을 소급 변경하지 않는다.
2. exact source는 family 이름이 아니라 manifest의 slot·bytes·face index로만 등록한다.
3. K0 출력과 exact source 부재 문서는 R4 진입 전과 동일하다.
4. native와 WASM이 같은 face·request·lane에서 같은 measurement·line·positions를 낸다.
5. backend는 source lookup·shaping을 다시 하지 않고 layout positions만 재생한다.
6. full font bytes를 core/WASM/fixture에 내장하지 않는다.
7. private corpus 원본·본문·파일명·경로·문서 hash를 새 증적에 넣지 않는다.
8. 기존 32MiB font, 4,096 scalar/glyph, 4,095 pair, 256 segment 상한을 낮추거나 우회하지 않는다.
9. stored `LineSeg` 유효성은 version 분기가 아니라 현재 구조의 feature detection으로 유지한다.
10. GSUB·vertical metrics·variable axis는 #4969 범위로 남긴다.

## 6. 중단·재계획 조건

- Docker WASM 공개 API가 exact source 등록 뒤에도 layout cache를 무효화하지 못함
- native/WASM K1 projection이 target별 상수나 허용 오차 확대 없이는 같아지지 않음
- runtime fixture를 위해 Q2 정본 또는 full Noto bytes를 변형·내장해야 함
- Canvas2D/CanvasKit이 layout positions 위에 자체 pair adjustment를 중복 적용함
- K0 JSON/SVG/Canvas output이 새 하니스·fixture 외 제품 source 변경 없이 drift함
- R4E 증거를 만들기 위해 저장소 전체 WASM integration topology나 Studio fallback을 함께 고쳐야 함
- 실제 stored-valid 문서 overflow나 의미 있는 code-only size 회귀가 확인됨

중단 조건이 발생하면 결과를 숨기거나 우회하지 않고 별도 수정 수행계획을 작성한다.

## 7. 승인 경계

R4E-0 → R4E-1 → R4E-2 → R4E-3 순서로 진행한다. 우선 R4E-0에서 historic Q2 정본과 분리된 공개 runtime
fixture·projection 계약을 고정한다. 메인테이너가 이 수행계획을 승인하기 전에는 fixture 생성, 제품/test
구현, 단계 커밋, remote push·PR·comment를 수행하지 않는다.
