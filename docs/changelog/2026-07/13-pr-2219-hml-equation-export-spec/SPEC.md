# PR 2219 후속 패치 명세: HML 수식 Import/Edit/Export 및 Embed RPC

## Intent

- Domain: EXECUTE
- Artifact: PR 2219 후속 기술 명세
- Decision: `pr/2219`에 HML 수식 round-trip과 외부 embed HML export를 어떤 범위·계약·검증으로 추가할지 확정한다.
- Completion bar: 개발자가 이 문서만으로 RED fixture를 고정하고, 최소 패치를 구현하고, 브라우저에서 실제 수학 HML을 열어 수정·저장·재열기까지 검증할 수 있어야 한다.
- 기준 revision: `pr/2219` / `3f1db67d` (`Merge branch 'devel' into pr/2219`)
- 상태: 구현 전 상세 스펙. 이 문서는 코드 변경이나 배포 승인을 의미하지 않는다.

## 1. Problem / Context

### 사용자 문제

ExamBank는 오브젝트 스토리지의 HML 문제를 웹 편집기에서 열고, 특히 수식을 수정한 뒤 HML로 다시 저장해야 한다. 현재 호스팅된 `rhwp-studio`는 HML 파일과 일반 텍스트는 열지만 `<EQUATION>`을 미지원 요소로 건너뛴다. 사용자는 문장이 `다항식 을 전개하시오.`처럼 수식 자리만 빈 상태로 보게 된다.

### 재현 증거

- 호스트: `https://hwp-editor.agentic-worker.store/`
- 런타임 표시 버전: `rhwp 0.7.18`
- fixture: `/Users/chaeseong-gug/Documents/PARA/Project/ExamBank/exambank-generator/tests/fixtures/serial_curated_min.hml`
- fixture 크기: 4,087 bytes
- fixture 수식: `<EQUATION>` 4개
- SCRIPT 순서: `x^2 +1`, `x^2 +1`, `3`, `3`
- 실제 경고: `/HWPML/BODY/SECTION/P/TEXT/EQUATION` 미지원. 총 5개 경고는 수식 4개와 보존 가능한 `TAIL/BINDATASTORAGE` 1개로 재현된다.
- 실제 화면: `docs/changelog/2026-07/13-hml-web-poc/exambank-real-math-hml-opened.png`

### 코드상 직접 원인

1. `src/parser/hml/reader.rs`
   - `HmlControl`은 현재 `Rectangle`, `Table`만 표현한다.
   - `is_unsupported_inline()`은 `TEXT` 하위에서 `EQUATION`을 허용하지 않는다.
   - `warn_if_unsupported()`는 `EQUATION`을 명시적 미지원 요소로 분류한다.
2. `src/parser/hml/adapter.rs`
   - HML 중간 모델에서 `Control::Equation`으로 변환하는 분기가 없다.
3. `src/serializer/hml/body.rs`
   - `write_control_run()`은 `Table`, `Shape(Rectangle)`만 출력하고 나머지는 `HML_UNSUPPORTED_IR`로 거부한다.
4. `src/serializer/hml/preflight.rs`
   - `Control::Equation`은 저장 blocker가 된다.
5. `rhwp-studio/src/embed/rpc-router.ts`, `rhwp-studio/src/main.ts`
   - WASM `exportHml()`은 존재하지만 embed RPC는 `exportHwp`, `exportHwpx`만 노출한다.

### 기존 자산

- 공용 모델: `src/model/control.rs::Equation`
- HWPX 수식 파서: `src/parser/hwpx/section.rs::parse_equation`
- 새 수식 기본값: `src/document_core/commands/object_ops/equation.rs::insert_equation_native`
- WASM HML export: `src/wasm_api.rs::HwpDocument::export_hml`
- HML export preflight/save state: `src/document_core/commands/document.rs`
- embed transferable `Uint8Array` 응답: `rhwp-studio/src/embed/runtime.ts`

### 선행 Validation Gates

구현자는 다음 사실을 검증하기 전 parser/serializer 동작을 확정하지 않는다. 결과는 구현 run vault에 명령과 fixture 근거로 기록한다.

