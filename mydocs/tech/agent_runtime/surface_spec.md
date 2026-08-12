---
kind: canonical
status: active
canonical: mydocs/tech/agent_runtime/surface_spec.md
last_verified: 2026-08-03
---

# 에이전트 런타임 표면 명세 — 실행 파일 없이 부르는 rhwp

> **v0.8.4 현행성 주의:** Python·Node 바인딩을 근거로 든 비교와 코드 인용은
> 철회 전 설계 이력이다. 두 공식 바인딩은 #4655에서 제거됐으며 현재 지원 표면이 아니다.

> rhwp의 CLI·MCP 진입로는 **rhwp 실행 파일을 먼저 구해야
> 한다**는 관문을 공유한다. 이 문서는 그 관문 없이 부를 수 있는 **에이전트 전용
> WASM 표면**의 설계를 확정한다. 로드맵 [#3869](https://github.com/edwardkim/rhwp/issues/3869).
> 봉투 동등성은 [envelope_parity.md](envelope_parity.md), 축 지도는 [README.md](README.md).

모든 기술 주장에 코드 경로(`파일:줄`) 또는 실제 명령 출력을 붙였다. 근거를 대지 못하는
항목은 **"확인되지 않음"** 으로 적었다. 측정하지 않은 성능은 측정하지 않았다고 적었다 —
추측을 사실처럼 적은 설계 문서는 반년 뒤 거짓말이 된다.

실측 환경: Windows 11, `target/release/rhwp.exe` v0.8.2 (2026-08-03 빌드), 저장소
`samples/`. 이 문서의 봉투 인용은 전부 그 바이너리를 직접 돌려 받은 것이다.

---

## 0. 정하는 것과 정하지 않는 것

**정한다**: `--json` 명령 **31개** 중 무엇을 넣고 무엇을 빼는가와 그 근거 / 넣은
동사의 입력·반환 봉투·실패 표현·CLI 대응 / 경로가 없는 세계에서 `source`·`-o`·
상대 경로·진단 채널이 어떻게 되는가 / 문서를 객체로 들고 있을 때 무엇이 단순해지고
무엇이 위험해지는가.

**정하지 않는다**: 코드를 짜지 않는다(아래 시그니처는 계약의 *모양*을 보이기 위한
의사 표기다) / 기존 표면을 대체하지 않는다 — CLI·`mcp-serve`는
그대로 남는다 / 렌더링 WASM 표면(`renderPageToCanvas`
계열, rhwp-studio 의 계약)을 건드리지 않는다 / 성능을 약속하지 않는다(§7).

---

## 1. 문제 — 관문은 언제나 실행 파일이다

### 1.1 두 진입로의 공통 전제

| 진입로 | 전제 | 근거 |
| --- | --- | --- |
| CLI | `rhwp` 실행 파일이 `PATH` 에 있다 | `Cargo.toml:17-19` `[[bin]] name = "rhwp"` |
| MCP | 호스트가 `rhwp mcp-serve` 를 자식으로 띄운다 | `src/mcp_serve.rs` 전체가 stdio JSON-RPC 서버 |

둘 다 **바이너리를 구한 뒤에야** 시작된다. 샌드박스 안 에이전트 — 임의 실행 파일
반입이 막혀 있거나 프로세스 생성 자체가 없는 런타임 — 에게는 이 관문이 곧 벽이다.

### 1.2 WASM 표면은 이미 있다. 그런데 에이전트용이 아니다

rhwp 는 이미 WASM 으로 컴파일된다(`Cargo.toml:13-15` `crate-type = ["rlib","cdylib"]`,
`wasm-pack build --target web`). 규모 실측: `src/wasm_api.rs` **7,621줄**,
`wasm_bindgen` **372회**, 명시 `js_name` export **364개**(`get*` 121, `set*` 42,
`render*` 14). 이름 분포가 성격을 말한다 — `renderPageToCanvasFilteredWithProfile`
(`:721`), `getCanvasKitReplayPlan`(`:862`), `beginDeferredPagination`(`:1327`) 는
**편집기 프런트엔드의 요구**다. 에이전트가 묻는 질문("누름틀이 몇 개인가", "'계약' 이
어디 있는가")에 답하는 동사는 이 표면의 목표가 아니었다.

### 1.3 결정적 실측 — 봉투가 없다

에이전트 계약의 핵심은 **봉투**다. `schemaVersion`·`source`·`untrustedFields` 가
붙어야 소비자가 값의 출처를 판별한다([envelope_provenance.md](../envelope_provenance.md)).

```
$ grep -c schemaVersion src/wasm_api.rs   →  0
$ grep -c schemaVersion src/main.rs       →  112
```

**WASM 표면에는 봉투가 하나도 없다.** 이유는 구조적이다.

- 봉투 생성기(`structure_json_value` `src/main.rs:6793`, `tables_json_value` `:6810`,
  `fields_json_value` `:6826`, `search_json_value` `:6893`, `info_json_value` `:6993`,
  `extract_data_json_value` `:9956`)가 **전부 `src/main.rs` 안에 있다.**
- `src/main.rs` 는 `[[bin]]` 이고(`Cargo.toml:17-19`) WASM 빌드는 `--lib` 만 간다
  (`.github/workflows/ci.yml:816` `cargo check --target wasm32-unknown-unknown --lib`).
- 즉 **`src/wasm_api.rs` 는 봉투 생성기를 볼 수 없다.**

같은 데이터가 두 갈래로 나가는 것이 코드로 보인다.

```rust
// CLI: src/main.rs:3406,6793 — build_structure → 봉투 → provenance::marked
let st = build_structure(doc.document(), mode);
let envelope = structure_json_value(file_path, &st);   // schemaVersion·source·표지

// WASM: src/wasm_api.rs:7541 → src/document_core/queries/structure.rs:403
pub fn get_structure_native(&self, mode: &str) -> Result<String, HwpError> {
    let st = build_structure(&self.document, mode);
    serde_json::to_string(&st)                          // 알맹이만, 봉투 없음
}
```

**알맹이(`build_structure`)는 공유하고 봉투만 갈린다.** 이 축이 메울 공백의 정확한
모양이 이것이다. 다행히 표지를 붙이는 `provenance::marked` 는 `Value → Value` 순수
함수이고(`src/provenance.rs:468`) `src/lib.rs:20` 으로 라이브러리에 있다 — **WASM 에서
부를 수 있다.** 옮겨야 하는 것은 봉투의 *모양*뿐이다.

### 1.4 철회 전 TypeScript 분기 실측

이 절은 #4655 이전 Node 바인딩의 historical evidence다. 당시 브라우저 어댑터
(`bindings/node/src/browser.ts`, 249줄)가 WASM 위에서 봉투를 **손으로 조립한다.**

```ts
// bindings/node/src/browser.ts:143-150
async info(source) {
  return withDocument(source, (doc) =>
    new Envelope({ schemaVersion: '1.0', source: '(bytes)', pageCount: doc.pageCount() }));
}
```

CLI `info --json` 실측(`samples/hwpers_test4_complex_table.hwp`)과 나란히 놓으면:

```jsonc
{"fonts":["맑은 고딕"],"format":"hwp5","pageCount":1,"paraCount":19,
 "schemaVersion":"1.0","sections":1,"sizeBytes":6656,
 "source":"samples/hwpers_test4_complex_table.hwp","title":"복잡한 표 테스트",
 "untrustedContent":true,"untrustedFields":["title","fonts[]"],"version":"5.0.3.4"}
```

**필드 10개 + 표지 2개 대 필드 3개.** `untrustedContent` 가 없으므로 소비자는 이
봉투가 문서 파생 값을 담았는지 판별할 수 없다 — "표지는 항상 실린다"는 정책
(`capabilities.jsonContract.provenance.policy` 실측)이 여기서 끊긴다.
`exportText`(`browser.ts:152-167`)는 더 갈린다: `source` 자체가 없고 `truncated`·
`omittedCount` 도 없다 — CLI 는 `--max-chars 100` 에서
`{"truncated":true,"omittedCount":33585,…}` 로 절단을 드러내는데(실측,
`samples/2022년 국립국어원 업무계획.hwp`) 브라우저 경로에는 그 필드가 없다.

그리고 어댑터가 기대하는 WASM 함수는 **전부 옵셔널이다** — `structureJson?()`
`tablesJson?()` `fieldsJson?()` `searchJson?()`(`browser.ts:95-98`). 없는 함수를
옵셔널로 선언해 둔 것이다. `bindings/README.md` 가 바인딩에 못박은 원칙
("repackaging, not a new surface … doing it twice guarantees the two answers
eventually disagree")이 브라우저 경로에서만 이미 깨져 있고, DESIGN.md 도 그 공백을
스스로 적었다(`bindings/node/docs/DESIGN.md:366`: "WASM 경로가 같은 인터페이스를
계속 만족하는지는 **아직 타입 수준에서만 보장된다**").

**이 축은 새 위험을 만드는 것이 아니라 이미 벌어진 분기를 닫는다.**

---

## 2. 판단 기준 — 31개를 다 노출하지 않는다

"전부 넣는다"는 설계가 아니다. 31개는 **CLI 라는 실행 맥락**에서 자란 목록이고, 그
맥락의 절반(프로세스·파일시스템·병렬·stderr)이 WASM 에 없다. 맥락이 사라진 명령을
이름만 옮기면 부르는 쪽은 되는 줄 알고 부르다 런타임에 깨진다 — `browser.ts` 가
옵셔널 메서드로 피해 간 바로 그 함정이다.

세 관문을 **모두** 통과하는 명령만 넣는다.

### 관문 ① — 바이트만으로 성립하는가

동사의 **본질**이 파일시스템에 있으면 뺀다. 입력 파일을 바이트로 바꾸는 것은
문제가 아니다 — CLI 자신이 이미 그렇게 한다:

```rust
// src/main.rs:107-113 — CLI 의 모든 문서 로드가 지나는 곳
fn load_document(data: &[u8]) -> Result<rhwp::wasm_api::HwpDocument, LoadError> { … }
//               ^^^^^^^^^^^ 경로가 아니라 바이트
```

CLI 는 파일을 읽어 **`wasm_api::HwpDocument::from_bytes` 로 넘긴다**
(`src/wasm_api.rs:358`). `src/main.rs` 안에서 그 타입을 쓰는 곳이 42군데다. 즉
**읽기 축은 이미 바이트 기반**이고 경로는 봉투의 `source` 문자열로만 남는다.

문제는 **쓰기**다. `-o` 로 디렉터리에 N개를 흩뿌리거나(`export-svg`·
`export-markdown`), 자산 디렉터리 규약을 갖거나(`export-doclang --assets-dir`,
`build-from-ingest --media-dir`), 별도 입력 파일을 경로로 받는(`csv-to-table --csv`)
명령은 WASM 에서 **규약 자체를 새로 발명해야** 성립한다.

### 관문 ② — 에이전트 동사인가, 엔진 개발자 동사인가

- **에이전트 질문**: "이 문서에 뭐가 있나", "어디에 있나", "이걸 바꾸면 뭐가 되나".
- **엔진 개발자 질문**: "우리 조판기가 이 줄을 어디에 놓았나", "왕복 뒤 IR 이 몇
  항목 달라졌나".

두 번째의 답은 rhwp 를 **고치는** 사람에게 값지고, rhwp 를 **쓰는** 에이전트의
컨텍스트에서는 소음이다. `capabilities` 의 `category: "diagnostic"` 이 이미 이 선을
긋고 있다. 다만 카테고리는 CLI 사용자를 위한 분류지 WASM 표면의 판정이 아니므로
셋을 각각 따진다(§3.5).

### 관문 ③ — 경계를 넘길 값이 컨텍스트에 들어갈 만한가

WASM 경계를 넘는 값은 **복사된다.** 그보다 중요한 것은 **그 값이 결국 어디로
가는가**다. 실측 — 393쪽 `samples/2025 행정업무운영 편람(최종).hwp`(원본 10,687,488 B):

| 명령 | `--json` stdout | digest 대비 |
| --- | ---: | ---: |
| `digest --json` | 1,375 B | 1× |
| `info --json` | 636 B | 0.46× |
| `fields --json` | 47,738 B | 35× |
| `export-structure --json` | 284,892 B | 207× |
| `export-text --json` | 645,108 B | **469×** |

469배를 기본값으로 컨텍스트에 밀어 넣는 표면은 잘못 설계된 것이다. `digest` 가
`nextStep` 을 실어 다음 호출을 안내하는 이유가 이것이다(실측:
`"nextStep":"더 읽으려면 export-text --json -p <쪽>, 찾으려면 search --json"`).

다만 "크다"가 곧 "빼라"는 아니다 — 소비자가 **그 바이트 자체를 원해서** 부르면
(문서 저장, 썸네일 표시) 통과시키고, 원하는 것이 바이트가 아니라 **그 안의 정보**
이면(PDF 를 만들어 다시 읽게 하기) 뺀다.

---

## 3. 31개 전수 판정

`rhwp capabilities` 의 `commands[]` 에서 `json: true` 인 항목을 전수 확인했다
(2026-08-03, v0.8.2). 전체 61개 중 **31개**.

**결론: 18개를 넣고 13개를 뺀다.** 18개는 WASM 동사 **17개**로 모인다
(`convert` 와 `export-hwpx` 가 하나의 `save` 로 합쳐진다).

### 3.1 `query` — 8개 중 7개

| 명령 | 판정 | 근거 |
| --- | :---: | --- |
| `info` | **넣음** | 관문 3개 통과. 636 B. 문서 식별의 최소 동사 |
| `digest` | **넣음** | 진입 동사. 1,375 B 로 393쪽을 요약. `nextStep` 이 다음 호출을 지정 |
| `search` | **넣음** | 주소(구역·문단·페이지·문자 오프셋)가 붙은 결과. 절단 규약 내장 |
| `extract-data` | **넣음** | 날짜·금액·수량 구조화. 같은 주소 체계 |
| `fields` | **넣음** | 누름틀. `textSecurity` 표지를 싣는 유일한 조회 축 |
| `inspect` | **넣음** | 3 하위축. 문서를 **읽기 전에** 부르는 동사라 샌드박스에서 가치가 가장 크다 |
| `capabilities` | **넣음(재정의)** | ↓ |
| `export-provenance-map` | **뺌** | ↓ |

**`capabilities` 재정의.** CLI 판은 CLI 명령 61개를 서술한다. 그대로 내면 **없는
동사를 광고**하게 된다 — `browser.ts` 옵셔널 메서드와 같은 실패다. 그렇다고 뺄 수도
없다: 이름 환각(F1)을 막는 단일 출처이고([weak_agent_proofing.md](../weak_agent_proofing.md)
§4 P1), 철회 전 Node 바인딩의 패리티 가드가 실제 결함 7건을 잡은 근거가 그 자기서술이었다
(`bindings/node/docs/DESIGN.md:224` D11). 따라서 **`surface: "wasm"` 로 표시된, 이
빌드가 실제로 가진 동사만 담은** capabilities 를 낸다. 생성 출처는 CLI 와 같은
표여야 한다([envelope_parity.md](envelope_parity.md) §6).

**`export-provenance-map` 제외.** 봉투가 이미 자기 표지를 싣고(`src/provenance.rs:468`)
지도는 그 표지의 *설명*이며 **버전당 고정**이다 — 실측 11,885 B 가 문서와 무관하게
매번 같다. 런타임 호출이 아니라 패키지 동봉 **정적 자산**이 맞다. 같은 근거가 아래
두 스키마 명령에도 적용된다.

### 3.2 `export` — 16개 중 8개

| 명령 | 판정 | 근거 |
| --- | :---: | --- |
| `export-text` | **넣음** | 본문. `--max-chars` 절단 규약이 관문 ③ 의 안전판 |
| `export-structure` | **넣음** | 개요/조문 트리. 알맹이를 이미 공유(§1.3) |
| `export-tables` | **넣음** | 병합·중첩 보존 격자. 표는 행정문서의 데이터 그 자체 |
| `export-svg` | **넣음(1쪽 단위)** | ↓ |
| `thumbnail` | **넣음** | ↓ |
| `extract-pages` | **넣음(편집 동사로)** | 바이트→바이트, 외부 입력 없음. 발췌 제출은 실제 업무 |
| `convert` | **넣음(`save` 흡수)** | 바이트→바이트. `--verify` 판정을 봉투에 실어 온다 |
| `export-hwpx` | **넣음(`save` 흡수)** | 〃 |
| `export-pdf` | **뺌** | ↓ |
| `export-markdown` | **뺌** | `recordFields` 에 `outputDir`·`imageCount` — 텍스트 + 이미지 N개를 디렉터리에 푸는 명령(관문 ①). 문자열이 필요하면 `text()`+`structure()` 조립이 손실 통제에 낫다 |
| `export-doclang` | **뺌** | `--assets-dir` 자산 그래프(`assetsDir`·`assetCount`). 바이트 맵 규약을 새로 발명해야 성립 — 관문 ① |
| `export-hml` | **뺌** | 입력이 `.hml` 원본 한정(`cli_commands.md` "입력은 `.hml`만 받는다"). 첫 판에 넣을 만큼 넓지 않다. 필요해지면 `save("hml")` 로 흡수 |
| `export-ir-schema` | **뺌** | 정적 자산. 실측 44,119 B, 문서 무관. **코드 생성 입력**이지 에이전트 동사가 아니다 |
| `export-capabilities-schema` | **뺌** | 〃 |
| `table-to-csv` | **뺌** | `tables()` 가 이미 격자를 온전히 준다. CSV 는 그 격자의 *표현 형식*이고 구분자·BOM·인코딩은 전부 소비 환경 문제다(CLI 에 `--bom` 이 있는 이유). 넘길 것은 격자다 |
| `build-from-ingest` | **뺌** | `--media-dir` 디렉터리 규약(관문 ①). 그리고 이것은 문서를 *읽는* 동사가 아니라 *조립하는* 파이프라인 도구다 |

**`export-svg` 를 1쪽 단위로만.** CLI 봉투는 **파일 경로 매니페스트**다. 실측:

```jsonc
{"format":"svg","outputDir":"…/svgout","pageCount":1,"renderedCount":1,
 "pages":[{"page":0,"bytes":24353,"path":"…/hwpers_test4_complex_table.svg"}],
 "schemaVersion":"1.0","source":"…","untrustedContent":false,"untrustedFields":[]}
```

`path` 는 WASM 에 없지만 대응물이 이미 있다 — `renderPageSvg(page) -> String`
(`src/wasm_api.rs:623`). **경로 대신 본문**을 돌려주면 된다. 전 쪽 일괄은 393쪽 ×
24 KB 규모라 관문 ③ 에서 막는다. 어느 쪽을 볼지 아는 호출자가 루프를 짜야 한다
(`changedPages` 가 존재하는 이유 — [weak_agent_proofing.md](../weak_agent_proofing.md) §4 P3).

**`thumbnail` 은 새 기능이 아니라 패리티 수복이다.** WASM 에 이미 독립 함수가 있고
(`extractThumbnail(data)`, `src/wasm_api.rs:7594`) 반환 모양만 갈라져 있다 — WASM 은
`{"format","base64","dataUri","width","height"}`(없으면 `null`), CLI 는 실측
`{"bytes":17256,"format":"png","height":1024,"mime":"image/png","output":"output/…_thumb.png","schemaVersion":"1.0","source":"…","untrustedContent":false,"untrustedFields":[],"width":724}`.
`schemaVersion` 도 표지도 없고 "없음"의 표현도 다르다(`null` 대 exit code). 게다가
CLI 에 `--base64`·`--data-uri` 가 이미 있다 — **바이트를 문자열로 넘기는 규약의
선례가 CLI 쪽에 존재한다.** 넣지 않으면 이 갈라짐이 그대로 남는다.

**`export-pdf` 제외.** 두 관문에 걸린다. ① `--font-path` 로 시스템 폰트 디렉터리를
탐색한다(한컴 전용 폰트 폴백) — 브라우저에는 그 탐색 대상이 없고, 대안인 폰트 바이트
주입은 새 규약이다. ③ 산출은 한 덩이 바이너리인데 에이전트는 보통 그 바이트를 *다시
읽고* 싶어 한다 — 그러면 `export-text`·`renderPage` 로 곧장 가는 편이 언제나 싸다.

### 3.3 `edit` — 3개 중 2개

| 명령 | 판정 | 근거 |
| --- | :---: | --- |
| `edit` | **넣음** | 6 하위축 ↓ |
| `run` | **넣음(입출력 제거판)** | ↓ |
| `csv-to-table` | **뺌** | `--csv <경로>` 로 **별도 입력 파일**을 받는다(관문 ①). 격자를 바이트로 받는 규약을 새로 만들 바에는 `edit set-cell` 반복 또는 계획으로 표현하는 편이 단일 출처를 지킨다 |

**`edit` 하위축 실측(2026-08-03):**

```
$ rhwp edit
사용법: rhwp edit <fill-fields|replace-text|set-cell|insert-image|redact|sanitize> …
```

여섯이다. (`agent_security/README.md` 는 2026-08-02 기준 "3종"으로 적고 있다 — 그
뒤 늘었다. 현재 동작은 언제나 바이너리로 재확인한다.) `insert-image` 의
`--image <경로>` 는 **이미지 바이트**로 바뀐다 — 입력 축이므로 읽기와 같은 변환이다.
나머지 다섯은 문자열·좌표 인자만 쓴다.

**`run` 을 "입출력 제거판"으로.** 계획 실행기는 파일시스템 쓰기가 본질처럼 보이지만
코드는 다르다:

```rust
// src/main.rs:13341
fn run_plan_engine(plan: &serde_json::Value) -> (serde_json::Value, i32)
```

`plan` 을 받아 `(봉투, exit code)` 를 돌려주는 **거의 순수한 함수**다. 파일시스템에
닿는 곳은 `fs::read(input)`(`:13379`)와 `fs::write(output, &out_bytes)`(`:13774`)
둘뿐이다. 그 둘을 걷어내면 **"계획 → 저널 + 바뀐 문서"** 라는 순수 변환이 남는다.

그래서 WASM 판은 **`input`/`output` 필드를 계획에서 제거하고** 입력은 이미 열린
문서, 산출은 `save()` 로 꺼내는 바이트로 한다. 이것이
[#3719](https://github.com/edwardkim/rhwp/issues/3719) 불변식 1(의도의 단일 출처는
계획서)을 지키는 유일한 방식이다 — 계획에 경로를 남기면 그 경로가 어디로 해석되는지
아무도 답할 수 없다(§4.3).

> 이 결정은 [agent_boundary_contract.md](../agent_boundary_contract.md) S5("문서
> 내용은 어떤 파일 경로에도 성분으로 들어가지 않는다")와 정면으로 맞는다. WASM 에는
> **경로 자체가 없으므로** S5 의 위협이 구조적으로 소멸한다. 이 축이 보안을 나쁘게
> 만들지 않는 몇 안 되는 지점이다.

### 3.4 `batch` — 뺌

`batch` 는 **프로세스 병렬이 본질**이다. `capabilities` 의 `batch` 절 실측이 그것을
말한다: `input` 은 "stdin, 한 줄당 파일 경로 하나", `flags` 에 `--threads`,
`exitAggregation` 은 여러 파일의 exit code 합산. WASM 에서는 셋 다 사라진다 —
프로세스 기동 상각은 **이미 없고**(모듈은 한 번 인스턴스화되고 계속 산다), 파일 간
병렬은 **성립하지 않으며**(표준 `wasm32-unknown-unknown` 은 단일 스레드), stdin
경로 목록은 **경로가 없다.**

남는 것은 루프뿐이고 루프는 호출자가 짠다. **빼는 이유는 능력 부족이 아니라 그
이름이 약속하는 것(병렬·스트림·집계)을 지킬 수 없기 때문이다.**

다만 batch 가 남긴 자산 하나는 반드시 가져간다 — **오류 레코드 모양**이다. 실측:

```jsonc
{"error":"파일을 읽을 수 없습니다: 지정된 파일을 찾을 수 없습니다. (os error 2)",
 "exitClass":"runtime","schemaVersion":"1.0","source":"samples/__nope__.hwp",
 "untrustedContent":false,"untrustedFields":[]}
```

`batch_fail_record`(`src/main.rs:6501`)가 만드는 이 레코드는 rhwp 가 **실패를 봉투
안에 표현한 선례**다. WASM 예외의 payload 를 이 모양으로 고정한다
([envelope_parity.md](envelope_parity.md) §4).

### 3.5 `diagnostic` — 3개 중 1개

| 명령 | 판정 | 근거 |
| --- | :---: | --- |
| `ir-diff` | **넣음** | ↓ |
| `dump-pages` | **뺌** | "조판 진단 기계 계약"(`capabilities` summary). 답이 조판기 내부 상태다 — 관문 ② |
| `render-diff` | **뺌** | ↓ |

**`ir-diff` 를 넣는 이유.** 카테고리는 `diagnostic` 이지만 답하는 질문은 에이전트의
질문이다 — **"내가 만든 산출이 원본과 같은가"**. 모양도 WASM 에 맞는다: 바이트 두 벌
들어가고 판정 하나 나온다(`{"identical","diffCount","categories"}`). 무엇보다 이
명령이 rhwp 의 **판정 규약**을 대표한다 — `cli_commands.md` 가 "판정은 데이터"
규약의 기준점으로 `ir-diff --json` 을 든다. 판정을 반환값으로 두는 규칙
([envelope_parity.md](envelope_parity.md) §3)을 실증할 동사가 표면에 하나는 있어야 한다.

**`render-diff` 를 빼는 이유 — 측정하지 않았기 때문이다.** 렌더 두 벌 + 기하 비교는
rhwp 에서 가장 무거운 조합이고 그 비용을 **WASM 에서 측정한 적이 없다.** `--batch`
플래그(NDJSON)까지 있어 batch 와 같은 문제도 겹친다. 측정 없이 표면에 올리면 첫
사용자가 대신 측정하게 된다.

### 3.6 판정 요약

| 카테고리 | 전체 | 넣음 | 뺌 |
| --- | ---: | ---: | ---: |
| `query` | 8 | 7 | 1 |
| `export` | 16 | 8 | 8 |
| `edit` | 3 | 2 | 1 |
| `batch` | 1 | 0 | 1 |
| `diagnostic` | 3 | 1 | 2 |
| **합계** | **31** | **18** | **13** |

**빠진 13개**: `export-provenance-map` `export-pdf` `export-markdown`
`export-doclang` `export-hml` `export-ir-schema` `export-capabilities-schema`
`table-to-csv` `csv-to-table` `build-from-ingest` `batch` `dump-pages`
`render-diff`. 현재 이것들을 쓰려면 CLI·내장 MCP 표면을 쓴다. 이 조사 당시 함께
비교했던 공식 Python·Node 바인딩은 #4655에서 철회됐다.

---

## 4. 동사 명세

표기: `rhwp.동사()` 는 문서 없이 부르는 것, `doc.동사()` 는 열린 문서에 대한 것.
반환은 **봉투 객체**(JS 객체인지 JSON 문자열인지는 구현이 정하되
[envelope_parity.md](envelope_parity.md) §2 의 필드 계약을 지킨다).

### 4.0 공통 실패 규약

- **판정은 반환값, 실패는 예외.** CLI exit 3/4 → 봉투 안 필드, exit 1/2 → 예외.
  근거와 상세는 [envelope_parity.md](envelope_parity.md) §3.
- 예외 payload 는 `{error, exitClass, schemaVersion, source}` — `batch` 오류 레코드와
  같은 모양(`src/main.rs:6501`).
- **0건은 실패가 아니다.** `search` 매치 0건·`extract-data` 항목 0건·`fields` 0개는
  전부 정상 반환이다(`cli_commands.md` "매치 0건은 오류가 아니다"; 실측
  `fields --json` on `hwpers_test4_complex_table.hwp` → `{"fieldCount":0,…}` exit 0).

### 4.1 생명주기

| 동사 | 입력 | 반환 | CLI 대응 |
| --- | --- | --- | --- |
| `rhwp.capabilities()` | — | `surface:"wasm"` 자기서술 | `capabilities` |
| `rhwp.open(bytes, {password?, name?})` | 필수 문서 바이트 | 문서 핸들 | 없음 — CLI 는 매 호출 재파싱 |
| `doc.free()` | — | — | 없음 (§5.3) |

`from_bytes` / `from_bytes_with_password` 는 이미 있다(`src/wasm_api.rs:358,362`).
비밀번호는 **응답과 상태에 보존하지 않는다** — MCP 가 이미 이 규약을 명시한다
(`src/mcp_serve.rs:424` `"writeOnly": true`). 포맷 미지원·암호 불일치는 예외.

### 4.2 조회 축

| 동사 | 선택 인자 | 반환 봉투 (실측 필드) | CLI 대응 |
| --- | --- | --- | --- |
| `doc.digest()` | `sections` `pages` `maxChars` | `schemaVersion` `source` `format` `pageCount` `paraCount` `outline` `excerpt` `truncated` `nextStep` + 표지 2 | `digest --json` |
| `doc.info()` | — | `schemaVersion` `source` `format` `sizeBytes` `version` `sections` `pageCount` `paraCount` `fonts` `title` + 표지 2 | `info --json` |
| `doc.text()` | `page` `maxChars` | `schemaVersion` `source` `pageCount` `truncated` `omittedCount` `pages[]{page,text}` + 표지 2 | `export-text --json` |
| `doc.structure()` | `mode`(`auto`\|`outline`\|`clause`) | `schemaVersion` `source` `mode` `nodeCount` `structure` + 표지 2 | `export-structure --json` |
| `doc.tables()` | — | `schemaVersion` `source` `tableCount` `tables` + 표지 2 | `export-tables --json` |
| `doc.fields()` | — | `schemaVersion` `source` `fieldCount` `fields` `textSecurity` + 표지 2 | `fields --json` |
| `doc.search(q)` | `ignoreCase` `limit` `maxMatches` | `schemaVersion` `source` `query` `caseSensitive` `matchCount` `totalMatchCount` `truncated` `omittedCount` `matches[]` + 표지 2 | `search --json` |
| `doc.extractData()` | `kind` `limit` | `schemaVersion` `source` `kind` `itemCount` `totalItemCount` `truncated` `counts` `items[]` + 표지 2 | `extract-data --json` |
| `doc.inspect(axis)` | 축별 (`thresholdPt` `minConfidence` `kind` …) | 축마다 다름 ↓ | `inspect <axis> --json` |

필드는 전부 실제 실행 결과에서 옮겼다. 예: `digest --json`(393쪽 문서) →
`{…,"outline":[5개],"pageCount":393,"paraCount":2618,"truncated":false,
"untrustedContent":true,"untrustedFields":["outline[]","excerpt"]}`.

**`inspect` 세 축.** CLI 는 명령군이라 하위 명령을 요구한다(실측: 파일만 주면
`알 수 없는 inspect 하위 명령입니다` + exit 2). 봉투가 축마다 다르므로 WASM 도 축을
인자로 받는다. 실측(`samples/2022년 국립국어원 업무계획.hwp`):

| 축 | 반환 봉투 |
| --- | --- |
| `hidden-text` | `{"clean":true,"hiddenCharCount":0,"hiddenText":[],"includeOffPage":false,"thresholdPt":1.0,…}` |
| `injection` | `{"clean":true,"highestConfidence":null,"includeFields":false,"injectionSignals":[],"minConfidence":"low","scanScopes":[8종],"signalCount":0,…}` |
| `unicode` | `{"clean":true,"findingCount":0,"findings":[],"kindCounts":{4종},"kindFilter":"all","scannedChars":31126,"severityCounts":{3종},…}` |

세 축 모두 `clean: bool` 을 싣는다 — 소비자가 축을 몰라도 읽을 수 있는 공통 필드다.
`highestConfidence: null` 은 **"신호가 없다"** 이지 "판정하지 않았다"가 아니다
([envelope_parity.md](envelope_parity.md) §2.3).

> **샌드박스에서 이 축이 특히 중요한 이유**: `inspect` 는 문서 내용을 컨텍스트에
> 넣기 **전에** 부르는 동사다. 실행 파일을 못 구해 rhwp 를 못 쓰는 에이전트는 이
> 검사를 건너뛰고 문서를 읽는다. **"설치 없는 실행"은 편의 문제이자 보안 문제다.**
> 위협 모델은 [agent_security/threat_model.md](../agent_security/threat_model.md).

### 4.3 산출 축

| 동사 | 인자 | 반환 | CLI 대응 |
| --- | --- | --- | --- |
| `doc.renderPage(page)` | 선택 `profile` | `{schemaVersion, source, page, svg, bytes}` + 표지 2 | `export-svg -p N --json` |
| `doc.thumbnail()` | — | `{schemaVersion, source, format, mime, width, height, bytes, data}` + 표지 2, 없으면 `null` | `thumbnail --json` |
| `doc.save(format)` | `hwp5`\|`hwpx`. 선택 `verify` `verifyPages` `password` | `{schemaVersion, source, format, bytes, data, verify, verifyPages, wasDistribution}` + 표지 2 | `convert` / `export-hwpx --json` |

`renderPage` 의 `svg` 는 CLI 봉투의 `pages[].path` 자리에 오는 **본문 문자열**이다.
`bytes` 는 CLI 와 같은 의미(길이)로 남긴다. 매핑 규칙은
[envelope_parity.md](envelope_parity.md) §2.4.

**`save` 가 두 명령을 흡수하는 근거**: `recordFields` 가 거의 같다 — `convert`:
`source·output·format·bytes·wasDistribution·verify·verifyPages`, `export-hwpx`:
`source·output·format·bytes·verify·verifyPages`. 차이는 **목표 형식** 하나다. CLI 에서
둘로 나뉜 이유는 명령 이름이 곧 방향을 뜻해야 사람이 읽기 쉽기 때문이고, 형식을
인자로 받는 API 에는 그 이유가 없다. 실측 `convert --verify --json`:

```jsonc
{"bytes":4159488,"format":"hwp5","output":"…/conv.hwp","passwordProtected":false,
 "schemaVersion":"1.0","source":"samples/3-09월_교육_통합_2022.hwpx",
 "untrustedContent":false,"untrustedFields":[],
 "verify":{"diffCount":0,"identical":true},"verifyPages":null,"wasDistribution":false}
```

`output` 은 사라지고 `data`(바이트)가 대신 온다. `verify`/`verifyPages` 의 `null`
의미(옵션 미요청)는 그대로 보존한다.

### 4.4 편집 축

| 동사 | 인자 | 반환 | CLI 대응 |
| --- | --- | --- | --- |
| `doc.edit(action, args)` | `action` 6종 + 축별 인자, `dryRun` | 축별 봉투 + `changedPages` + 표지 2 | `edit <action> --json` |
| `doc.extractPages(from, to)` | 필수 둘 | `{schemaVersion, source, from, to, pagesBefore, pagesAfter, paragraphsKept, paragraphsRemoved}` + 표지 2 | `extract-pages --json` |
| `doc.runPlan(plan)` | `input`/`output` 없는 계획 | `{schemaVersion, planVersion, steps, verify, changedPages, assertions, invalid}` + 표지 2 | `run --json` |
| `doc.irDiff(otherBytes)` | 비교 대상 바이트 | `{schemaVersion, a, b, identical, diffCount, categories}` + 표지 2 | `ir-diff --json` |

**모든 편집 동사는 열린 문서를 제자리에서 바꾼다.** CLI 는 `-o` 로 새 파일을 쓰거나
`--in-place` 로 덮어쓰지만(둘 다 경로) WASM 은 **바꾼 뒤 `save()` 로 꺼낸다.** 이
결정의 대가는 §5.3. `--dry-run` 은 인자로 유지한다. 실측 대비:

```jsonc
// edit replace-text --dry-run --json
{"caseSensitive":true,"changedPages":null,"dryRun":true,"find":"표","occurrence":null,
 "replace":"TABLE","replacedCount":3,"schemaVersion":"1.0","source":…,
 "untrustedContent":false,"untrustedFields":[]}
// edit replace-text -o <경로> --json
{…,"changedPages":[0],"dryRun":false,"output":"…/edited.hwp","outputFormat":"hwp5",
 "replacedCount":3,…,"verify":null}
```

`changedPages` 가 `null` ↔ `[0]` 으로 갈리는 데 주목한다. `null` 은 "쓰지 않았으므로
바뀐 쪽이 없다"이고 `[]` 였다면 "썼는데 바뀐 쪽이 없다"였을 것이다. WASM 에서도
살린다 — `dryRun: true` 면 `null`, 아니면 배열.

---

## 5. 파일 없는 세계

### 5.1 `source` — 이미 답이 나와 있다

`source` 는 모든 봉투에 실리고 [agent_security/threat_model.md](../agent_security/threat_model.md)
§2.3 이 **"신뢰 없음"** 으로 분류한 필드다(첨부 파일명은 발신자가 정한다). 경로가
없으면 무엇을 넣는가? **rhwp 는 이미 경로 아닌 값을 넣고 있다:**

```rust
// src/mcp_serve.rs:882-890 — &id 는 docId 다
tool_ok_text(crate::info_json_value(&id, sd.size_bytes, sd.detected_format, &sd.doc).to_string())
```

실측(`rhwp mcp-serve` 에 JSON-RPC 직접 투입):

```jsonc
// hwp_open     → {"docId":"doc-1","pageCount":1,"schemaVersion":"1.0","source":"samples/hwpers_test4_complex_table.hwp"}
// hwp_doc_info → {…,"sizeBytes":6656,"source":"doc-1","title":"복잡한 표 테스트",
//                 "untrustedContent":true,"untrustedFields":["title","fonts[]"],"version":"5.0.3.4"}
```

**`"source":"doc-1"`** — 경로가 아니라 핸들이다. `source` 는 이미 *"이 봉투가 어느
문서를 말하는가"의 식별자*이지 "파일이 어디 있는가"가 아니다.

**규칙**: WASM 도 같은 자리에 **핸들 식별자**를 넣고 접두어로 표면을 구분한다(예:
`wasm-1`). `browser.ts:147` 의 `'(bytes)'` 같은 **상수는 쓰지 않는다** — 문서 둘을
동시에 열면 두 봉투가 구별되지 않는다. 호출자가 원본 파일명을 알면 `open(bytes, {name})`
으로 실을 수 있게 한다. 그 값은 문서가 아니라 **호출자**가 정하므로 표지 계산에서
문서 파생이 아니다([envelope_parity.md](envelope_parity.md) §2.2).

### 5.2 `-o` 와 상대 경로

CLI 의 `-o` 는 세 가지 다른 일을 한다. 하나씩 다르게 매핑한다.

| `-o` 의 성격 | CLI 예 | WASM 매핑 |
| --- | --- | --- |
| **산출이 곧 반환값** | `convert`·`export-hwpx`·`extract-pages`·`edit -o` | 바이트 반환. `output` 제거, `data` 추가 |
| **디렉터리에 N개** | `export-svg -o <폴더>`·`export-markdown`·`export-doclang` | 1쪽 단위 문자열 반환(`export-svg`) 또는 **뺌**(나머지) |
| **stdout 대안** | `export-structure -o out.json` | **없음.** 반환값이 곧 그 JSON |

세 번째가 중요하다. `export-text --json` 은 CLI 에서도 파일을 쓰지 않는다(`--help`
실측: "결과를 JSON으로 stdout에 출력 **(파일 저장 안 함)**"). 즉 `--json` 축은 이미
상당 부분 "파일 없는" 축이고, 우리가 새로 정하는 것은 첫 번째 성격뿐이다. 단
`thumbnail` 은 `-o` 없이도 파일을 쓴다(실측 봉투의
`"output":"output/…_thumb.png"` 는 기본 출력 폴더다) — WASM 판은 **항상 데이터를
돌려주고 파일을 쓰지 않는다.** 쓸 곳이 없다.

**상대 경로는 해석하지 않는다 — 경로 인자를 받지 않기 때문이다.** CLI 는 프로세스의
현재 작업 디렉터리를 기준으로 푼다. 브라우저에는 그 기준이 없고, 있는 것처럼 흉내
내면 [agent_boundary_contract.md](../agent_boundary_contract.md) S5 가 지키는 경계가
흐려진다. 그래서 모든 파일성 입력이 바이트다: `<파일.hwp>` → `bytes`,
`edit insert-image --image <경로>` → `image: Uint8Array`(+MIME 힌트),
`ir-diff <A> <B>` → 열린 문서 + `otherBytes`, `run` 계획의 `input`/`output` →
**제거**(§3.3). 경로 문자열을 받는 인자는 **하나도 두지 않는다.** 받아 놓고 무시하면
호출자는 동작한 줄 안다.

### 5.3 진단 채널 — 이것은 해결되지 않았다

CLI 계약은 stdout 과 stderr 를 가른다(`capabilities.jsonContract.stdout` 실측:
"데이터(JSON/NDJSON)만 — 진단·진행·요약은 stderr"). 실제로 조회 중에도 진단이 나온다:

```
$ rhwp export-text --json --max-chars 100 "samples/2022년 국립국어원 업무계획.hwp"
(stderr) LAYOUT_OVERFLOW_DRAW: section=0 pi=546 line=1 y=1030.9 col_bottom=1028.0 overflow=2.8px
```

발생원은 라이브러리 안이다(`src/renderer/layout/paragraph_layout.rs:3248`,
`eprintln!`). 라이브러리 크레이트(`src/main.rs` 제외)의 `eprintln!` 은 **2,487곳**.

**WASM 에는 stderr 가 없다.** 그 2,487곳의 출력이 어디로 가는지는 **확인되지 않음**
— 이 PC 에서 WASM 을 빌드해 확인하지 못했다. 구조적으로 가능한 결말은 셋이다:
조용히 버려진다 / `console.error` 로 나간다 / 패닉한다. **어느 쪽인지 측정하기 전에는
이 축의 어떤 문서도 "진단이 보존된다"고 적어서는 안 된다.**

설계 요구사항으로만 못박는다: 표면이 진단을 버리든 실어 보내든 **봉투 필드로
드러나야 한다.** 조용히 버리면 CLI 로 재현했을 때만 보이는 정보가 생기고, 그것은
"CLI 에서는 되는데 WASM 에서는 안 된다"는 재현 불가 버그의 온상이다(§8 O2).

---

## 6. 상태 모델

### 6.1 세 표면의 상태

| 표면 | 상태 | 근거 |
| --- | --- | --- |
| CLI | **없음.** 매 호출이 다시 읽고 다시 파싱한다 | `load_document(&data)` 가 명령마다 불린다(`src/main.rs:107`) |
| MCP | **핸들.** `docId` 로 세션 테이블을 찾는다 | `Sessions { docs: HashMap<String, SessionDoc>, next_id: u64 }`(`src/mcp_serve.rs:52`) |
| WASM | **객체.** 그냥 들고 있으면 된다 | `HwpDocument` 가 이미 `#[wasm_bindgen] pub struct`(`src/wasm_api.rs:336`) |

MCP 의 `SessionDoc` 이 무엇을 들고 있는지 보면 WASM 이 무엇을 공짜로 얻는지 보인다:

```rust
// src/mcp_serve.rs:42-49
struct SessionDoc {
    doc: HwpDocument,
    source_is_hwpx: bool,        // save 의 형식 보존(#3383)
    size_bytes: usize,           // hwp_doc_info 봉투용
    detected_format: rhwp::parser::FileFormat,
}
```

**WASM 객체는 이 구조체와 같은 것이다.** `HashMap` 과 `next_id` 와 문자열 핸들이
없을 뿐이다.

### 6.2 단순해지는 것

**① 핸들 무결성 문제가 사라진다.** [agent_boundary_contract.md](../agent_boundary_contract.md)
S8 이 지키는 것 — 없는·닫힌·형태가 틀린 `docId` 를 전부 `isError` 로 — 은 **문자열
핸들이 있기 때문에** 생기는 문제다. 언어의 객체 참조에는 위조가 없다. ABA 문제,
번호 재사용 금지 규칙, 죽은 핸들 안내 문구가 전부 불필요해진다.

**② 재파싱이 사라진다.** CLI 는 `search`·`fields`·`text` 세 번에 같은 문서를 세 번
판다. 참고치: 393쪽 문서 `digest` 265 ms / `export-text` 648 ms(네이티브 release,
프로세스 기동 포함, 이 PC). WASM 에서 얼마가 될지는 **측정하지 않았다**(§7).
확실한 것은 **재파싱 횟수가 1회로 준다**는 구조적 사실뿐이다.

**③ 편집→조회 왕복과 형식 보존이 공짜다.** MCP 가 `hwp_doc_fields` 를
"`hwp_doc_fill_fields` 직후 반영값 확인에 쓴다"고 설명하는 흐름
(`src/mcp_serve.rs:451`)이 WASM 에서는 메서드 두 번이고, `save` 가 원본 형식을
지키려고 기억해 두는 `source_is_hwpx`(#3383)는 객체가 원래 알고 있다.

### 6.3 위험해지는 것

**① 대형 문서가 메모리에 상주한다.** CLI 는 프로세스가 죽으면 회수된다. WASM 모듈은
탭이 사는 동안 산다. 참고치: 393쪽 샘플 원본 10,687,488 B. **파싱 후 IR 이 메모리에서
몇 바이트인지는 측정하지 않았다** — 원본의 배수라고 짐작만 할 수 있고 짐작은 근거가
아니다.

**② 해제를 잊으면 새는 것이 아니라 쌓인다.** `wasm-bindgen` 객체는 JS GC 가 자동
회수하지 않는다. `browser.ts:130-139` 이 이미 이 함정을 알고 `try/finally` 로
감쌌다("브라우저에서 누수가 나면 탭이 무거워지고 원인을 찾기 어렵다"). **규칙:
`free()` 를 계약에 명시하고, 어긋난 사용을 감지할 수단을 표면이 제공한다** —
최소한 `capabilities()` 가 살아 있는 문서 수를 보고할 수 있어야 한다. MCP 는 핸들
개수·메모리 상한이 없다고 스스로 선언하는데(S8 "보장하지 않는 것") 그 무제한을
그대로 물려받으면 상한 없는 상주 메모리가 된다.

**③ 편집이 제자리에서 일어난다 — 원본이 사라진다.** CLI `edit` 은 기본이 `-o` 새
파일이고 `--in-place` 는 명시해야 한다. **WASM 은 언제나 제자리다.**

> **설계 판단**: 이 위험은 감추지 말고 이름으로 드러낸다. `edit` 이 이미 "바꾼다"를
> 말하고 `dryRun` 이 미리 보기를 준다. 되돌리기(undo/스냅샷)는 별개 축이다 — rhwp 에
> 이미 편집 undo/redo 설계가 있고([edit_action_undo_redo_architecture.md](../edit_action_undo_redo_architecture.md))
> 두 축을 섞으면 둘 다 흐려진다. **이 축은 "원본 바이트는 호출자가 보관한다"를
> 계약으로 못박는 데서 그친다.**

**④ 상태가 있으면 오염도 남는다.** 편집이 반쯤 적용된 문서를 계속 조회할 수 있다.
CLI 는 실패하면 파일을 안 써서 원본이 온전하다("실패 시 원본 불변", `cli_commands.md`).

> **규칙: 편집 동사는 전부 아니면 전무(all-or-nothing)다.** 실패 시 문서는 호출 전
> 상태여야 하고, 그것이 보장되지 않는 축은 표면에 넣지 않는다. `run` 계획 실행기가
> 이미 이 성질을 갖고 있다 — verify 단언 실패 시 **"디스크 무변경"** 을 봉투에 적는다
> (`src/main.rs:13766` 실측 문자열). WASM 판은 같은 자리에 "문서 무변경"을 적는다.

---

## 7. 성능 — 측정한 것과 측정하지 않은 것

**측정한 것** (전부 네이티브 CLI, 이 PC, 프로세스 기동 포함):

| 항목 | 값 |
| --- | --- |
| `samples/2025 행정업무운영 편람(최종).hwp` | 10,687,488 B / 393쪽 / 2,618문단 |
| `digest --json` | 265 ms, stdout 1,375 B |
| `export-text --json` | 648 ms, stdout 645,108 B |
| `export-structure --json` | stdout 284,892 B |
| `fields --json` | stdout 47,738 B |
| `export-ir-schema --json` | stdout 44,119 B (문서 무관) |
| `export-provenance-map --json` | stdout 11,885 B (문서 무관) |

**측정하지 않은 것 — 그러므로 주장하지 않는다**: WASM 에서의 파싱·조회 시간(한 번도
재지 않았다) / 파싱 후 메모리 상주 크기 / WASM 경계 복사 비용 / 모듈 인스턴스화
비용과 `pkg/rhwp_bg.wasm` 크기. **이 축의 어떤 문서도 "WASM 이 충분히 빠르다"를
근거 없이 적어서는 안 된다.** `render-diff` 를 뺀 근거가 곧 이것이다(§3.5).

**측정 계획**: CI 의 `wasm-build` 잡이 `pkg/` 를 아티팩트로 올린다
(`.github/workflows/ci.yml:1111-1160`; 다만 `workflow_dispatch` 또는 태그일 때만
돈다 — `:1118`). 그 아티팩트로 위 항목을 재고 이 절을 갱신한다. **측정 전에는 성능
기반 설계 결정을 내리지 않는다.**

---

## 8. 열린 질문

| # | 질문 | 왜 지금 못 정하나 |
| --- | --- | --- |
| O1 | 반환값은 JS 객체인가 JSON 문자열인가 | 직렬화 비용을 측정하지 않았다. 계약은 **필드 수준**에서 고정되므로 이 선택과 독립이다 |
| O2 | 라이브러리의 `eprintln!` 2,487곳이 WASM 에서 어떻게 되나 | 이 PC 에서 WASM 빌드 불가 — **확인되지 않음**(§5.3) |
| O3 | 상주 문서 개수·메모리 상한을 표면이 강제하나 | MCP 는 상한을 두지 않았다. 브라우저에서 같은 선택이 맞는지 판단하려면 §7 측정이 먼저다 |
| O4 | 비밀번호 문서를 브라우저에서 여는 것이 옳은가 | `from_bytes_with_password` 는 이미 있다. 그러나 비밀번호가 JS 문자열로 존재하는 순간의 노출은 CLI 의 `--password`(프로세스 목록 노출)와 다른 위협이다. [agent_security](../agent_security/) 축의 판단이 필요하다 |
| O5 | `capabilities()` 의 동사 목록을 무엇으로 생성하나 | CLI 표와 같은 출처여야 드리프트가 안 생긴다. 설계는 [envelope_parity.md](envelope_parity.md) §6 |

---

## 인접 문서

- [envelope_parity.md](envelope_parity.md) — **이 문서의 짝.** 여기서 정한 동사가 거기서 봉투 모양을 얻는다
- [README.md](README.md) — 축 지도, 읽는 순서, 이 축이 무엇이 아닌지
- [envelope_provenance.md](../envelope_provenance.md) — 출처 표지의 단일 출처
- [agent_boundary_contract.md](../agent_boundary_contract.md) — S5 경로·S7 자원 한계·S8 핸들. 이 축이 S5/S8 위협을 구조적으로 없애는 이유
- [agent_security/threat_model.md](../agent_security/threat_model.md) — `inspect` 축의 존재 이유
- [weak_agent_proofing.md](../weak_agent_proofing.md) — F1 이름 환각. `capabilities()` 를 표면에 두는 이유
- [mydocs/manual/cli_commands.md](../../manual/cli_commands.md) — 31개 명령의 사람용 계약. 정확한 인자는 언제나 여기와 `--help` 가 기준
- 이슈 [#3869](https://github.com/edwardkim/rhwp/issues/3869) — 로드맵 "설치 없는 실행"
