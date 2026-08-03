---
kind: canonical
status: draft
canonical: mydocs/tech/wasm_agent_surface/self_description.md
last_verified: 2026-08-03
---

# WASM capabilities 자기서술 설계

> CLI 는 `rhwp capabilities` 로, MCP 는 `tools/list` 로 자기를 서술한다.
> **WASM 은 무엇으로 서술하는가.** 브라우저 안의 소비자 — 스튜디오, 확장, 임베드 호스트,
> 그리고 언젠가 브리지를 통해 붙을 에이전트 — 가 "이 모듈이 뭘 할 수 있나"를 아는 방법을
> 확정한다.
> 로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M24 첫 줄,
> [#3869](https://github.com/edwardkim/rhwp/issues/3869) W1·W2 에 대응한다.

이 문서의 모든 기술 주장에는 코드 경로(`파일:줄`) 또는 실제 명령 출력이 붙는다.
근거를 대지 못하는 항목은 **"확인되지 않음"** 으로 적었다.
축 전체의 지도는 [README.md](README.md) 에 있다.

---

## 0. 결론 먼저

세 문장으로 요약한다. 근거는 §1~§5 에 있다.

1. **WASM 표면에는 자기서술이 전혀 없다.** `src/wasm_api.rs` 7,621줄 안에
   `capabilities`·`schemaVersion` 이라는 문자열이 **한 번도** 등장하지 않는다.
2. **없는 이유는 기능 부족이 아니라 크레이트 경계다.** 자기서술 데이터
   (`capabilities_command_entries()`)는 **bin 크레이트**인 `src/main.rs:1463` 에 있고,
   WASM 이 컴파일되는 lib(`src/lib.rs`, 54줄)은 그것을 볼 수 없다.
3. **따라서 설계의 핵심은 "WASM 에 무엇을 새로 짤까"가 아니라
   "자기서술의 단일 출처를 lib 으로 내리고, 두 소비자(CLI·WASM)가 같은 출처를 읽게
   할까"다.** 이걸 안 하면 목록이 둘로 갈리고, 갈린 순간 문서도 둘이 된다.

---

## 1. 지금 있는 것 — 실측 전수

### 1.1 표면 규모

```
$ wc -l src/wasm_api.rs
7621 src/wasm_api.rs

$ grep -c "wasm_bindgen" src/wasm_api.rs
372

$ grep -oE "js_name\s*=\s*[A-Za-z0-9_]+" src/wasm_api.rs | sed 's/.*= *//' | sort -u | wc -l
360

$ grep -oE "pub fn [a-z_0-9]+" src/wasm_api.rs | sort -u | wc -l
367
```

구조체는 둘이다.

- `HwpDocument` — `src/wasm_api.rs:337`. 문서 하나를 감싸는 핸들.
  `impl Deref for HwpDocument` (341행) 로 `DocumentCore` 에 위임한다.
  `#[wasm_bindgen]` 이 붙은 `impl` 블록은 `504` 행부터.
- `HwpViewer` — `src/wasm_api.rs:7440`. 뷰 상태 전용(`7448`, `7522` 에 impl).

에러는 `impl From<HwpError> for JsValue` (`src/wasm_api.rs:45`) 하나로 JS 에 넘어간다.
즉 **실패는 JS 예외(throw)** 이지 봉투가 아니다. 이 사실이 §4.3 의 설계를 결정한다.

### 1.2 자기서술은 없다 — 실측

```
$ grep -cE "text_security|TextSecurity|inspect|extract_data|extract-data|digest|export_tables|schemaVersion|schema_version|untrustedContent" src/wasm_api.rs
0
```

**0이다.** 봉투 어휘(`schemaVersion`), 출처 표지(`untrustedContent`), 자기서술
(`capabilities`) 중 어느 것도 WASM 표면에 없다.

반면 CLI 는 그 전부를 싣는다. 실측 출력의 최상위 키:

```
$ rhwp capabilities | jq 'keys'
["batch","commands","exitCodes","formats","jsonContract","schemaVersion",
 "tool","untrustedContent","untrustedFields","version"]

$ rhwp capabilities | jq '{commands:(.commands|length), json:([.commands[]|select(.json)]|length)}'
{ "commands": 61, "json": 31 }

$ rhwp capabilities --mcp | jq '.tools | length'
39
```

### 1.3 에이전트 동사 — 있는 것 vs 없는 것

`js_name` 360개를 CLI 의 에이전트-가치 명령(#3608 §1-A~1-C)에 대응시킨 전수 표다.
**렌더링 API 는 이 표에 넣지 않는다**(§1.4).

#### 있다 — 이름은 다르지만 같은 일을 한다

| CLI 동사 | WASM 대응 | 코드 경로 | 차이 |
| --- | --- | --- | --- |
| `info` | `getDocumentInfo()` | `wasm_api.rs:1017` | 봉투 아님. 코어 문자열 그대로 |
| `export-structure` | `getStructure(mode)` | `wasm_api.rs:7542` | `mode` 어휘가 `auto\|outline\|clause` 로 **CLI 와 동일** |
| `search` | `searchText(...)` / `searchAllText(...)` | `wasm_api.rs:4846` / `4869` | 커서 기준 단건 + 전체. `include_cells` 인자가 CLI 엔 없다 |
| `fields` | `getFieldList()` | `wasm_api.rs:4542` | 반환은 `[{fieldId,fieldType,name,guide,command,value,location}]`(주석 4540행) |
| — | `getFieldValue(id)` / `getFieldValueByName(name)` | `wasm_api.rs:4550` / `4558` | CLI 엔 개별 조회가 없다 |
| `edit fill-fields` | `setFieldValue(id,v)` / `setFieldValueByName(...)` | `wasm_api.rs:4566` / `4575` | 한 건씩. CLI 는 `--data` 로 다건 |
| `edit replace-text` | `replaceOne(...)` / `replaceAll(...)` | `wasm_api.rs:4903` / `4916` | CLI `--occurrence k` 대응이 없다 |
| `edit insert-image` | `insertPicture` / `insertPictureEx` | `wasm_api.rs` (js_name 목록) | 좌표 인자 형태 미대조 |
| `convert` / `export-hwpx` | `exportHwp()` / `exportHwpx()` | `wasm_api.rs:5684` / `5700` | 바이트 반환. 파일 경로 개념 없음 |
| `... --verify` | `exportHwpVerify()` | `wasm_api.rs:5733` | **왕복 검증이 이미 WASM 에 있다** |
| `export-hml` | `exportHml()` | js_name 목록 | |
| `thumbnail` | `extractThumbnail(data)` | `wasm_api.rs:7595` | 자유 함수(구조체 밖) |
| `export-text` (부분) | `getTextRange(...)` / `getPageTextLayout(p)` | `wasm_api.rs:2345` / `1025` | **페이지별 텍스트 봉투는 없다.** 후자는 좌표까지 실린 레이아웃 |
| 암호 문서 | `openWithPassword` / `exportHwpWithPassword` | js_name 목록 | CLI `--password-stdin` 대응 |

#### 없다 — 코어는 있는데 노출이 없다

| CLI 동사 | WASM | 코어 위치(lib, wasm32 도달 가능) |
| --- | --- | --- |
| `digest` | **없음** | 봉투 조립이 `src/main.rs` |
| `extract-data` | **없음** | `document_core/queries/extract_data.rs:850` `pub fn extract_data(&self, kinds:&[DataKind]) -> Vec<DataItem>` (1,251줄) |
| `export-tables` (격자) | **없음** | `document_core/queries/table_extract.rs:292` `pub fn extract_tables(doc:&Document) -> Vec<TableGrid>` (304줄) |
| `table-to-csv` / `csv-to-table` | **없음** | `document_core/queries/table_csv.rs` (265줄) |
| `inspect hidden-text` | **없음** | `document_core/queries/hidden_text.rs:642` `pub fn detect_hidden_text(...)` (1,117줄) |
| `inspect injection` | **없음** | `document_core/queries/injection_scan.rs:1034` `pub fn scan_injection(...)` (1,467줄) |
| `inspect unicode` | **없음** | `document_core/text_security.rs` (1,345줄) |
| `edit redact` | **없음** | `document_core/queries/pii_scan.rs` (773줄) |
| `edit sanitize` | **없음** | 확인되지 않음 |
| `run`(계획 실행기) | **없음** | 확인되지 않음 |
| `ir-diff` | **없음** | 확인되지 않음 |
| `export-markdown` / `export-pdf` / `export-doclang` | **없음** | `doclang` 은 lib 모듈(`src/lib.rs:9`) |
| `extract-pages` | **없음** | 확인되지 않음 |
| `batch` | **없음** | 브라우저에 "파일 목록"이 없다 — 의미론 재정의 필요 |
| `capabilities` / `export-*-schema` / `export-provenance-map` | **없음** | §2.2·§2.4 |

혼동 주의: **`getValidationWarnings()`(`wasm_api.rs:5812`)는 `inspect` 가 아니다.**
스튜디오 타입 정의를 보면 `LinesegArrayEmpty | LinesegUncomputed | LinesegTextRunReflow`
세 종류의 **HWPX 비표준 감지 경고**다(`rhwp-studio/src/core/wasm-bridge.ts` 의
`ValidationReport`). 보안 판정과 무관하다.

### 1.4 렌더링 API 와 섞지 않는다는 뜻

`js_name` 360개 중 `render|canvas|paint|layer|overlay|cursor|hitTest|bbox|rect|zoom|dpi|
viewport|caret|line|preview|thumb` 를 포함하는 이름이 **55개**다(키워드 기반 근사 — 정확
분류가 아니다). 대표: `renderPageToCanvas`·`renderPageToCanvasFiltered`·
`renderPagePatchToCanvasFilteredWithProfile`·`getPageLayerTree`·`getCanvasKitReplayPlan`.

이들은 **브라우저 UI 의 것**이다. #3869 §4 가 명시적으로 제외한다. 자기서술이 이 둘을
한 목록에 담으면 소비자는 "rhwp WASM 이 할 수 있는 일" 360개를 보고 그중 무엇이
에이전트 동사인지 판별할 방법이 없다.

**따라서 자기서술은 처음부터 두 개의 축을 가진다** — `agent` 과 `render`. 하나의 목록에
`surface: "agent" | "render" | "ui"` 를 실어 소비자가 걸러 쓰게 한다(§4.2).

---

## 2. 왜 없는가 — 크레이트 경계

### 2.1 lib 과 bin 은 다른 크레이트다

```toml
# Cargo.toml:13-19
[lib]
crate-type = ["rlib", "cdylib"]

[[bin]]
name = "rhwp"
path = "src/main.rs"
```

`wasm-pack build --target web` 은 **lib**(cdylib)을 빌드한다
(`.github/workflows/deploy-pages.yml:60`). `src/main.rs` 는 그 산출물에 들어가지 않는다.

`src/lib.rs` 는 54줄이고, 공개 모듈은 다음과 같다(`src/lib.rs:7-25`).

```
capabilities_schema  diagnostics  doclang  document_core  emf  error  ir_schema
model  ole_chart  ooxml_chart  paint  parser  password_crypto  provenance
renderer  serializer  wasm_api  wmf
```

반면 `src/main.rs:7-9` 는 다음을 **자기 모듈로** 선언한다.

```rust
mod agent_profiles;
mod atomic_file;
mod mcp_serve;
```

즉 **에이전트 프로필 7종도, MCP 서버(`mcp_serve.rs`, 1,596줄)도 bin 전용**이다.
WASM 은 이들을 볼 수 없다.

### 2.2 자기서술 *데이터* 는 bin 에 있다

- 디스패치: `src/main.rs:277` — `Some("capabilities") => exit_with(show_capabilities(&args[2..]))`
- 명령 테이블: `src/main.rs:1463` — `fn capabilities_command_entries() -> Vec<serde_json::Value>`
- 출력: `src/main.rs:2283` — `println!("{}", provenance::marked(caps, "capabilities"))`
- did-you-mean 후보: `src/main.rs:2132` — `fn capabilities_command_names()`

`capabilities_command_entries()` 는 `serde_json::json!` 리터럴을 손으로 쌓는다.
`src/capabilities_schema.rs:9-13` 의 모듈 주석이 이 사실을 명시한다.

> `capabilities` 출력은 `serde_json::json!` 리터럴로 조립된 값이라 파생할 타입 자체가 없다.

**이 문장이 M24 첫 줄의 난이도를 정한다.** 타입이 없으므로 WASM 쪽에서 "같은 것"을
자동으로 만들 수 없다. 사람이 손으로 두 번 쓰면 드리프트가 시작된다.

### 2.3 그런데 *로직* 은 lib 에 있다

`document_core` 는 lib 모듈이다(`src/lib.rs:10`). 그 안 `queries/` 의 규모(`wc -l`):

```
   1251 extract_data.rs      1467 injection_scan.rs     1117 hidden_text.rs
    773 pii_scan.rs           738 search_query.rs        530 structure.rs
    304 table_extract.rs      265 table_csv.rs          1745 field_query.rs
```

이들은 대부분 순수 IR 조회다. `std::fs`/`std::path`/`std::io` 를 참조하는 파일은
`cursor_rect.rs`·`form_query.rs`·`grep.rs`·`rendering.rs`·`search_query.rs` 다섯이고,
`rendering.rs` 는 이미 `#[cfg(not(target_arch = "wasm32"))]` 로 네이티브 전용 경로를
가른다(`rendering.rs:1004`·`1014`·`1023`·`1044`·`1066`·`1073`, 그리고 `native-skia`
게이트 `1086` 이후).

**즉 `extract-data`·`inspect`·`export-tables` 를 WASM 에 노출하는 데 필요한 것은
새 로직이 아니라 `#[wasm_bindgen]` 한 줄과 봉투 조립이다.**

다만 `grep.rs`·`search_query.rs`·`form_query.rs` 의 wasm32 컴파일 가능성은
**확인되지 않음** — 실제로 빌드해 보지 않았다.

### 2.4 스키마는 이미 lib 에 있다 (그런데 노출은 안 된다)

`src/capabilities_schema.rs` 는 **lib 모듈**이고 613줄이다.

- `CAPABILITIES_SCHEMA_VERSION = "1.1"` — `src/capabilities_schema.rs:29`
- `schema`(명령 표면)와 `mcpSchema`(도구 매니페스트)를 **분리**한다는 결정이
  같은 파일 15~19행 주석에 기록돼 있다.
- `additionalProperties: true` 는 의도적 — "추가-전용 진화" 계약(같은 파일 51~54행).

`src/ir_schema.rs`·`src/provenance.rs`(518줄)도 lib 이다. 그런데

```
$ grep -c wasm_bindgen src/ir_schema.rs src/capabilities_schema.rs src/provenance.rs src/agent_profiles.rs
src/ir_schema.rs:0
src/capabilities_schema.rs:0
src/provenance.rs:0
src/agent_profiles.rs:0
```

**전부 0이다.** 스키마도 출처 표지도 lib 안에 있으면서 브라우저로 나가지 않는다.
이건 공백이라기보다 **아직 배선하지 않은 것**이다 — 가장 싼 조각이다.

---

## 3. 설계 — 세 후보와 판정

문제를 다시 정의한다. **브라우저 소비자가 런타임에 "이 모듈이 뭘 할 수 있나"를
알아야 한다.** 후보는 셋이다.

### 후보 A — 런타임 반사(reflection)

`Object.keys(wasmModule)` 로 export 된 함수 이름을 훑는다. 스튜디오는 이미 모듈 전체를
가져온다(`import * as wasmExports from '@wasm/rhwp.js'`,
`rhwp-studio/src/core/wasm-bridge.ts:2`).

- 장점: 코드 추가 0. 항상 실제 표면과 일치한다.
- 단점: **이름만 나온다.** 인자 스키마도, 반환 모양도, 어떤 것이 에이전트 동사이고
  어떤 것이 렌더링인지도 알 수 없다. 360개 이름을 받은 소비자는 아무것도 판단하지
  못한다. `capabilities` 가 CLI 에서 하는 일 — 요약·플래그·`recordFields` — 을
  전혀 대신하지 못한다.

### 후보 B — 빌드 시점 정적 스키마 동봉

`pkg/` 에 `capabilities.json` 을 함께 실어 소비자가 `fetch` 한다.

- 장점: WASM 바이너리를 안 키운다. 파싱이 싸다. CI 에서 CLI 출력으로 생성해
  **동등성을 생성 시점에 강제**할 수 있다.
- 단점: **파일이 분리되면 어긋난다.** 소비자가 옛 `capabilities.json` 과 새 `.wasm` 을
  섞어 쓰는 걸 막을 수단이 없다. 확장(`rhwp-chrome`)은 파일을 개별 복사하므로
  실제로 어긋날 수 있는 배포 형태다. 오프라인(`file:`)에서 `fetch` 가 막히는 경우도 있다
  — [zero_install_onboarding.md §4](zero_install_onboarding.md).

### 후보 C — 런타임 함수가 빌드 시점 상수를 돌려준다 (채택)

`lib` 안에 자기서술 데이터를 두고, `wasm_bindgen` 자유 함수 하나가 그것을 JSON 문자열로
돌려준다. 데이터는 컴파일 시점에 고정되고 배포 단위는 `.wasm` 하나다.

```rust
// 개념 — 실제 코드가 아니다
#[wasm_bindgen(js_name = capabilities)]
pub fn capabilities_json() -> String { /* lib::capabilities::wasm_envelope() */ }
```

- 장점: **버전 스큐가 구조적으로 불가능**하다. `.wasm` 을 바꾸면 자기서술도 같이 바뀐다.
- 단점: 바이너리가 커진다. 얼마나 커지는지는 **확인되지 않음** — 현재 `.wasm` 크기
  자체가 미실측이다(§8).

### 판정

| 기준 | A 반사 | B 정적 파일 | C 내장 함수 |
| --- | --- | --- | --- |
| 인자·반환 스키마를 준다 | ✗ | ○ | ○ |
| 에이전트/렌더 축 구분 | ✗ | ○ | ○ |
| 버전 스큐 불가 | ○ | ✗ | ○ |
| 오프라인 `file:` 동작 | ○ | △ | ○ |
| 바이너리 증가 | 0 | 0 | **미측정** |
| CLI 와 동등성 강제 지점 | 없음 | CI 생성기 | CI 생성기 + 계약 테스트 |

**C 를 채택한다.** 단, B 를 완전히 버리지 않는다 — **C 가 돌려주는 것과 동일한 JSON 을
`pkg/capabilities.json` 으로도 내보낸다.** 번들러·타입 생성기는 파일을 원하고,
런타임 소비자는 함수를 원한다. 둘이 같은 출처에서 나오면 어긋날 수 없다.

---

## 4. 표면 모양

### 4.1 진입점

에이전트 동사와 마찬가지로 **자유 함수**로 둔다. `HwpDocument` 인스턴스가 없어도
— 즉 **문서를 열기 전에** — 물을 수 있어야 하기 때문이다. 선례가 있다:
`extract_thumbnail` 은 `src/wasm_api.rs:7595` 에서 자유 함수이고, `version()` 은
`src/lib.rs:41` 에 있다.

- `capabilities()` → `String` (JSON) — CLI `rhwp capabilities` 대응
- `capabilitiesMcp()` → `String` — CLI `rhwp capabilities --mcp` 대응
- `capabilitiesSchema()` → `String` — `export-capabilities-schema` 대응.
  `src/capabilities_schema.rs` 가 이미 lib 에 있으므로 **배선만 하면 된다**(§2.4)

### 4.2 봉투 — 같아야 하는 것

CLI 실측 출력의 최상위 키 중 **아래는 그대로 유지한다.** 다르면 소비자가 두 벌의
파서를 써야 하고, 그 순간 문서가 둘로 갈린다(#3869 W2 가 말하는 실패 조건).

| 필드 | CLI 실측 값 | WASM 에서 |
| --- | --- | --- |
| `schemaVersion` | `"1.0"` | 동일 문자열 |
| `tool` | `"rhwp"` | 동일 |
| `version` | `"0.8.2"` | `env!("CARGO_PKG_VERSION")` — 이미 `version()` 이 쓰는 값 |
| `formats.read` | `["hwp5","hwpx","hwp3","hml"]` | 동일 |
| `formats.write` | `["hwp5","hwpx","hml","pdf","svg","png","txt","md","doclang"]` | **다르다** — §4.3 |
| `commands[].name` | 61개 | 부분집합. 이름은 **글자 그대로 같아야** 한다 |
| `commands[].category` | `query\|export\|edit\|batch\|diagnostic\|serve\|internal` | 동일 어휘 |
| `commands[].recordFields` | 봉투 필드 목록 | 동일 |
| `jsonContract.schemaPolicy` | 추가 허용, 변경·삭제는 범프 | 동일 |
| `untrustedContent` / `untrustedFields` | `false` / `[]` | **반드시 유지** — §4.4 |

여기에 **WASM 전용 필드 하나**를 더한다.

- `surface`: `"agent" | "render" | "ui"` — §1.4 의 축 구분.
  `additionalProperties: true` 정책(`src/capabilities_schema.rs:51-54`)이 이 추가를
  허용한다. **추가는 되고 변경은 안 된다.**

### 4.3 달라야 하는 것 — 그리고 왜

동등성은 **복제가 아니다.** 브라우저에 존재하지 않는 개념을 억지로 실으면 그게 더 큰
거짓말이 된다. 아래는 **의도적으로 다르게** 두고, 그 이유를 봉투 안에 적는다.

| 필드 | 왜 다른가 |
| --- | --- |
| `exitCodes` (0~4) | **브라우저에 종료 코드가 없다.** WASM 실패는 JS 예외다 (`impl From<HwpError> for JsValue`, `src/wasm_api.rs:45`). 대신 `errorCodes` 로 같은 5개 부류를 이름으로 싣고, `exitCodeMapping` 으로 CLI 대응을 명시한다 |
| `batch` | `stdin` 파일 목록 개념이 없다. 브라우저 대응물(File 배열)은 **의미론이 달라** 같은 이름을 쓰면 안 된다. 초기엔 `batch: null` 로 명시 부재를 싣는다 |
| `commands[].flags` | 플래그가 아니라 **함수 인자**다. `params` 로 이름을 바꾸고 JSON Schema 로 적는다 |
| `formats.write` 의 `pdf`·`png` | `export-pdf`·`export-png` 는 WASM 에 없다(§1.3). **없는 걸 있다고 적으면 안 된다.** `svg` 는 `renderPageSvg` 로 있다 |
| `mcp.invocation.transport` | CLI 는 `"cli"`. 브라우저는 `"postMessage"` 계열 — [browser_bridge.md](browser_bridge.md) 가 정한다 |
| 파일 경로 인자 | `path` 가 없다. 입력은 바이트(`from_bytes`, `src/wasm_api.rs:358`)다. `inputSchema` 에서 `path: string` 을 `data: Uint8Array` 로 바꾼다 |

**차이는 숨기지 말고 봉투 안에 적는다.** CLI `batch.authentication` 이
`"지원하지 않음 — ... 계약은 아직 정의되지 않았다"` 라고 한국어 문장으로 부재를
설명하는 선례가 이미 있다(실측 출력). 같은 방식을 쓴다.

### 4.4 출처 표지는 반드시 간다

CLI `capabilities` 봉투에도 `untrustedContent: false`, `untrustedFields: []` 가 실린다
(실측). 정책이 명시돼 있다.

> 표지는 항상 실린다 — 문서를 열지 않는 명령의 봉투도 `untrustedContent:false` 를 명시한다

**WASM 에서 이게 더 중요하다.** 브라우저에서는 문서 내용과 UI 상태가 같은 JS 힙에
섞이고, 문서에서 온 문자열이 그대로 DOM 이나 에이전트 컨텍스트로 흘러갈 수 있다.
`src/provenance.rs`(518줄)는 이미 lib 이므로 배선만 하면 된다.

위협 모델의 전제 — 문서는 공격자가 만들 수 있다 — 는
[../agent_security/threat_model.md](../agent_security/threat_model.md) 를 그대로 따른다.
브라우저가 추가하는 **새 경계(origin)** 는 [browser_bridge.md §5](browser_bridge.md) 가 다룬다.

---

## 5. 동등성을 유지하는 기제

동등성은 선언으로 유지되지 않는다. **깨지면 CI 가 빨개져야** 유지된다.

### 5.1 단일 출처를 lib 으로 내린다

지금은 `capabilities_command_entries()`(bin)가 유일 출처다. 이걸
`src/capabilities.rs`(lib)로 옮기고, bin 과 wasm 이 **같은 함수를 읽는다.**

```
        src/capabilities.rs  (lib, 단일 출처)
                 │
      ┌──────────┴──────────┐
  src/main.rs           src/wasm_api.rs
  (CLI 출력 + 필터)      (WASM 필터 + 봉투)
```

`surface` 축(§4.2)이 필터의 근거가 된다. WASM 은 `surface == "agent"` 이고 실제로
`js_name` 이 존재하는 항목만 싣는다.

**이 이동은 동작 무변경 리팩터여야 한다.** `rhwp capabilities` 출력이 바이트 단위로
같아야 한다 — 그 자체가 첫 계약 테스트다.

### 5.2 기존 드리프트 가드를 확장한다

이미 하나가 돈다.

- `scripts/frontend-wasm-bindings.test.mjs` — `#[wasm_bindgen(js_name = X)]` 로 선언된
  모든 이름이 `pkg/rhwp.d.ts` 에 나타나는지 단언한다.
  CI 연결: `.github/workflows/ci.yml:951`.

**이건 *이름* 가드이지 *의미* 가드가 아니다.** 필요한 것은 셋이다.

| 가드 | 무엇을 막나 | 형태 |
| --- | --- | --- |
| G1 이름 존재 | 자기서술이 없는 함수를 광고 | `capabilities()` 의 모든 항목이 `pkg/rhwp.d.ts` 에 있다 |
| G2 역방향 | 광고되지 않은 에이전트 동사 | `surface=="agent"` 로 분류된 `js_name` 중 자기서술에 빠진 것 = 0 |
| G3 봉투 동등 | CLI 와 WASM 의 봉투 모양 분기 | 같은 문서에 대해 `recordFields` 집합이 일치 |

G1 은 기존 스크립트의 대칭 확장이다. G3 은 #3869 W2 가 요구하는 계약 테스트 그 자체다.

### 5.3 계약 테스트의 모양

#3869 수용 기준은 "WASM 봉투와 CLI `--json` 봉투가 **동일**함을 계약 테스트가 고정"이다.
현실적인 형태는 다음 셋이다.

1. **Rust 단위 테스트** — 같은 샘플 바이트로 lib 코어를 두 번 호출해 봉투를 비교한다.
   `wasm32` 타깃이 아니어도 lib 함수는 네이티브에서 돌므로 **CI 에서 가장 싸다.**
2. **`wasm-bindgen-test`** — 실제 wasm32 에서 도는지 확인. 의존성은 이미 있다
   (`Cargo.toml:94` `wasm-bindgen-test = "0.3"`).
3. **Node 계약 테스트** — `scripts/` 의 기존 패턴(`node --test`)을 따라
   `capabilities()` JSON 과 `rhwp capabilities` 출력을 대조.

`.wasm` 산출물이 필요한 2·3 은 CI 에서만 돌 수 있다. 로컬 개발자는 1 로 대부분을 잡는다.

---

## 6. 조각 분해 (#3869 W1·W2 대응)

착수 순서다. 각 조각은 이전 조각 없이는 검증할 수 없다.

| # | 조각 | 산출 | 검증 |
| --- | --- | --- | --- |
| S1 | `capabilities` 데이터를 lib(`src/capabilities.rs`)로 이동 | 동작 무변경 | `rhwp capabilities` 출력 바이트 동일 |
| S2 | 항목마다 `surface` 축 부여 | `agent`/`render`/`ui` 분류 | 분류 누락 = 0 인 테스트 |
| S3 | `capabilities()` / `capabilitiesMcp()` / `capabilitiesSchema()` wasm 노출 | js_name 3개 | `pkg/rhwp.d.ts` 에 등장(G1) |
| S4 | 봉투 조립 헬퍼를 lib 으로 (`schemaVersion`·`untrustedContent` 포함) | 공용 봉투 | CLI 출력 무변경 |
| S5 | 없는 동사 노출 — `digest`·`extract-data`·`export-tables`·`inspect` 3종 | 새 js_name | G2 + G3 |
| S6 | `pkg/capabilities.json` 동시 산출 | 빌드 산출물 | 함수 반환값과 바이트 동일 |

**S5 를 S1~S4 보다 먼저 하고 싶은 유혹을 경계한다.** 봉투 없이 동사를 먼저 노출하면
스튜디오가 그 모양에 붙고, 나중에 봉투를 씌울 때 호환을 깨야 한다.

---

## 7. 하지 않는 것

- **360개 전부를 자기서술하지 않는다.** `surface=="agent"` 만 싣는다. 렌더 API 의
  자기서술은 별개 문제이며 이 문서의 범위가 아니다.
- **JSON Schema 를 손으로 두 번 쓰지 않는다.** `src/capabilities_schema.rs` 가 이미
  단일 출처다(§2.4).
- **CLI 봉투를 WASM 에 맞추어 바꾸지 않는다.** 방향은 한쪽이다 — WASM 이 CLI 를 따른다.
  CLI 봉투는 이미 외부 소비자(바인딩·MCP)가 쓰고 있다.
- **`exitCodes` 를 억지로 옮기지 않는다.** §4.3.

---

## 8. 확인되지 않음

1. **자기서술 내장이 `.wasm` 을 얼마나 키우는가** — 현재 `.wasm` 크기 자체가 미실측이다
   (`.gitignore:12` 가 `*.wasm` 제외, 이 작업 트리에 산출물 없음).
   [zero_install_onboarding.md §2](zero_install_onboarding.md) 와 같은 공백이다.
2. **`document_core/queries/` 전 모듈의 wasm32 컴파일 가능성** — `grep.rs`·
   `search_query.rs`·`form_query.rs` 가 `std::fs`/`std::io` 를 참조한다. 실제 빌드 미확인.
3. **`edit sanitize`·`run`·`ir-diff`·`extract-pages` 의 코어 위치** — lib 인지 bin 인지
   대조하지 않았다.
4. **`insertPicture` 인자 형태가 CLI `edit insert-image` 와 대응하는지** — 시그니처를
   줄 단위로 대조하지 않았다.
5. **WASM 호출 성능** — 이 문서에 성능 주장이 하나도 없는 이유다.

---

## 9. 관련 문서

- [README.md](README.md) — 이 축의 지도
- [browser_bridge.md](browser_bridge.md) — 자기서술을 **누가 어떻게 물어보는지**
- [zero_install_onboarding.md](zero_install_onboarding.md) — 자기서술이 실려 나가는 배포 경로
- [../agent_security/threat_model.md](../agent_security/threat_model.md) — 출처 표지의 근거
- [../agent_security/consumer_guide.md](../agent_security/consumer_guide.md) — 소비자 책임 경계
- [../bindings_foundation.md](../bindings_foundation.md) — 외부 바인딩의 표면 판단(M18~M20)
- [../wasm_pack_version_policy.md](../wasm_pack_version_policy.md) — 툴체인 고정
- 이슈 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M24 ·
  [#3869](https://github.com/edwardkim/rhwp/issues/3869) W1·W2 ·
  [#3787](https://github.com/edwardkim/rhwp/issues/3787)