| Gate | 검증 대상 | 방법 | 통과 조건 | 실패 시 |
|---|---|---|---|---|
| VG-01 | inline control slot | 기존 HWP/HWPX equation의 `char_offsets`, `char_count`, `control_text_positions`와 interleaved HML fixture 비교 | 수식 1개가 8 UTF-16 raw units를 소비 | 임의 offset을 구현하지 않고 모델/offset 설계를 별도 결정 |
| VG-02 | `BaseUnit` 의미 | HML 값, HWPX `font_size`, 새 수식 insertion, 렌더 결과 비교 | 같은 `1000` 입력이 같은 시각 크기와 round-trip 값을 가짐 | raw numeric 보존만 하고 렌더 매핑은 blocker로 남김 |
| VG-03 | `TextColor` 의미 | `0`, 비대칭 RGB fixture, HWPX color path 비교 | 렌더 색과 export numeric 값 모두 일치 | endian 변환을 문서화하고 expected 갱신 |
| VG-04 | zero-size common attrs | HML minimal equation을 기존 renderer로 시각 검증 | SCRIPT가 non-zero bounds로 표시 | insertion/layout 기본값을 재사용하되 근거 없는 크기 상수 금지 |
| VG-05 | casing/schema | repo-owned fixture와 corpus에서 attribute/child spelling 집계 | P0 allowlist가 실제 입력을 포함 | variant를 명시적으로 추가하거나 blocker로 유지 |

## 2. Goals and Success Metrics

### Goal G1 — 기존 HML 수식 손실 제거

ExamBank fixture의 수식 4개가 올바른 인라인 위치에서 렌더링되어야 한다.

- Metric M1: import 후 `Control::Equation` 개수 `4/4`.
- Metric M2: import metadata에서 `EQUATION` 관련 non-preserved warning `0`.
- Metric M3: 브라우저에서 네 SCRIPT가 모두 시각적으로 확인된다.

### Goal G2 — 수식 편집 후 HML round-trip

기존 수식 SCRIPT를 편집하고 HML export 후 재열었을 때 편집 결과와 나머지 수식이 보존되어야 한다.

- Metric M4: `x^2 +1` 하나를 `x^3 +2`로 수정 후 export/reparse 결과가 정확히 반영된다.
- Metric M5: 수정하지 않은 SCRIPT 3개는 byte-for-byte 같은 문자열로 유지된다.
- Metric M6: export 후 재열기에서 수식 개수 `4/4`, 위치 `4/4` 유지.

### Goal G3 — 부모 웹 앱이 HML bytes를 받을 수 있는 embed 계약

- Metric M7: `exportHml` RPC가 transferable `Uint8Array`를 반환한다.
- Metric M8: 지원 불가능한 문서는 쓰기 없이 구조화된 오류를 반환한다.
- Metric M9: 기존 embed RPC 테스트 전체 통과 및 기존 메서드 계약 변화 `0`.

### Goal G4 — 호스팅 배포에서 실제 흐름 증명

- Metric M10: `ExamBank GET bytes -> iframe loadFile -> edit -> exportHml -> local re-open` 브라우저 E2E 통과.
- 오브젝트 스토리지 PUT은 ExamBank 측 책임이므로 이 PR에서는 실제 외부 쓰기를 수행하지 않는다.

## 3. Non-goals

- ExamBank 저장 API, presigned URL 발급, 인증, 충돌 해결 구현
- `rhwp-studio`가 오브젝트 스토리지에 직접 접근하는 기능
- HWP/HWPX 출처 문서를 임의로 HML로 변환하는 기능
- PR 2219에서 모든 HWPML 요소를 한 번에 완전 지원하는 작업
- `PICTURE`, OLE, 차트, 스크립트 실행 등 수식과 무관한 미지원 컨트롤 구현
- 원본 HML과 byte-identical export 보장. 의미 보존 round-trip이 목표다.
- 경고를 숨기거나 HML save blocker를 강제로 우회하는 처리

## 4. Users and Use Cases

### Primary job story

When ExamBank 검수자가 수학 문제를 검토할 때, I want HML 원본의 수식을 웹에서 보고 수정하고 HML bytes로 되받아, so I can 기존 HML 기반 파이프라인을 유지하면서 문제를 교정할 수 있다.

### Use cases

- UC-01: 부모 앱이 object storage에서 받은 HML bytes를 `loadFile`로 전달한다.
- UC-02: 사용자가 기존 수식을 선택해 수식 편집기에서 SCRIPT를 수정한다.
- UC-03: 부모 앱이 `exportHml`을 호출해 bytes를 받고 자체 저장 API로 PUT한다.
- UC-04: 저장 불가능한 요소가 있으면 부모 앱이 blocker path/code를 표시하고 업로드하지 않는다.
- UC-05: 저장된 HML을 다시 열어 수식과 일반 서식이 보존되었는지 검증한다.

## 5. Requirements

