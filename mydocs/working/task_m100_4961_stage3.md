# Task M100 #4961 Stage 3 — backend capability와 paint trace

- **Issue**: [#4961](https://github.com/edwardkim/rhwp/issues/4961)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4961.md`](../plans/archives/task_m100_4961.md)
- **브랜치**: `local/task4961-font-decision-trace`
- **기준**: Stage 2 commit `950de3156c57bb872ea65c0dc0ed962a42818549`
- **날짜**: 2026-08-17 KST
- **단계 상태**: Stage 3 기술 완료, Stage 4 메인테이너 승인 대기

## 1. 결과

Stage 3은 Stage 2의 portable layout trace를 native Skia·Canvas2D·CanvasKit의 현재 paint capability로
보강했다. 기존 `getRendererDiagnostics`는 변경하지 않고 별도 `getFontDecisionTrace` RPC와 공개 editor
SDK 메서드를 추가했다.

| backend | 관측 결과 |
| --- | --- |
| native Skia | 준비된 renderer snapshot에서 실제 text replay와 공유하는 custom → system → bundled → legacy 후보 및 문자 glyph hit/miss |
| Canvas2D | 실제 CSS 후보 순서, 현재 local/web/generic supply, Local Font Access 상태와 실제 glyph face 미관찰 |
| CanvasKit | 이미 준비된 local/bundled/default/symbol SFNT 후보와 문자 glyph hit/miss, glyph resource join 실패 |

backend `status`와 `certainty`는 분리했다. 후보를 전부 관찰했지만 glyph가 없었던 결과는
`status=complete`, `certainty=observed`, `resolved=null`과 명시적 failure로 나타낸다. API나 snapshot이
없어 관찰 자체를 하지 못한 결과는 `unsupported` 또는 `notObserved`다.

## 2. 구현 경계

### 2.1 native Skia

- `font_lookup.rs`에 실제 후보 열거와 문자 glyph 선택 helper를 추출했다.
- `text_replay.rs`의 기존 paint 경로와 trace가 같은 helper를 사용한다.
- `native-skia` feature와 준비된 `SkiaLayerRenderer` snapshot이 있으면 요청 face와 문자별 후보·source·
  glyph 결과를 기록한다. feature가 없으면 `nativeSkiaFeatureUnavailable`, snapshot이 없으면
  `nativeRendererSnapshotRequired`로 fail-closed한다.
- trace 관측은 surface를 만들거나 문서를 다시 그리지 않는다.

> PR self-review에서 `DocumentCore` 단독 query가 빈 custom·bundled inventory를 가진 새 renderer를
> 관측하던 결함을 확인했다. Stage 6 보정 뒤 native 완료 판정은 이미 준비된 renderer snapshot을
> 전달한 경로에만 적용된다.

### 2.2 Studio RPC와 공개 SDK

- Embed capability `font-decision-trace-v1`과 `getFontDecisionTrace({page, limits})`를 추가했다.
- `page`는 0 이상의 safe integer, `maxCharacters`는 1..4,096 safe integer만 허용한다.
- 알려지지 않은 parameter와 limit field는 거부하며 기본값이나 상한으로 조용히 보정하지 않는다.
- `@rhwp/editor`의 공개 API·타입·60초 long-running transport 분류를 함께 추가했다.

### 2.3 Canvas2D

- 실제 CSS 출력과 공유하는 `fontFamilyCandidatesForDisplay`에서 ordered chain을 얻는다.
- 현재 저장된 Local Font Access/probe snapshot과 이미 등록·로드된 webfont 상태만 읽는다.
- 브라우저가 실제 glyph face를 공개하지 않으므로 `resolved`를 추정하지 않고 항상
  `cssActualGlyphFaceUnobservable`을 남긴다.
- `localFontApiUnsupported`, `localFontPermissionDenied`, `localFontSnapshotUnavailable`,
  `localFontEnumerationPartial`을 서로 다른 failure로 유지한다.

### 2.4 CanvasKit

- trace callback은 face 문자열이 아니라 `recordId`와 source 좌표를 포함한 record 전체를 받는다.
- 요청 page에 기존 CanvasKit render diagnostics가 없으면 `backendJoinMissing`으로 끝낸다.
- 일반 text replay에서는 이미 준비된 local/bundled/default/symbol typeface만 읽고 실제 paint와 같은
  순서로 `getGlyphIDs`를 실행한다. font fetch나 SFNT 준비를 시작하지 않는다.
- 해당 page가 portable glyph resource replay를 사용했지만 현재 source record와 resource를 안전하게
  연결할 수 없으면 일반 SFNT 결과로 가장하지 않는다. `notObserved`, `backendJoinMissing`,
  `canvaskitGlyphResourceSourceUnresolved`로 fail-closed한다.
- backend가 없는 경우에도 순수 SFNT plan snapshot이 있으면 `planned`, 없으면
  `canvaskitApiUnsupported`와 `canvaskitSfntAbsent`를 구분한다.

## 3. 부작용 방지

trace 보강 함수는 다음 동작을 호출하지 않는다.

- `fetch` 또는 local font byte load
- `queryLocalFonts`와 권한 요청
- Canvas/CanvasKit paint와 repaint
- page layout 또는 renderer backend 재선택

Studio test는 `fetch`와 `queryLocalFonts`를 trap으로 교체해 호출 횟수가 모두 0임을 확인했다. CanvasKit
경로는 이미 생성된 renderer·page diagnostics와 준비된 typeface만 읽는다. trace를 요청하지 않는 기본
경로에는 새 작업이 없다.

## 4. 검증 결과

### 4.1 Rust native·WASM

```bash
cargo check --no-default-features
cargo check --features native-skia
cargo check --target wasm32-unknown-unknown --lib
cargo clippy --no-default-features --lib -- -D warnings
cargo clippy --features native-skia --lib -- -D warnings
cargo test --lib --features native-skia document_core::queries::font_decision
cargo test --lib --features native-skia renderer::skia::font_lookup
cargo fmt --check
```

- font decision query: **2 passed**
- native 후보·family 검사: **3 passed**
- 두 feature 조합 check·clippy, WASM check와 format: 통과

### 4.2 Studio·Embed·SDK

```bash
cd rhwp-studio
node --test tests/font-decision-trace.test.ts tests/embed-protocol.test.ts
npm run build

cd ..
node scripts/frontend-editor-embed.test.mjs
node --test npm/editor/tests/transport.test.mjs npm/editor/tests/renderer-diagnostics.test.mjs
```

- trace·Embed focused: **22 passed**
- public editor embed: **2 passed**
- editor transport·diagnostics: **13 passed**
- TypeScript와 Vite production build: 통과

추가로 font substitution·offline loader·CanvasKit font plan·local font 회귀 **24건**을 포함한 Studio
focused 묶음 **46건**이 통과했다. Vite의 기존 대형 chunk 경고는 non-failing이며 새 오류는 없었다.

### 4.3 schema·W1 계약

```bash
node --test scripts/tests/font_decision_trace_contract.test.mjs
node scripts/font_decision_trace_contract.mjs check
git diff --check
```

- trace contract: **12 passed**
- repository contract: `font decision trace Stage 1 contracts: ok`
- whitespace 검사: 통과

관찰 완료 후 모든 후보에 glyph가 없는 정상 결과를 허용하되 failure 없는 `observed + resolved=null`은
거부하는 회귀를 추가했다. W1 원장은 갱신하지 않았다. Stage 2의 4개 source에 native·Studio shared
helper 5개가 더해져 예상 historical digest drift는 9개이며, 그 밖의 drift는 test가 거부한다.

## 5. 종료 게이트 판정

| 항목 | 판정 |
| --- | --- |
| 한 record에서 native·Canvas2D·CanvasKit 결과 분리 | 통과 |
| CSS 미관찰·API 미지원·권한 거부·부분 열거 구분 | 통과 |
| SFNT 부재·renderer snapshot 부재·source join 실패 구분 | 통과 |
| 준비된 native snapshot과 Canvas2D 후보 helper 공유 | 통과 |
| font load·권한 요청·repaint·layout·backend switch 미발생 | 통과 |
| private corpus·재배포 불가 font bytes 포함 금지 | 통과 |

## 6. Stage 4 인계

- 공개 HWP/HWPX fixture에서 실제 RPC 전체 계보와 backend normalized trace를 고정한다.
- CanvasKit glyph resource page는 source-specific join이 증명되기 전까지 명시적 `backendJoinMissing`이다.
  Stage 4 fixture가 이 fail-closed 결과를 검증하며, exact join fixture가 필요하면 record/source key 연결을
  별도 구현하고 같은 단계에서 계약으로 고정한다.
- object key와 font enumeration 순서 변이, 반복 실행, 상한 초과와 unsupported backend를 end-to-end로
  검증한다.
- Stage 5의 전체 renderer output 0-delta와 FI-01~FI-14 감사는 아직 수행하지 않았다.

## 7. 다음 승인 지점

Stage 3 변경은 이 보고서를 포함한 별도 commit으로 고정한다. 메인테이너가 Stage 4 진행을 승인하면
공개 HWP/HWPX end-to-end fixture와 결정론 검증을 시작한다. remote push와 PR 생성은 여전히 별도 승인
대상이다.
