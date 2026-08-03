---
kind: canonical
status: active
canonical: mydocs/tech/agent_runtime/envelope_parity.md
last_verified: 2026-08-03
---

# 봉투 동등성 계약 — WASM 반환값 ↔ CLI `--json`

> WASM 반환값이 CLI `--json` 봉투와 모양이 다르면 **문서가 둘로 갈리고 그 순간
> 이 축은 실패한다.** 이 문서는 무엇이 같아야 하고, 무엇이 다를 수밖에 없으며,
> 다른 것을 **어떤 규칙으로 매핑하는지**, 그리고 그 동등성을 **무엇으로 강제하는지**를
> 확정한다. 동사 목록은 [surface_spec.md](surface_spec.md), 축 지도는 [README.md](README.md).
> 로드맵 [#3869](https://github.com/edwardkim/rhwp/issues/3869).

모든 주장에 코드 경로(`파일:줄`) 또는 실제 명령 출력을 붙였다. 근거를 못 대는 항목은
**"확인되지 않음"** 으로 적었다(§7). 실측 환경은 [surface_spec.md](surface_spec.md)
머리말과 같다.

---

## 0. 왜 이것이 이 축의 성패인가

봉투가 갈리면 **문서가 갈린다.** 그러면 순서대로 이렇게 된다. ① `cli_commands.md`
와 별개로 "WASM 판 필드 설명"이 생긴다. ② 두 설명이 조금씩 어긋나는데 아무도
알아채지 못한다 — **양쪽 다 잘 돌기 때문이다.** ③ 소비자가 환경에 따라 코드를 두 벌
쓴다. ④ 그 시점에 이 축은 "CLI 를 못 쓰는 환경을 덮는 다섯 번째 진입로"가 아니라
**두 번째 제품**이 된다.

이것은 가설이 아니다. **저장소 안에서 이미 ③ 까지 진행됐다** —
`bindings/node/src/browser.ts` 가 WASM 위에서 봉투를 손으로 조립하고
(`:143-150` `source: '(bytes)'`), 그 결과 `info` 봉투가 CLI 의 12필드 대 3필드가 됐다
(실측 대비는 [surface_spec.md](surface_spec.md) §1.4). Node DESIGN.md 자신이 남은
공백을 적어 두었다(`bindings/node/docs/DESIGN.md:366`).

**이 문서의 목표는 ④ 를 막는 것이다.**

---

## 1. 계약의 범위 — 무엇을 "봉투"라 부르는가

봉투는 `--json` 이 stdout 에 내는 **한 줄 JSON 객체**다(`capabilities.jsonContract`
실측: `"stdout": "데이터(JSON/NDJSON)만 — 진단·진행·요약은 stderr"`). 계약 대상은
**[surface_spec.md](surface_spec.md) §3 이 넣기로 한 18개 명령의 봉투**뿐이다.
**범위 밖**: 사람용 텍스트 출력(`dump`·`diag` — 계약 봉투가 없다), `--json` 없이 부른
기본 출력(`export-structure` 의 무봉투 pretty JSON 등 — 기존 소비자 계약이라 건드리지
않는다, `src/main.rs:3347` 주석), CLI 에만 있는 명령의 봉투(§4 R7).

---

## 2. 무엇이 같아야 하는가

### 2.1 필드 이름과 타입 — 전부 같다

**규칙 S1. 같은 뜻의 값은 같은 이름과 같은 타입으로 나간다.** 예외는 §4 의 매핑
규칙이 명시한 것뿐이다.

`capabilities` 의 `commands[].recordFields` 가 명령별 필드 목록의 단일 출처다.
실측 예: `search` → `schemaVersion` `source` `query` `caseSensitive` `matchCount`
`totalMatchCount` `truncated` `omittedCount` `matches`. 전 목록은
[surface_spec.md](surface_spec.md) §4 표에 있다.

**타입도 같다.** `pageCount` 는 정수, `truncated` 는 불리언, `matches` 는 배열이다.
JS 로 넘어간다고 `pageCount` 를 문자열로 바꾸지 않는다. 숫자 폭이 갈릴 위험이 있는
값(`sizeBytes`·`bytes`)은 **JSON 숫자 그대로** 둔다 — 안전 정수 범위를 넘길
가능성은 문서 크기상 현실적이지 않다(실측 최대치 10,687,488 B).

### 2.2 출처 표지 — 항상 실린다

**규칙 S2. 모든 봉투는 `untrustedContent`(bool)와 `untrustedFields`(경로 배열)를
싣는다.** 문서를 열지 않는 봉투도 `untrustedContent: false` 를 **명시**한다.

정책의 원문(`capabilities.jsonContract.provenance.policy` 실측): "표지는 항상
실린다 — 문서를 열지 않는 명령의 봉투도 `untrustedContent:false` 를 명시한다".
이유는 `src/provenance.rs:24-27` 이 적었다 — 키가 없으면 "깨끗함"이 아니라 "이
바이너리는 출처를 판정하지 않음"으로 읽어야 소비자가 옛 바이너리와 구별할 수 있다.

계산은 **선언을 베끼지 않고 봉투를 훑는다**(`src/provenance.rs:447-460`
`present_fields`) — 같은 명령도 모드마다 봉투 모양이 다르기 때문이다. WASM 도
`provenance::marked(envelope, command)`(`:468`)를 **그대로 부른다.** `Value → Value`
순수 함수이고 `src/lib.rs:20` 으로 라이브러리에 있어 WASM 에서 호출 가능하다.
**표지 계산 로직을 두 번 쓰지 않는다.** 실측으로 CLI `info --json` 과 MCP
`hwp_doc_info` 가 표지까지 일치한다(둘 다
`"untrustedFields":["title","fonts[]"]`).

**`name` 은 문서 파생이 아니다.** [surface_spec.md](surface_spec.md) §5.1 이 도입한
`open(bytes, {name})` 의 `name` 은 **호출자가 정한다.** `untrustedFields` 의 정의는
"문서를 만든 사람이 내용을 정하는 값"이므로(`src/provenance.rs:23`) `name` 은 여기
들어가지 않는다. CLI 의 `source`(경로) 도 같은 이유로 표지에 없다.

> 다만 **호출자가 파일명을 그대로 넘기면 그 문자열의 출처는 발신자**다.
> [agent_security/threat_model.md](../agent_security/threat_model.md) §2.3 이 파일명을
> "신뢰 없음"으로 분류하는 이유가 그것이다. 표지 계산에는 안 들어가지만 **소비자
> 가이드는 이 값을 지시로 읽지 말라고 말해야 한다.**

### 2.3 `null` 의 의미 — 세 가지가 있고 섞으면 안 된다

실측된 `null` 사용을 전수 분류하면 셋이다.

| 부류 | 뜻 | 실측 사례 |
| --- | --- | --- |
| **N1 미요청** | 그 옵션을 주지 않았다 | `convert --verify --json` → `"verifyPages":null` (─ `--verify-pages` 를 안 줬다) |
| **N2 미해당** | 요청은 성립했으나 그 상황이 아니다 | `edit replace-text --dry-run` → `"changedPages":null` (쓰지 않았으므로 바뀐 쪽이 없다) |
| **N3 신호 없음** | 계산했고 결과가 비었다 | `inspect injection --json` → `"highestConfidence":null` (신호가 0건) |

**규칙 S3. 세 부류를 전부 `null` 로 유지하고, `undefined`·필드 생략·`0`·`""` 로
바꾸지 않는다.**

JS 에서 가장 흔한 사고가 이것이다 — `null` 을 `undefined` 로 흘리면
`JSON.stringify` 가 키를 **지운다.** 그 순간 N1(미요청)과 "그런 필드 자체가 없는
옛 버전"이 구별되지 않는다. `edit` 의 `changedPages` 가 특히 위험하다: `null`(안 썼다)
과 `[]`(썼는데 바뀐 쪽이 없다)는 **다른 사실**인데 둘 다 falsy 다. 실측 대비 —
`--dry-run` → `"changedPages":null,"dryRun":true` (output 필드 없음);
`-o <경로>` → `"changedPages":[0],"dryRun":false,"output":"…","outputFormat":"hwp5"`.

### 2.4 배열과 "0건"

**규칙 S4. 빈 결과는 빈 배열이고, 0건은 실패가 아니다.**

실측: `fields --json` on `hwpers_test4_complex_table.hwp` →
`{"fieldCount":0,"fields":[],…}` **exit 0**; 같은 문서 `export-tables --json` →
`{"tableCount":0,"tables":[],…}` exit 0. `cli_commands.md` 가 두 곳에서 못박는다
("매치 0건은 오류가 아니다 — `matchCount:0`, 종료 코드 0"; "0건은 오류가 아니다 —
`itemCount:0`").

동반 개수 필드(`fieldCount`·`tableCount`·`matchCount`·`itemCount`·`nodeCount`)도
`0` 으로 실린다. **개수 필드를 생략하지 않는다** — 배열 길이로 대신하게 하면 절단
상황에서 `matchCount`(반환된 수)와 `totalMatchCount`(전체 수)의 구분이 무너진다.
실측 `search --limit 2` → `{"matchCount":2,"totalMatchCount":36,"omittedCount":34,"truncated":true,…}`.

### 2.5 `schemaVersion` 과 진화 규칙

**규칙 S5. `schemaVersion` 은 CLI 와 같은 값이고, 같은 정책을 따른다.**

정책 원문(`capabilities.jsonContract.schemaPolicy` 실측): **"필드 추가 허용,
변경·삭제는 schemaVersion 범프"**. 현재 값은 전 명령 `"1.0"` 이다.

따라서 **WASM 이 필드를 더 실을 수 있다.** 더 싣는 것은 계약 위반이 아니다 —
`provenance::marked` 가 표지를 덧붙이면서 `schemaVersion` 을 올리지 않는 근거가
바로 이 조항이다(`src/provenance.rs:462-466` 주석). **그러나 이름을 바꾸거나
빼는 것은 위반이다.** §5 금지 규칙 참조.

---

## 3. 판정과 실패를 가르는 규칙 (#3719 불변식)

### 3.1 CLI 의 종료 코드 의미 (실측)

`rhwp capabilities` 의 `exitCodes`:

| 코드 | 의미 |
| ---: | --- |
| 0 | 성공 |
| 1 | 런타임 실패 (읽기·파싱·렌더·쓰기) |
| 2 | 사용법 오류 (인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과) |
| 3 | 검증 단언 실패 — `convert`/`export-hwpx --verify` IR 차이, `edit --verify` 저장본 불일치, `run` 계획 assertions 미충족, `render-diff --json` 시각 회귀 |
| 4 | `--verify-pages` 페이지 수 불일치 |

**3/4 는 "판정"이고 1/2 는 "실패"다.** `cli_commands.md` 가 이 구분을 명시한다 —
"차이가 검출되면 봉투를 낸 뒤 exit 3/4 로 끝난다(`ir-diff --json` 과 같은 **'판정은
데이터'** 규약). 재파싱 실패는 판정 불가이므로 stdout 을 비우고 기존 코드로 끝난다."

### 3.2 규칙

**규칙 V1. 판정은 반환값이다.** exit 3/4 에 해당하는 상황에서 WASM 은 **정상 반환**
하고 판정을 봉투 필드에 싣는다(`verify.identical`·`verify.diffCount`·`verifyPages`·
`invalid`·`status`·`regression`).

**규칙 V2. 실패는 예외다.** exit 1/2 에 해당하는 상황에서 WASM 은 **던진다.**

**규칙 V3. 예외에도 봉투가 붙는다.** 던지는 값은 문자열이 아니라 다음 payload 를
가진 오류 객체다:

```jsonc
{"error": "<사람이 읽을 진단>", "exitClass": "runtime"|"usage",
 "schemaVersion": "1.0", "source": "<핸들 또는 name>",
 "untrustedContent": false, "untrustedFields": []}
```

이 모양은 **발명이 아니다.** `batch` 의 오류 레코드가 이미 이것이다
(`batch_fail_record`, `src/main.rs:6501`). 실측:

```jsonc
{"error":"파일을 읽을 수 없습니다: 지정된 파일을 찾을 수 없습니다. (os error 2)",
 "exitClass":"runtime","schemaVersion":"1.0","source":"samples/__nope__.hwp",
 "untrustedContent":false,"untrustedFields":[]}
```

### 3.3 왜 V1 이 옳은가 — 이미 내려진 결정이다

Node 바인딩이 같은 판단을 먼저 했다(`bindings/node/docs/DESIGN.md:44` D2,
`bindings/node/src/errors.ts:1-27`): exit 1 → `RhwpRuntimeError`, exit 2 →
`UsageError`("호출을 조립한 **우리 쪽** 버그"), exit 3/4 → **"예외가 아니라 반환값의
판정 필드"**.

근거도 그대로 옮겨온다: `--verify` 불일치는 **도구가 정상 동작한 결과**다. 예외로
올리면 호출자가 `try/catch` 로 "고장"처럼 다루고, 정작 봉투에 담긴 판정 근거
(`diffCount`)를 읽지 않는다. 그러면 "표 하나가 달라졌다"와 "파일을 못 읽었다"가
같은 경로로 처리된다 — 대응이 전혀 다른데도. JS 고유의 근거도 유효하다:
"`Promise` 거절은 타입에 나타나지 않아서, 예외로 만들면 **타입 시스템이 판정의
존재를 잊는다.** 반환값에 두면 `result.verify` 가 시그니처에 남는다."

**따라서 WASM 은 새 규약을 만들지 않는다. 같은 규약의 세 번째 구현이다.**

### 3.4 현행 WASM 예외는 이 규칙을 만족하지 못한다

```rust
// src/wasm_api.rs:45-49
impl From<HwpError> for JsValue {
    fn from(err: HwpError) -> Self { JsValue::from_str(&err.to_string()) }
}
```

**던지는 값이 맨 문자열이다.** `HwpError` 는 네 변종
(`InvalidFile`·`PageOutOfRange`·`RenderError`·`InvalidField`, `src/error.rs:7-16`)
인데 경계를 넘으면서 **부류 정보가 사라진다.** CLI 는 그 손실을 문자열 부분일치로
메우고 있다:

```rust
// src/main.rs:96-104
fn classify_hwp_error(msg: &str) -> LoadError {
    if msg.contains("비밀번호가 일치하지 않") { LoadError::WrongPassword }
    else if msg.contains("비밀번호가 필요한 암호 문서") { LoadError::NeedPassword }
    else { LoadError::Other(msg.to_string()) }
}
```

**한국어 메시지 부분일치를 WASM 소비자에게 물려주면 안 된다.** 메시지 한 글자만
바뀌어도 소비자 코드가 조용히 깨진다. V3 의 `exitClass` 가 이 문제의 답이다 —
부류를 **구조로** 실어 보낸다.

### 3.5 실측된 예외 두 건 — 규칙이 덮어야 할 현실

계약 문서를 쓰면서 **자기서술과 실제가 어긋나는 지점** 두 곳을 측정했다. 판단하지
않고 사실만 적는다. 패리티 규칙은 이 현실을 덮어야 한다.

**① `run --json` 은 실패에도 봉투를 낸다.** `capabilities.jsonContract.failure` 는
"단건 명령 실패 시 stdout 0바이트; batch 는 error 레코드 + 최종 exit 1"이라고
선언하는데, `run` 은 셋째 경우다:

```console
$ rhwp run --plan-json '{"planVersion":"1.0"}' --json
{"error":"input (원본 문서 경로)이 필요합니다","schemaVersion":"1.0",
 "untrustedContent":false,"untrustedFields":[]}
$ echo $?
2
```

stdout 128바이트 + exit 2. 근거는 `run_plan_engine` 의 `usage()`/`fail()` 이 둘 다
`provenance::marked` 로 봉투를 만든다는 것(`src/main.rs:13342-13359`).

> **패리티 규칙**: WASM `runPlan` 도 **예외를 던지되 payload 에 이 봉투를 그대로
> 싣는다.** 그러면 "봉투가 있다"는 CLI 의 사실과 "실패는 예외"라는 V2 가 둘 다
> 지켜진다. 다른 명령의 0바이트 실패는 payload 에 `error`+`exitClass` 만 담긴다.

**② 스키마 두 명령은 출처 표지를 싣지 않는다.** S2 정책("표지는 항상 실린다")과
어긋난다. 실측: `capabilities` 와 `export-provenance-map --json` 은
`untrustedContent` 키가 **있고**(둘 다 `false`), `export-ir-schema --json` 과
`export-capabilities-schema --json` 은 **없다.**

> **패리티 규칙**: 이 축은 두 스키마 명령을 표면에서 **뺐으므로**
> ([surface_spec.md](surface_spec.md) §3.2) WASM 에서 재현할 대상이 아니다.
> **넣기로 한 18개는 예외 없이 S2 를 지킨다.** 위 사실은 CLI 쪽 후속 판단
> 사항이며, 이 문서는 관찰만 기록한다.

---

## 4. 무엇이 다를 수밖에 없는가 — 매핑 규칙

### R1. `source` — 경로가 없다

**규칙**: `source` 에는 **핸들 식별자**를 넣는다(예: `wasm-1`). 호출자가
`open(bytes, {name})` 로 이름을 주면 그 값을 쓴다.

**근거**: rhwp 는 이미 `source` 에 경로 아닌 값을 넣고 있다. MCP `hwp_doc_info` 는
`crate::info_json_value(&id, …)`(`src/mcp_serve.rs:882-890`)로 **`docId` 를 `source`
자리에 넣는다.** 실측: `"source":"doc-1"`.

**금지**: `browser.ts:147` 의 `'(bytes)'` 같은 **상수**. 문서를 둘 열면 두 봉투가
구별되지 않는다. 타입은 CLI 와 같은 문자열이고, 값이 경로 형태일 필요는 없다 —
이미 MCP 가 그렇지 않다.

### R2. exit code — 예외가 된다

**규칙**: §3 의 V1/V2/V3. `exitClass` 는 `"runtime"`(exit 1) 또는 `"usage"`(exit 2).

**왜 코드 숫자를 안 싣는가**: WASM 에는 프로세스가 없으므로 `exitCode: 1` 은
존재하지 않는 사실이다. 대신 `batch` 가 이미 쓰는 **부류 이름**을 쓴다
(`"exitClass":"runtime"`, `src/main.rs:6507`). 숫자를 원하는 소비자는 부류에서
역산할 수 있고, 부류는 프로세스 유무와 무관하게 참이다.

**exit 0 의 특수 사례 — 판정도 exit 0 이다.** `search` 매치 0건, `edit` 치환 0건은
exit 0 이자 정상 반환이다(§2.4). 예외를 던지지 않는다.

### R3. 파일 산출 — 바이트가 된다

**규칙**: `output`(경로 문자열) **제거**, `data`(바이트) **신설**, `outputFormat`
과 `bytes`(길이)는 **유지**. `bytes` 를 남기는 이유는 CLI 와 같은 뜻이고 소비자가
크기만 알고 싶을 때 `data.length` 를 세지 않아도 되기 때문이다.

실측 대비 — `convert --verify --json` 은
`{"bytes":4159488,"format":"hwp5","output":"…/conv.hwp",…}` 를 낸다. WASM `save()` 는
`output` 자리에 아무것도 놓지 않고 `data` 를 더한다. `bytes` 4159488 은 그대로다.

**매니페스트형 산출은 다르다.** `export-svg --json` 은 파일 목록을 낸다 —
`{"outputDir":"…/svgout","pages":[{"page":0,"bytes":24353,"path":"…/x.svg"}],"renderedCount":1,…}`.
**규칙**: `outputDir` **제거**, `pages[].path` → `pages[].svg`(본문 문자열),
`pages[].bytes` 유지. [surface_spec.md](surface_spec.md) §3.2 대로 WASM 은 1쪽
단위만 내므로 `pages` 는 길이 1 이 된다 — **`page`·`bytes` 필드 이름은 같다.**

**썸네일**: CLI 는 `--base64`/`--data-uri` 라는 **바이트를 문자열로 넘기는 선례**를
이미 갖고 있다. WASM 은 `data` 를 바이트로 내되 `mime`·`width`·`height`·`bytes` 는
CLI 봉투와 같은 이름·같은 뜻으로 싣는다.

### R4. `stderr` — 대응물이 없다

**규칙**: **아직 정하지 않는다.** 정할 근거가 없다.

CLI 계약은 진단을 stderr 로 보낸다(`capabilities.jsonContract.stdout` 실측). 조회
중에도 나온다 — 실측:

```
(stderr) LAYOUT_OVERFLOW_DRAW: section=0 pi=546 line=1 y=1030.9 col_bottom=1028.0 overflow=2.8px
```

발생원은 라이브러리 안(`src/renderer/layout/paragraph_layout.rs:3248`, `eprintln!`)
이고 라이브러리 크레이트의 `eprintln!` 은 **2,487곳**이다. WASM 에 stderr 가 없으므로
그 출력이 어디로 가는지는 **확인되지 않음**(§7 U1).

**지금 못박는 것은 하나**: 진단을 버리기로 하든 실어 보내든 **결정이 봉투에
드러나야 한다.** 조용히 버리면 "CLI 에서는 보이는데 WASM 에서는 안 보이는" 정보가
생기고, 그것은 재현 불가 버그의 온상이다. 측정 후 이 절을 갱신한다.

### R5. NDJSON 스트림 — 없다

**규칙**: `batch` 를 뺐으므로([surface_spec.md](surface_spec.md) §3.4) NDJSON
대응물을 만들지 않는다. **단건 봉투와 batch NDJSON 레코드가 같은 스키마라는 CLI 의
성질은 유지된다** — `structure_json_value` 같은 봉투 생성기를 단건과 batch 가 공유
하기 때문이다(`src/main.rs:3409` 주석: "봉투는 한 줄 — NDJSON(batch)과 같은 스키마로
단건/배치 동일 소비"). WASM 은 그 **단건** 모양을 쓴다.

### R6. 상태 — 재파싱이 없다

**규칙**: 같은 문서에 대한 연속 호출이 **같은 봉투를 낸다.** CLI 는 매 호출 재파싱
하고 WASM 은 안 하지만 그 차이가 봉투에 나타나면 안 된다. **주의 지점은 편집 후
조회다** — CLI 에서 `edit` 은 새 파일을 만들고 이후 조회는 그 파일을 열지만 WASM 은
같은 객체를 이어 쓴다. `pageCount` 처럼 편집으로 바뀌는 값은 **편집이 반영된 뒤의
값**이어야 한다. MCP 가 같은 요구를 이미 갖고 있다(`hwp_doc_info` 설명: "편집 후
페이지 수 변화를 추적할 때 쓴다", `src/mcp_serve.rs:447`).

### R7. 없는 명령 — 흉내 내지 않는다

**규칙**: 표면에서 뺀 13개 명령의 봉투를 **WASM 에서 합성하지 않는다.** 부르면
`usage` 부류 예외를 던지고 무엇을 대신 쓸지 안내한다. **근거**: `browser.ts` 가
없는 함수를 옵셔널로 선언한 것이 정확히 이 실패의 초기 형태다.
`bindings/node/src/browser.ts:22` 가 같은 원칙을 적었다 — "없는 기능을 인터페이스에
넣고 런타임에 던지는 것보다, 타입이 처음부터 말하는 편이 낫다."

---

## 5. 금지 규칙 — 절대 하지 말 것

| # | 금지 | 왜 |
| --- | --- | --- |
| X1 | 필드 이름을 camelCase 로 "다듬기" | CLI 봉투가 이미 camelCase 다(`pageCount`·`untrustedFields`). 다듬을 것이 없고, 다듬는 순간 두 이름이 생긴다 |
| X2 | `null` 을 `undefined` 로 흘리기 | `JSON.stringify` 가 키를 지운다. N1/N2/N3(§2.3)이 "필드 없음"과 뭉개진다 |
| X3 | 표지(`untrustedContent`/`untrustedFields`) 생략 | S2 위반. 소비자가 옛 바이너리와 구별할 수 없다(`src/provenance.rs:24-27`) |
| X4 | 봉투를 소비 측 언어로 조립 | `browser.ts` 가 한 일. 두 벌이 되면 반드시 어긋난다(`bindings/README.md`) |
| X5 | 판정(verify 불일치)을 예외로 올리기 | V1 위반. Node D2 가 이미 기각한 설계 |
| X6 | 오류를 맨 문자열로 던지기 | 현행 `From<HwpError> for JsValue` 의 문제(§3.4). 부류가 사라진다 |
| X7 | 필드를 **빼거나 이름을 바꾸면서** `schemaVersion` 유지 | S5 위반 (`jsonContract.schemaPolicy`) |
| X8 | `source` 에 상수 넣기 | R1. 문서 둘을 구별할 수 없다 |

**추가할 수는 있다.** WASM 전용 필드(예: `surface:"wasm"`)를 더 싣는 것은 S5 가
허용한다. 다만 **CLI 에 없는 필드에 CLI 에 있는 이름을 쓰지 않는다** — 같은 이름
다른 뜻이 가장 고치기 어려운 드리프트다.

---

## 6. 동등성을 어떻게 강제할 것인가

선언은 강제가 아니다. `provenance_contract.rs` 가 이 교훈을 파일 머리에 적었다 —
"출처 표지는 **선언**이다. 선언은 코드가 바뀌어도 조용히 그대로 남는다 … 그래서
여기서는 **선언을 믿지 않는다.**"

### 6.1 왜 기존 `tests/` 로는 안 되는가

저장소의 계약 테스트는 **CLI 바이너리를 띄워** 봉투를 받는다(`rhwp_bin()` →
`std::process::Command`). 그래서 전부 네이티브 전용이다:

```rust
// tests/provenance_contract.rs:17
#![cfg(not(target_arch = "wasm32"))]
```

`tests/*.rs` 440개 중 **104개**가 이 게이트를 달고 있다(`grep -rl`). WASM 표면은 이
안에서 검증할 수 없다 — 프로세스를 띄울 수 없고, 애초에 `wasm32` 로 컴파일되지 않는다.

### 6.2 2단 설계 — 골든 생성과 비교를 분리한다

```
┌── 1단계: 네이티브 (CI, 기존 테스트와 같은 방식) ────────────┐
│  샘플 N개 × 동사 18개 → rhwp <명령> --json                   │
│  → tests/fixtures/envelope_golden/<명령>__<샘플>.json 로 저장 │
└──────────────────────────────────────────────────────────────┘
                              ↓ 골든 파일 (저장소에 커밋)
┌── 2단계: WASM (frontend-package-gates 잡) ───────────────────┐
│  wasm-pack build → 같은 샘플 바이트를 WASM 동사에 투입        │
│  → 반환 봉투를 골든과 §6.3 규칙으로 대조                      │
└──────────────────────────────────────────────────────────────┘
```

**둘 다 붙을 자리가 이미 있다.**

- 2단계의 실행 환경: `frontend-package-gates` 잡이 `wasm-pack build --target web --dev`
  로 `pkg/` 를 만들고(`.github/workflows/ci.yml:930`) 그 뒤 Node 테스트를 돌린다
  (`:951` `node --test scripts/frontend-wasm-bindings.test.mjs …`).
- 그 잡의 트리거에 **`src/wasm_api.rs` 가 이미 들어 있다**(`ci.yml:364`
  `isFrontendPath`). 즉 WASM 표면을 건드린 PR 은 자동으로 이 게이트를 탄다.
- 선례가 되는 드리프트 가드도 그 자리에 있다 — `scripts/frontend-wasm-bindings.test.mjs`
  는 `src/wasm_api.rs` 의 `js_name` 전수를 `pkg/rhwp.d.ts` 와 대조한다.

**골든을 커밋하는 이유**: 두 단계를 한 잡에서 돌리려면 그 잡이 Rust 네이티브 빌드와
WASM 빌드를 둘 다 해야 한다. 골든을 파일로 남기면 2단계는 **`pkg/` 만** 있으면 되고,
1단계는 기존 네이티브 테스트 잡에 얹힌다. 대가는 골든이 낡을 수 있다는 것 — 그래서
1단계 테스트가 **골든을 재생성해 커밋본과 비교**하고, 다르면 실패한다(재생성 스크립트
경로를 실패 메시지에 실어 준다).

### 6.3 비교 규칙 — 무엇을 정규화하고 무엇을 정규화하지 않는가

**정규화한다 (R1~R3 이 다르다고 인정한 것):** `source` — 양쪽 마스킹, **존재와
타입(문자열)만** 확인 / `output`·`outputDir` — 골든에서 제거, WASM 쪽에 **없어야**
통과 / `data` — 골든에 없다, WASM 쪽 존재와 `bytes` 와의 길이 일치만 확인 /
`pages[].path` ↔ `pages[].svg` — 이름 대응만 확인, 본문은 비교하지 않는다.

**정규화하지 않는다 (같아야 하는 것):** 나머지 전부. 키 집합, 타입, `null` 위치,
빈 배열 여부, 개수 필드 값, `truncated`/`omittedCount`, `untrustedContent`,
`untrustedFields` 배열 내용, `textSecurity`, `verify` 하위 필드, `schemaVersion`.

**세 층으로 나눠 판정한다** — 어긋남의 성격이 다르면 진단도 달라야 한다.
① **키 집합**: 빠진 키는 **실패**, 더 붙은 키는 S5 가 허용하므로 **경고**.
② **타입**: 같은 키의 JSON 타입이 다르면 실패이고 `null` 은 별도 타입으로 센다
(`null` ↔ `0` 은 §2.3 위반). ③ **값**: 문서 파생 문자열(`text`·`title`·`context`)을
제외한 **엔진 산출 값**만 비교한다 — 문서 파생 값에서 차이가 나면 그것은 봉투
문제가 아니라 **엔진 문제**이므로 다른 테스트가 잡을 일이다. 섞으면 진단이 흐려진다.

### 6.4 실패 축 — 판정과 실패도 대조한다

봉투만 맞춰서는 §3 이 지켜지지 않는다. 같은 샘플 축에 **실패 사례**를 넣는다.

| 사례 | CLI 실측 | WASM 기대 |
| --- | --- | --- |
| 깨진 바이트 | exit 1, stdout 0 B, stderr `문서 파싱 실패 - 유효하지 않은 파일: …` | `exitClass:"runtime"` 예외 |
| 알 수 없는 옵션 | exit 2, stdout 0 B | `exitClass:"usage"` 예외 |
| 없는 쪽 번호 | exit 2 (`PageOutOfRange`) | `exitClass:"usage"` 예외 |
| `convert --verify` 차이 | exit 3 + **봉투 있음** | **정상 반환**, `verify.identical:false` |
| `run` 인자 부족 | exit 2 + **봉투 있음**(§3.5 ①) | 예외, payload 에 그 봉투 |

앞 세 줄은 실측했다. 넷째 줄은 이 PC 의 샘플에서 `verify.identical:true`(exit 0)만
재현됐다 — **exit 3 을 실제로 내는 문서·명령 조합은 확보하지 못했다**(§7 U3).
계약 테스트를 만들 때 `tests/convert_verify_corpus_ratchet.rs` 가 쓰는 코퍼스에서
골라야 한다.

### 6.5 선례 — 이 설계가 작동한다는 증거

같은 발상의 가드가 이미 결함을 잡았다(`bindings/node/docs/DESIGN.md:224` D11):

- **M18 에서 5건** — `export-doclang` 래퍼 누락, 세션 `output` 필수 인자 누락,
  MCP 도구 누락, `inputSchema.required` 누락, 선언만 하고 배선 안 된 속성.
- **M19 에서 2건, 그것도 정반대 방향** — `render-diff` 를 (1) `--json` 이 없는데
  감쌌고(**없는 계약을 감쌈**) (2) `--json` 이 생기자 이번엔 빠뜨렸다(**있는 계약을
  빠뜨림**). **테스트 하나에 둘 다 걸렸다.**

DESIGN.md 의 결론이 이 축에도 그대로 적용된다: "노출 기준을 손으로 고른 목록이
아니라 자기서술에 둔 설계의 값어치가 여기서 나온다."

**따라서 §6.2 의 골든도 손으로 적지 않는다.** 대상 명령 목록은 `capabilities` 에서
읽고, 그중 [surface_spec.md](surface_spec.md) §3 이 넣기로 한 18개만 거른다. 목록을
손으로 관리하면 새 명령이 조용히 빠진다.

### 6.6 이 설계가 잡지 못하는 것

과장은 계약 문서의 흔한 실패다. 못 잡는 것을 적는다.

- **골든에 없는 샘플의 차이.** 샘플이 덮지 않는 문서 특성(암호·HWP3·HML·수식·대형
  표)에서 갈라지면 못 잡는다. 샘플 축을 넓히는 것은 별개 과제다.
- **성능 회귀.** 봉투가 같아도 100배 느릴 수 있다(§7 U2). **진단 손실**도 못 잡는다 —
  R4 가 미정인 동안 stderr 축은 비교 대상이 아니다.
- **의미는 같은데 값이 다른 경우.** 부동소수 좌표가 마지막 자리에서 갈리면 값 비교가
  실패로 보고한다 — 허용 오차 규약이 필요하고, 대상 필드가 나온 뒤에 정한다.
- **`frontend-package-gates` 가 안 도는 PR.** 트리거는 경로 기반이라
  (`ci.yml:358-370`) WASM 표면 파일을 안 건드리면서 봉투를 바꾸는 변경 — 예를 들어
  `src/provenance.rs` 만 고치는 PR — 은 이 게이트를 타지 않는다.
  **`isFrontendPath` 에 봉투 관련 경로를 추가해야 한다.**

---

## 7. 확인되지 않음

| # | 항목 | 왜 |
| --- | --- | --- |
| U1 | 라이브러리의 `eprintln!` 2,487곳이 WASM 에서 어디로 가는가 | 이 PC 는 rhwp WASM 빌드가 불가능하다. R4 가 미정인 이유 |
| U2 | WASM 에서의 파싱·조회 시간, 메모리 상주 크기, 경계 복사 비용 | 한 번도 재지 않았다. [surface_spec.md](surface_spec.md) §7 |
| U3 | exit 3(verify 불일치)을 실제로 내는 문서·명령 조합 | 이 PC 샘플로 재현하지 못했다. §6.4 넷째 줄은 **기대치이지 실측이 아니다** |
| U4 | 현행 WASM 함수들이 반환하는 JSON 의 정확한 필드 목록 | `getFieldList`·`getStructure` 등은 문서화된 doc 주석만 확인했고 실행 결과를 못 봤다(빌드 불가). 이 문서는 **CLI 쪽 실측만** 근거로 삼았다 |
| U5 | `serde-wasm-bindgen` 대 JSON 문자열의 비용 차 | 측정하지 않았다. 계약이 필드 수준이라 결정과 독립이다 |

**U4 는 특히 중요하다.** 이 문서가 CLI 봉투를 기준으로 쓰인 이유가 그것이다 —
CLI 는 실행해서 봤고, WASM 은 코드로만 읽었다. WASM 쪽 실측이 가능해지면 §3.4 와
§4 를 재검토해야 한다.

---

## 인접 문서

- [surface_spec.md](surface_spec.md) — **이 문서의 짝.** 어떤 동사가 표면에 있는지
- [README.md](README.md) — 축 지도
- [envelope_provenance.md](../envelope_provenance.md) — 출처 표지 계약의 단일 출처. S2 의 근거
- [agent_boundary_contract.md](../agent_boundary_contract.md) — S7 자원 한계(절단은 반드시 봉투에 드러난다). §2.4 의 근거
- [agent_security/threat_model.md](../agent_security/threat_model.md) — `source`·파일명이 왜 "신뢰 없음"인지
- [weak_agent_proofing.md](../weak_agent_proofing.md) — F4(정보 없는 실패가 루프의 원료). V3 의 이유
- [mydocs/manual/cli_commands.md](../../manual/cli_commands.md) — 종료 코드 계약(#2707)과 "판정은 데이터" 규약
- `bindings/node/docs/DESIGN.md` — D2(판정을 예외로 만들지 않는다)·D8(같은 인터페이스)·D11(패리티 가드)
- 이슈 [#3719](https://github.com/edwardkim/rhwp/issues/3719) — 판정/실패 불변식의 출처 · [#3869](https://github.com/edwardkim/rhwp/issues/3869) — 로드맵 "설치 없는 실행"