### 5.1 P0 Functional requirements — HML equation import

#### REQ-HML-EQ-001 — Reader 중간 모델

`HmlControl`에 `Equation(HmlEquation)`을 추가한다. `HmlEquation`은 최소 다음 데이터를 가진다.

| HML | IR target | Rule |
|---|---|---|
| `EQUATION/SCRIPT` text | `Equation.script` | XML entity decode 후 문자열 보존 |
| `BaseUnit` | `Equation.font_size` | VG-02 통과 후 매핑. 누락 시 기존 수식 삽입 기본값 `1000` |
| `TextColor` | `Equation.color` | VG-03 통과 후 매핑. export numeric 값 보존 |
| `BaseLine` | `Equation.baseline` | 누락 시 `0` |
| `Version` | `Equation.version_info` | 문자열 보존 |
| optional `Font` | `Equation.font_name` | 누락 시 기존 기본값 `HYhwpEQ` |

HML 속성명은 fixture의 PascalCase를 기준으로 처리한다. 다른 casing 지원은 VG-05 증거가 생기기 전까지 확장하지 않는다.

`HmlEquation`은 allowlist 밖 속성/자식의 path/name/value를 import metadata warning으로 남긴다. P0 allowlist는 `BaseLine`, `BaseUnit`, `TextColor`, `Version`, optional `Font` 및 단일 `SCRIPT`다. 그 밖의 의미는 읽기 자체를 실패시키지 않지만 `preserved=false` warning을 생성하여 HML export blocker가 된다. 알 수 없는 의미를 읽을 때 버리고 저장을 허용하는 경로는 금지한다.

Durable diagnostic 경로는 새 저장소를 만들지 않고 기존 흐름을 확장한다.

1. reader가 `HmlWarningCode::UnsupportedEquationSemantics`와 exact path를 생성한다.
2. `parse_document_with_metadata()`가 warning을 `HmlImportMetadata.warnings`로 이동한다.
3. `DocumentCore::from_bytes()`가 `hml_metadata`를 document IR과 별도로 소유한다.
4. 일반 텍스트/수식 편집, undo/redo 중에도 metadata warning을 유지한다.
5. `hml_export_preflight()`가 non-preserved warning을 `HML_UNSUPPORTED_EQUATION_SEMANTICS` blocker로 변환한다.
6. P0에서는 warning을 자동 삭제하지 않는다. 이미 버린 unknown 의미의 안전한 제거를 증명할 수 없기 때문이다. 전체 문서 replacement/reload만 metadata를 교체한다.

#### REQ-HML-EQ-002 — Inline control position

- VG-01 통과 후 `EQUATION` 시작 시 `reserve_control_slot()`과 동일한 8 UTF-16 raw position을 소비한다.
- 앞·뒤 `CHAR`의 `char_offsets`, `char_shapes`, `char_count`가 유지되어야 한다.
- HML `TEXT` 하위 수식은 VG-04 통과 후 `CommonObjAttr { ctrl_id: CTRL_EQUATION, treat_as_char: true, width: 0, height: 0, ..Default }`로 adapter에서 생성한다.
- 수식 표시 크기는 임의 pixel 상수를 추가하지 않고 기존 equation layout 경로가 `font_size`와 SCRIPT로 계산하게 한다.

#### REQ-HML-EQ-003 — Reader event handling

- `EQUATION`을 `is_unsupported_inline()` 허용 목록에 추가한다.
- `warn_if_unsupported()`의 명시적 미지원 목록에서 `EQUATION`을 제거한다.
- `SCRIPT` 내부의 `Text`, `CDATA`, XML character/general reference를 손실 없이 수집한다.
- `EQUATION` 종료 시 현재 문단의 controls에 정확히 한 번 추가한다.
- allowlist 밖 attribute/child와 두 번째 `SCRIPT`는 정확한 XML path의 non-preserved warning으로 기록한다.
- 중첩/종료 불일치 XML은 `HmlError::InvalidXml`로 실패하며 부분 성공으로 위장하지 않는다.

#### REQ-HML-EQ-004 — Adapter mapping

`src/parser/hml/adapter.rs::into_control()`에 `HmlControl::Equation -> Control::Equation`을 추가한다. `raw_ctrl_data`와 HWP 전용 `unknown`은 HML 출처에서 기본값을 사용한다.

### 5.2 P0 Functional requirements — edit and HML export

#### REQ-HML-EQ-005 — Existing equation editor compatibility

HML에서 읽은 `Control::Equation`은 새로 삽입한 수식과 같은 선택·속성·편집 경로를 사용해야 한다. HML 전용 편집 UI나 복제된 parser를 만들지 않는다.

#### REQ-HML-EQ-006 — HML serializer

`src/serializer/hml/body.rs::write_control_run()`에 `Control::Equation` 분기를 추가한다.

출력 형식:

```xml
<TEXT CharShape="...">
  <EQUATION BaseLine="65" BaseUnit="1000" TextColor="0" Version="Equation Version 60">
    <SCRIPT>x^2 +1</SCRIPT>
  </EQUATION>
</TEXT>
```

Rules:

- SCRIPT는 `XmlWriter::text()`를 사용해 escape한다.
- `Font`는 원본/모델 값이 비어 있지 않을 때만 출력한다.
- 현재 HML canonical serializer의 attribute ordering을 고정해 테스트를 결정적으로 만든다.
- `Control::Equation`은 8-unit control slot으로 paragraph offset을 유지한다.
- 미지원 속성을 조용히 삭제하지 않는다. import metadata의 unknown equation semantics는 `HML_UNSUPPORTED_EQUATION_SEMANTICS` blocker와 원래 XML path로 export를 거부한다.

#### REQ-HML-EQ-007 — Preflight

- 지원 범위의 `Control::Equation`은 더 이상 `HML_UNSUPPORTED_IR` blocker가 아니다.
- 다음은 blocker다.
  - 유효하지 않은 XML character를 포함한 SCRIPT
  - paragraph control offset 불일치
  - 지원한다고 선언하지 않은 equation semantics가 실제 모델에 존재하고 의미 보존이 불가능한 경우
- blocker는 export 전에 모두 수집하며 부분 HML bytes를 반환하지 않는다.

### 5.3 P0 Functional requirements — embed Export HML

#### REQ-EMBED-HML-001 — RPC method

`EmbedRpcHandlers`와 `routeEmbedRequest()`에 다음 메서드를 추가한다.

```ts
exportHml(): Promise<Uint8Array>
```

- method name: `exportHml`
- params: 없음
- success result: transferable `Uint8Array`
- failure: 기존 `rhwp-response.error` envelope 사용
- source format이 HML이 아니거나 preflight blocker가 있으면 bytes를 반환하지 않는다.

#### REQ-EMBED-HML-002 — Runtime handler

`rhwp-studio/src/main.ts`의 embed handler는 `await initPromise` 후 `wasm.exportHml()`을 호출한다. 별도 serializer나 다운로드 경로를 구현하지 않는다.

#### REQ-EMBED-HML-003 — Capability discovery

additive protocol v1 확장으로 `EMBED_CAPABILITIES`에 `hml-export`를 추가한다.

- 기존 필수 capability `transferable-array-buffer`는 유지한다.
- 부모 앱은 `rhwp-connected.capabilities.includes('hml-export')`로 버튼 활성화를 결정한다.
- 기존 client가 추가 capability를 무시해도 동작해야 하므로 protocol version은 `1`을 유지한다.

#### REQ-EMBED-HML-004 — Required preflight API

다음은 같은 PR의 MUST다.

```ts
getHmlSaveState(): Promise<{
  sourceFormat: string;
  hmlSavable: boolean;
  blockers: Array<{ code: string; xmlPath: string; message: string; preserved: false }>;
}>;
```

부모 앱이 export를 예외 기반으로 탐지하지 않도록 한다. 기존 WASM metadata/save-state를 재사용하고 새 판정 로직을 만들지 않는다.

`exportHml` 실패 envelope는 transport/runtime 오류 전달용이다. 저장 가능 여부와 typed blocker는 반드시 `getHmlSaveState` 결과로 판단하며 오류 문자열 파싱을 금지한다.

### 5.4 P1 — Unsupported-format inventory

수식 패치와 “모든 서식 지원”을 한 변경으로 합치지 않는다. PR 2219 후속 작업에서 read-only corpus 검사기를 추가하거나 기존 diagnostics를 사용해 다음 집계를 만든다.

| Field | Meaning |
|---|---|
| element/path | 미지원 HML 요소와 XML path |
| count/files | 발생 횟수와 영향 파일 수 |
| preserved | 원문 fragment로 round-trip 가능한지 |
| visibleImpact | 누락·레이아웃·저장 차단 영향 |
| proposedPriority | P0/P1/P2 |

우선순위 규칙:

1. 원본 문제 의미를 삭제하는 요소 — P0
2. 수식 편집/저장을 차단하는 요소 — P0
3. 레이아웃은 다르지만 내용은 보이는 요소 — P1
4. 실행 스크립트·보안 위험 요소 — 지원하지 않고 preserved/blocked 정책 유지

### 5.5 Non-functional requirements

- NFR-01 Backward compatibility: HWP/HWPX/HML 기존 parser 및 export 결과에 의도하지 않은 변화가 없어야 한다.
- NFR-02 Security: embed는 URL, credential, callback URL을 받지 않는다. 부모가 bytes를 전달하고 받는다.
- NFR-03 Atomicity: preflight 실패 시 HML bytes나 외부 성공 상태를 반환하지 않는다.
- NFR-04 Determinism: 같은 IR은 같은 canonical HML을 출력해야 한다.
- NFR-05 Complexity: equation parsing은 기존 streaming XML event loop 안에서 입력 크기에 선형이어야 하며 별도 DOM 전체 복제나 네트워크 요청을 추가하지 않는다. 숫자 성능 목표는 baseline이 없어 이번 acceptance에서 제외한다.
- NFR-06 Accessibility: 기존 수식 편집 UI의 keyboard/focus 동작을 유지한다.
- NFR-07 Diagnostics: import warning에는 `code`, `xml_path`, `message`, `preserved`가 있어야 한다. save blocker에는 같은 필드를 wire에 제공하며 `preserved`는 항상 `false`다.

## 6. Flow and API Contract

```text
ExamBank parent
  -> presigned GET (HML bytes)
  -> iframe rhwp-connect (capabilities: transferable-array-buffer, hml-export)
  -> loadFile({ data, fileName })
  -> user edits existing equation
  -> getHmlSaveState() [MUST]
  -> exportHml()
  <- Uint8Array
  -> parent-owned presigned PUT
  -> parent reloads saved bytes for verification
```

### RPC success and transfer ownership

```ts
const result = wasm.exportHml();   // handler-owned Uint8Array
const outbound = result.slice();  // 원본을 detach하지 않는 독립 복사본
port.postMessage(
  { type: 'rhwp-response', version: 1, sessionId, id: 12, result: outbound },
  [outbound.buffer],
);
```

이는 기존 `postPortResponse()` 계약이다. sender의 handler-owned `result.buffer`는 detach되지 않고 복사본 `outbound.buffer`만 transfer되어 receiver가 소유한다. zero-copy를 보장하지 않는다. receiver는 structured clone 결과의 `result instanceof Uint8Array`를 확인한다.

### RPC failure example

```json
{
  "type": "rhwp-response",
  "version": 1,
  "sessionId": "...",
  "id": 12,
  "error": {
    "code": "RPC_ERROR",
    "message": "HML_SOURCE_REQUIRED: ..."
  }
}
```

구조화된 blocker 배열은 `getHmlSaveState`에서 제공한다. `exportHml` 오류 문자열을 부모가 파싱하도록 요구하지 않는다.

### Repo-owned fixture contract

구현 시작 전 원본을 최소화·익명화한 fixture를 `tests/fixtures/hml/exambank_math_equations_min.hml`로 rhwp repo에 추가한다.

- source: ExamBank `tests/fixtures/serial_curated_min.hml`
- source SHA-256: `66998b57e70d38175e68facc3bf2fb2b7e6e0839c41c012acb47209d3071c538`
- content: 개인정보 없음, synthetic 문제 식별자, EQUATION 4개와 필요한 최소 style/page metadata
- provenance: fixture README에 source repo/path, 최소화 목적, 사용 허가 주체 기록
- CI는 repo 밖 절대경로를 참조하지 않는다.
- source 사용 허가를 기록할 수 없으면 구현을 중단하고 user-owned synthetic fixture를 새로 만든다.

## 7. Detailed Test Plan

### 7.1 RED tests first

| ID | Layer | Precondition | Action | Expected before patch | Expected after patch |
|---|---|---|---|---|---|
| T-EQ-01 | HML parser | `serial_curated_min.hml` | parse | EQUATION warning + controls 0 | equation controls 4, warning 0 |
| T-EQ-02 | HML parser | same | inspect scripts | scripts missing | exact ordered scripts 4개 |
| T-EQ-03 | offsets | 수식 앞뒤 CHAR | inspect offsets | 수식 slot 없음 | 각 수식이 8-unit slot 소비 |
| T-EQ-04 | renderer | same | render page SVG/canvas | 수식 자리 빈칸 | 4개 수식 표시 |
| T-EQ-05 | edit | first equation selected | `x^3 +2`로 수정 | 대상 없음/불가 | 기존 editor로 수정 성공 |
| T-EQ-06 | HML export | edited HML-origin doc | export/reparse | blocked | 4개 수식, 첫 script 변경 |
| T-EQ-07 | preservation | edited doc | compare untouched scripts | 해당 없음 | 나머지 3개 동일 |
| T-EQ-08 | XML escaping | SCRIPT `<`, `>`, `&` | export/reparse | blocked | 의미 동일 문자열 |
| T-EQ-09 | malformed XML | unclosed EQUATION/SCRIPT | parse | failure | 명시적 InvalidXml 유지 |
| T-EQ-10 | attributes | BaseLine/BaseUnit/Version/Font | import/export/reparse | missing control | exact mapped values 유지, VG-02 반영 |
| T-EQ-11 | color | asymmetric RGB value | render/export/reparse | missing control | VG-03에서 확정한 색과 값 일치 |
| T-EQ-12 | XML text forms | Text/CDATA/entity in SCRIPT | import/export/reparse | missing control | 의미 동일 문자열 |
| T-EQ-13 | unknown attribute | `FutureAttr="1"` | import/preflight | generic warning | `HML_UNSUPPORTED_EQUATION_SEMANTICS`, exact attribute path |
| T-EQ-14 | unknown child | `EQUATION/FUTURE` | import/preflight | generic warning | 같은 typed blocker, exact child path |
| T-EQ-15 | invalid XML char | model SCRIPT contains invalid char | preflight | blocked generically | `HML_INVALID_XML_CHARACTER`, SCRIPT path |
| T-EQ-16 | offset mismatch | char_offsets/control slot mismatch | preflight | blocked | `HML_UNSUPPORTED_IR`, paragraph/control path |
| T-EQ-17 | blocker aggregation | 2개 이상 invalid semantics | preflight | generic | 모든 blocker가 stable order로 반환 |
| T-EQ-18 | durable diagnostic | unknown attr 문서 load 후 known SCRIPT 편집 | preflight/export | generic warning may disappear | metadata warning 유지, typed blocker, export bytes 없음 |
| T-RPC-01 | embed unit | mock handler | `exportHml` | Unknown method | Uint8Array result |
| T-RPC-02 | transfer | real MessageChannel | `exportHml` | unsupported | original buffer intact, sliced outbound transfer, receiver-owned Uint8Array |
| T-RPC-03 | blocker | non-HML origin | `exportHml` | unsupported | structured RPC error, no result |
| T-RPC-04 | compatibility | existing calls | all old methods | pass | unchanged pass |
| T-RPC-05 | capability | connect | inspect capabilities | `hml-export` absent | 기존 capability와 `hml-export` 존재 |
| T-RPC-06 | save state | unsupported equation semantics | `getHmlSaveState` | unknown method | `hmlSavable=false`, typed blockers |
| T-RPC-07 | save state | supported fixture | `getHmlSaveState` | unknown method | `hmlSavable=true`, blockers empty |
| T-RPC-08 | save state durability | unknown attr load 후 수식 편집 | save state then export | unknown method | blocker `{code,xmlPath,message,preserved:false}`, export error, result 없음 |

### 7.1.1 Requirement-to-test traceability

| Requirement | Tests / gates |
|---|---|
| REQ-HML-EQ-001 | T-EQ-01, 02, 10, 11, 13, 14, 18; VG-02, 03, 05 |
| REQ-HML-EQ-002 | T-EQ-03, 04, 16; VG-01, 04 |
| REQ-HML-EQ-003 | T-EQ-09, 12, 13, 14, 18 |
| REQ-HML-EQ-004 | T-EQ-01, 02, 10 |
| REQ-HML-EQ-005 | T-EQ-04, 05 |
| REQ-HML-EQ-006 | T-EQ-06, 07, 08, 10, 11, 12 |
| REQ-HML-EQ-007 | T-EQ-13, 14, 15, 16, 17, 18 |
| REQ-EMBED-HML-001 | T-RPC-01, 02, 03 |
| REQ-EMBED-HML-002 | T-RPC-01, 02 |
| REQ-EMBED-HML-003 | T-RPC-04, 05 |
| REQ-EMBED-HML-004 | T-RPC-06, 07, 08 |

### 7.2 Required automated commands

구현 시 실제 repo 명령을 확인해 정확한 이름으로 기록한다. 최소 실행 범위:

```bash
cargo test --test hml_parser
cargo test --test hml_serializer
cargo test --test hml_cli
cargo test issue_1061_equation
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
```

### 7.3 Required browser E2E

1. HTTP origin의 parent harness에서 hosted/local `rhwp-studio` iframe 연결.
2. `serial_curated_min.hml` bytes 전달.
3. 경고에 `EQUATION`이 없음을 확인.
4. `x^2 +1`이 문제 문장과 답 위치에 보이는지 screenshot 및 DOM/accessibility snapshot 확인.
5. 기존 수식 선택 -> 수식 편집기 -> `x^3 +2` 저장.
6. `exportHml` 호출 -> bytes 수신.
7. 같은 iframe에 exported bytes 재로드.
8. 수정 수식, 미수정 수식, 문제 텍스트, 페이지 수 확인.
9. console error, page error, network error가 없어야 한다.

시각 검증은 `파일이 열림`, `pageCount=1`만으로 합격 처리하지 않는다. 수식 SCRIPT별 visible assertion이 필수다.

## 8. Implementation Slices

### Slice A — Parser and adapter

Files:

- `src/parser/hml/reader.rs`
- `src/parser/hml/adapter.rs`
- `tests/hml_parser.rs`
- ExamBank fixture를 repo-owned test fixture로 추가할 경우 출처를 문서화

Exit: VG-01~05와 parser 관련 T-EQ-01~04, 09~14, 18 green.

### Slice B — HML serializer and preflight

Files:

- `src/serializer/hml/body.rs`
- `src/serializer/hml/preflight.rs`
- `tests/hml_serializer.rs`
- `tests/hml_cli.rs` if CLI behavior is covered

Exit: serializer/preflight 관련 T-EQ-05~18 green; export/reparse semantic equivalence와 durable blocker proven.

### Slice C — Embed RPC

Files:

- `rhwp-studio/src/embed/protocol.ts`
- `rhwp-studio/src/embed/rpc-router.ts`
- `rhwp-studio/src/embed/runtime.ts` only if capability/transfer handling needs change
- `rhwp-studio/src/main.ts`
- `rhwp-studio/tests/*embed*`

Exit: T-RPC-01~08 green; `hml-export` capability와 required save-state contract proven.

### Slice D — Exact browser verification

Files:

- `rhwp-studio/e2e/*` or isolated existing embed harness
- evidence under the implementation run vault

Exit: load/edit/export/reload screenshots and exact visible-script assertions green.

### Slice E — Corpus compatibility report

수식 patch와 독립 커밋/PR로 분리 가능하다. ExamBank corpus를 읽기 전용으로 검사하고 미지원 서식 우선순위표를 생성한다.

## 9. Risks, Assumptions, Dependencies

### Risks

- R1 High: 수식 control offset을 잘못 삽입하면 뒤 텍스트, char shape, selection 위치가 모두 어긋난다.
- R2 High: import만 지원하고 serializer/preflight를 풀면 수정 후 저장에서 수식이 소실될 수 있다.
- R3 High: `exportHml`을 노출하되 blocker를 무시하면 부모 앱이 손실된 HML을 정상 저장할 수 있다.
- R4 Medium: HML minimal equation은 width/height가 없으므로 기존 renderer가 zero-size common attrs를 어떻게 해석하는지 실제 E2E 확인이 필요하다.
- R5 Medium: `BaseLine`, `TextColor`의 HML 단위/색상 endian 의미가 HWPX/HWP와 다를 수 있다. fixture 및 한컴 재열기 검증 전 일반화하지 않는다.
- R6 Medium: 호스팅 배포가 source branch와 다르면 로컬 green만으로 운영 수정이 증명되지 않는다.

### Assumptions to validate during implementation

- A1: HML `EQUATION`의 control slot은 기존 HWP/HWPX equation과 동일하게 8 UTF-16 units다.
- A2: HML `BaseUnit`은 현재 `Equation.font_size`와 같은 HWPUNIT 의미다.
- A3: `TextColor`는 현재 model의 `0x00BBGGRR` 값으로 직접 보존 가능하다.
- A4: `Version`/`Font` 외 fixture에 없는 수식 속성은 P0에 필수적이지 않다.
- A5: protocol v1에 additive capability/method를 넣어도 기존 parent client와 호환된다.

A1~A5는 사실로 간주하지 않는다. 각각 VG-01~05 또는 T-RPC-04/05가 통과해야 해당 구현 결정을 유지한다.

### Dependencies

- `pr/2219` HML parser/serializer 기반
- equation renderer/editor의 기존 동작
- rhwp-studio WASM build artifact 및 호스팅 배포 파이프라인
- ExamBank fixture 사용 허가/출처 기록

## 10. Rollout and Measurement

### Stage 0 — Before patch baseline

- 현재 호스팅에서 fixture screenshot, warning 목록, equation count 0을 보존한다.
- 현재 revision과 deployed version을 함께 기록한다.

### Stage 1 — Local implementation

- Slice A~C를 RED->GREEN으로 구현한다.
- HML parser/serializer 전체 회귀 테스트 실행.

### Stage 2 — Local browser E2E

- 실제 WASM build로 iframe flow를 검증한다.
- fixture 4개 수식 visible assertion과 export/reload를 통과한다.

### Stage 3 — Hosted canary

- 호스팅 빌드 revision을 노출하거나 배포 로그로 commit SHA를 증명한다.
- 동일 fixture로 anonymous HTTP origin embed E2E를 재실행한다.
- 기존 HWP/HWPX open/export smoke도 함께 실행한다.
- owner: PR author가 local evidence를 준비하고, 호스팅 deploy operator가 SHA 일치와 canary 실행을 승인한다.
- observation window: 배포 직후 exact E2E 1회와 이후 24시간 host error log의 HML import/export failure를 확인한다.
- rollback trigger: fixture 수식 4/4 미표시, export/reload 불일치 1건, 기존 HWP/HWPX smoke 실패 1건, uncaught runtime error 1건 중 하나라도 발생.
- log gate: deploy operator는 배포 전에 실제 log source와 재실행 가능한 query를 run vault에 기록한다. 최소 패턴은 `HML_*` blocker/export errors, `RPC_ERROR` for `exportHml`, uncaught browser/runtime error다. log source/query를 확인할 수 없으면 24시간 관찰을 통과로 처리하지 않는다.
- observation owner: 호스팅 deploy operator가 24시간 종료 시 query 결과와 rollback 여부를 기록한다.

### Stage 4 — ExamBank integration readiness

다음 조건을 모두 만족해야 ExamBank 통합 작업을 시작한다.

- `hml-export` capability 확인
- fixture 수식 4/4 표시
- edit/export/reload 통과
- HML blocker 0 또는 부모가 처리 가능한 명시적 blocker만 존재
- hosted SHA와 검증 SHA 일치

### Rollback

- 호스팅에서 수식 손실, export corruption, 기존 HWP/HWPX 회귀가 발견되면 이전 build로 되돌린다.
- ExamBank parent는 `hml-export` capability가 없으면 HML 저장 버튼을 비활성화한다.

## 11. Definition of Done

- [ ] repo-owned fixture와 provenance/checksum이 추가되고 CI가 외부 절대경로에 의존하지 않는다.
- [ ] VG-01~05 결과가 구현 run vault에 기록된다.
- [ ] fixture 기반 parser RED가 patch 전 실패하고 patch 후 통과한다.
- [ ] HML 수식 4개가 올바른 inline 위치에 렌더링된다.
- [ ] 기존 수식 편집 UI로 SCRIPT 수정이 가능하다.
- [ ] HML export/reparse에서 수정 수식과 미수정 수식이 보존된다.
- [ ] `exportHml` embed RPC와 `hml-export` capability가 구현된다.
- [ ] `getHmlSaveState`가 typed blocker를 반환하고 부모가 오류 문자열을 파싱하지 않는다.
- [ ] preflight blocker가 손실 가능성을 차단한다.
- [ ] Rust targeted/full relevant tests, rhwp-studio test/build가 통과한다.
- [ ] 실제 브라우저 load/edit/export/reload가 visible equation assertions와 함께 통과한다.
- [ ] 호스팅 revision과 검증 revision 일치가 증명된다.
- [ ] P1 미지원 서식 inventory가 후속 작업으로 기록된다.

## 12. Open Decisions

구현 전에 코드/fixture로 우선 답하고, 답이 없을 때만 결정한다.

1. fixture에 없는 HML equation attributes를 P0에서 어디까지 지원할지.
   - 권고: corpus inventory 결과 없이 추측 확장하지 않고, 발견 시 blocker로 명시.

## Critic

- 1차 독립 검토: `CRITIC.md`, Blocker 6 / Gap 1.
- 반영: unknown semantics 손실 차단, VG-01~05, required `getHmlSaveState`, exact transfer ownership, requirement-test matrix, repo-owned fixture/provenance/checksum, canary owner/window/rollback trigger.
- 최종 독립 검토: `CRITIC-FINAL.md`, Blocker 0 / Gap 0. Wire field 이름 polish도 반영 완료.
