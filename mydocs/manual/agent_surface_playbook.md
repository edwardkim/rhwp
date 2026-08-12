---
kind: canonical
status: active
canonical: mydocs/manual/agent_surface_playbook.md
last_verified: 2026-08-09
---

# 에이전트 표면 플레이북 — 표면을 더하는 절차와, 그 표면을 굴리는 실무

rhwp 의 **에이전트 표면**(CLI 기계 계약 + MCP 도구 + 세션 도구)을 다루는 두 가지 일을
한 문서에 고정한다.

- **제1부 — 더할 때**: 새 조각을 추가하는 공식 절차와 수용 기준. 로드맵·잔여 목록의
  권위는 #3608(에이전트 표면 전면 커버리지, #2659 후속)이고, 제1부는 그 로드맵의
  조각을 **실제로 추가할 때 지켜야 하는 계약**이다. 절차를 어긴 표면 추가는 되돌린다.
- **제2부 — 쓸 때**: 이미 있는 표면을 실무로 굴리는 절차. 진입로별 첫 호출, 판정
  읽는 법, 컨텍스트 예산, 편집 안전 절차, 실패 처방.

**제2부의 모든 예시는 실제 실행 출력이다.** 기준은 `rhwp v0.8.2` release 빌드,
2026-08-03, 표본은 `samples/`. 지면상 긴 배열은 `…` 로 줄였고, 로컬 절대 경로는
상대 경로(`samples/`·`out/`)로 바꿔 실행했다. 돌려 보지 못한 것은 "확인되지 않음"으로
적었다.

---

# 제1부 — 표면을 더할 때

## 1. 표면의 3층 구조 (어디에 더하는가)

| 층 | 무엇 | 단일 출처 |
|---|---|---|
| CLI `--json` | stdout 순수 JSON 봉투 + #2707 종료 코드 | 각 명령 구현 + 봉투 helper (`*_json_value`) |
| MCP 무상태 도구 | 선언(`capabilities --mcp`)과 실행(`mcp-serve`)이 공유하는 도구 | `mcp_tool_definitions()` (src/main.rs) |
| MCP 세션 도구 | 열린 핸들(`docId`) 위의 재파싱 없는 연산 | `mcp_serve.rs` 의 `served_tools()`+디스패치 |

**규칙 1 — 선언·실행·문서는 한 곳에서 갈라진다.** 무상태 도구는
`mcp_tool_definitions()` 에만 추가하면 선언과 서버가 함께 얻는다. 도구 목록을
다른 곳에 복제하지 않는다. 드리프트 가드
(`capabilities_mcp_covers_every_json_command`,
`tools_list_matches_capabilities_manifest`)가 어긋남을 잡는다.

**규칙 2 — 새 편집·조회 로직을 만들지 않는다.** MCP/세션 도구는 검증된 코어
함수(`set_field_value_by_name_at`, `replace_all_native`, `grep`,
`collect_field_records`, `extract_tables`, `edit_serialize` …)와 기존 봉투
helper 를 재사용한다. 서버 전용 경로를 새로 만들면 CLI 와 계약이 갈라진다.

**규칙 3 — 판정은 데이터다.** 차이 검출(`identical:false`)·치환 0건·`notFound` 는
오류가 아니라 봉투 필드다. `isError:true` 는 실행 실패(없는 파일, 닫힌 핸들)에만
쓴다. CLI 는 exit 3/4 로 판정을 신호하되 봉투를 먼저 낸다.

**규칙 4 — 출처 표지를 함께 낸다.** 문서에서 온 값을 봉투에 실으면
`untrustedContent:true` 와 `untrustedFields[]` 를 같이 싣고,
`export-provenance-map` 의 지도에도 항목을 추가한다. 문서를 열지 않는 명령도
`untrustedContent:false` 를 **명시**한다(키 부재는 옛 바이너리와 구별되지 않는다).
`tests/provenance_contract.rs` 가 선언이 아니라 실제 봉투 값을 보고 누락을 잡는다.

## 2. 추가 절차 (순서 고정)

0. **잠금 확인·할당 (착수 = 할당)** — 조사보다 먼저 한다. 대상 이슈의 assignee 와
   같은 이슈를 가리키는 열린 PR 을 확인하고, 비어 있으면 즉시 선점한다.
   ```bash
   gh issue view <n> --repo edwardkim/rhwp --json assignees -q '.assignees[].login'
   gh pr list --repo edwardkim/rhwp --state open --limit 100 --search "<n>"
   gh issue edit <n> --repo edwardkim/rhwp --add-assignee @me   # 권한 있는 계정만 성공
   ```
   **외부 기여자 계정은 assignee 편집이 거부된다**(실측 2026-08-06:
   `gh issue edit --add-assignee` 가 `failed to update 1 issue` — 3회 재현). 그 경우
   **이슈에 착수 코멘트("착수합니다 — <범위>")를 남기는 것이 잠금**이다. 선점된
   이슈(assignee 있음 또는 착수 코멘트 있음)는 착수하지 않고, 작업을 접으면 즉시
   해제(코멘트)한다. 사고 사례·판정 기준·볼륨 캡의 canonical 은
   [병렬 세션 규약](../tech/autonomous_maintenance/parallel_session_protocol.md)이다.
1. **이슈 등록** — 공백을 실측으로 서술하고(#3608 매트릭스 갱신 포함) 검증 계획을 적는다.
2. **red 계약 테스트** — `tests/*_contract.rs` 신설. 구현 전 FAILED 를 확인한다.
   기존 테스트 파일 수정보다 신설을 우선한다(병렬 PR 충돌 회피).
3. **구현** — 규칙 1~4 준수. 실패 경로 stdout 순수성(부분 산출물 미출력) 포함.
4. **검증** — 신규 green + 인접 계약 스위트 무회귀 + `clippy -D warnings` 0 +
   rustfmt clean(변경 파일 기준).
5. **누적 머지 충돌검사** — `upstream/devel` 에서 임시 브랜치를 만들어 열린 PR
   브랜치 전부를 순차 merge, 충돌 0 확인. 겹치는 파일이 있으면 **선등재 성립
   여부를 먼저 보고**, 안 되면 적층(베이스 PR 을 본문에 명시, 체인 3단 이하)으로
   전환한다.

   **접합 기법 — 선등재**: 겹침이 "목록·표에 한 줄 추가" 형태이고 그 목록의 소비자가
   기준 집합만 순회한다면(초과 항목 무해), 상대 PR 이 추가할 항목을 미리 등재해
   머지 순서와 무관하게 무수정 통과시킬 수 있다. 실증 #3903 ↔ #3808 (SWEEP_EXEMPT
   면제 호출표 선등재 — 누적 머지로 전후 대조). 성립 조건과 한계의 canonical 은
   [선등재 패턴](../tech/autonomous_maintenance/pre_registration_pattern.md).
6. **처리 문서 + 증적 2종** — `mydocs/report/task_m100_<이슈>/README.md` 에:
   ① 실행 원문(터미널 봉투) 캡처 ② **산출물을 실제 rhwp 로 열어 렌더한 화면**
   (`export-svg` → PNG 변환 → 합성). 편집 계열은 전/후 비교로.
7. **PR** — 한글 제목·본문, `closes #<이슈>`, 증적 이미지는 저장소에 커밋 후
   raw 링크로 본문 참조. 열린 PR 은 10건 이내를 유지한다.

## 3. 수용 기준 (Definition of Done — 조각 단위)

- [ ] stdout 순수성: `--json` 모드에서 stdout 에 JSON 하나(배치는 NDJSON)만.
      진단·진행 메시지는 stderr.
- [ ] 실패 경로: 런타임 실패 시 stdout 비움(부분 매니페스트 금지), exit 1.
      조립 오류는 exit 2 — **미지 옵션 침묵 무시 금지**.
- [ ] `schemaVersion` 필드 포함, 필드 추가는 허용·변경/삭제는 계약 테스트가 잡는 구조.
- [ ] 출처 표지: `untrustedContent`·`untrustedFields` 를 **모든 모드에서**(dry-run
      포함) 싣고, `export-provenance-map` 에 항목 추가.
- [ ] 무상태 도구: `inputSchema.required` 와 `cli.args` 자리표시자가 1:1
      (선택 인자를 자리표시자로 쓰지 않는다 — 미치환 문자열이 CLI 로 새는 사고 방지).
- [ ] 세션 도구: 닫힌 핸들 `isError`, 디스크 기록은 `hwp_doc_save` 만, 판정 어휘는
      무상태 대응 도구와 동형.
- [ ] 실패 응답에 `nextCall{name,arguments,why}` 유도 — 에이전트가 다음 수를 안다.
- [ ] 문서: `cli_commands.md` 해당 절 현행화(+ 필요 시 `mcp_integration_guide.md`),
      [지식 지도](agent_knowledge_map.md) §1-1·§2·§5·§6·§8 에 **행 추가**.

### 3-1. 지금 남아 있는 수용 기준 미충족 (실측 2026-08-03)

출처 표지 항목이 **아직 전부 지켜지지 않는다.** 다음 6개 봉투에는
`untrustedContent`·`untrustedFields` 키가 아예 없다.

| 봉투 | 문서 파생 값을 싣는가 |
|---|---|
| `edit redact --json` | **예** — `findings[].raw` 에 원문 개인정보 |
| `edit sanitize --json` | **예** — `removed[].before` 에 원본 메타데이터 |
| `run --dry-run --json` | 예 — `preview[].targets[].name` |
| `edit insert-image --json` | 아니오 |
| `export-ir-schema --json` | 아니오 |
| `export-capabilities-schema --json` | 아니오 |

같은 `run` 이라도 실행 모드에서는 표지가 실린다. 소비자 쪽 대응은 §10-7 에 있다.

## 4. 증적 규약 (따라 하기 어려운 이유를 유지한다)

증적은 **재현 가능해야** 한다 — 이미지와 함께 재현 명령을 보고서에 남긴다.
가짜/합성 데이터로 만든 화면은 반드시 그렇게 표기하고, 실물 문서(인터넷 배포
서식 등)를 쓸 수 있으면 실물을 우선한다. 다쪽 문서 편집은 "건드리지 않은 쪽의
불변"(픽셀 대조)까지 포함한다.

## 5. 로드맵 연동

- 잔여 목록·우선순위·명시적 제외의 권위: **#3608** (§1 매트릭스, §6.5 백로그).
- 조각을 착수하면 #3608 의 해당 항목에 이슈 번호를 달고, 머지되면 체크한다.
- 매트릭스는 `capabilities` 교차 스크립트(#3608 §5)로 재생성해 감으로 갱신하지 않는다.

---

# 제2부 — 표면을 쓸 때

## 6. 진입로별 첫 호출 — "처음 뭘 부르나"

rhwp 에는 네 개의 진입로가 있고 **첫 호출이 서로 다르다.** 잘못된 첫 호출은
"도구가 없다"는 잘못된 결론으로 이어진다.

### 6-1. CLI — 자기서술 1회 캐시

```
$ rhwp capabilities
```

명령·플래그·`recordFields`·종료 코드·batch 규칙·formats 가 한 번에 온다(16.7KB).
`--json` 이 붙지 않는다 — `capabilities` 는 언제나 JSON 이다.

읽어야 할 것 두 가지:

```
$ rhwp capabilities | python -c "import sys,json;d=json.load(sys.stdin);
print(d['version'], len(d['commands']), sum(1 for c in d['commands'] if c.get('json')))"
0.8.2 61 31
```

그리고 **가용성**:

```
$ rhwp capabilities | python -c "import sys,json;d=json.load(sys.stdin);
print([(c['name'],c.get('available'),c.get('requiresFeature')) for c in d['commands'] if c.get('available') is not None])"
[('export-png', False, 'native-skia')]
```

`available:false` 인 명령을 부르면 이렇게 끝난다:

```
$ rhwp export-png samples/추진일정.hwp -o out/png
오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.
       cargo build --release --features native-skia
exit=2
```

**첫 호출로 하면 안 되는 것** — `rhwp --help` 를 파싱해 명령 목록을 만드는 것.
사람용 서식이라 계약이 아니다. `capabilities` 가 계약이다.

### 6-2. MCP 무상태 — 선언을 먼저 읽고, 자리표시자를 치환한다

```
$ rhwp capabilities --mcp
```

39개 도구의 `name`·`description`·`inputSchema`·`cli.args`·`outputFields` 가 온다.
`cli.args` 는 자리표시자를 담은 인자 배열이다:

```json
{"name":"hwp_search","cli":{"command":"search","args":["search","{path}","--json","--","{query}"]},
 "inputSchema":{"type":"object","required":["path","query"], "properties":{ … }}}
```

**규칙**: `{name}` 자리표시자를 `inputSchema` 의 같은 이름 값으로 치환해 실행한다.
`required` 에 없는 값은 자리표시자로 쓰이지 않는다 — 미치환 문자열이 CLI 로 새면
`{output}` 같은 이름의 파일이 만들어지는 사고가 난다.

`invocation.stdinTools` 에 적힌 도구(`hwp_batch`·`hwp_batch_search`)는 **경로 목록을
stdin 으로** 흘려야 한다. 이 둘만 예외다.

역할이 정해져 있으면 처음부터 좁힌다:

```
$ rhwp capabilities --mcp --profile 행정서식
# tools 8개 + profile.recipe[] 5줄
```

### 6-3. MCP 세션 — 핸드셰이크 → `tools/list` → `hwp_open`

`mcp-serve` 는 stdio JSON-RPC 서버다. 첫 세 줄이 고정 순서다.

```
$ printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"probe","version":"0"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | rhwp mcp-serve
```

`initialize` 응답:

```json
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"resources":{},"tools":{}},
 "protocolVersion":"2025-06-18","serverInfo":{"name":"rhwp","version":"0.8.2"}}}
```

**요청한 프로토콜 버전과 응답이 다르다**(`2024-11-05` → `2025-06-18`). 서버가 자기
버전을 말하는 것이니 응답 쪽을 기준으로 삼는다.

`tools/list` 는 **51개**를 준다 — `capabilities --mcp` 의 39개 + 세션 12개. 세션
도구는 선언 매니페스트에 없으므로, 세션을 쓰려면 반드시 서버에 물어야 한다.

문서를 여는 첫 호출:

```json
→ {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"hwp_open",
    "arguments":{"path":"samples/2025 행정업무운영 편람(최종).hwpx"}}}
← {"id":2,"jsonrpc":"2.0","result":{"content":[{"text":"{\"docId\":\"doc-1\", …}","type":"text"}],
    "isError":false,
    "structuredContent":{"docId":"doc-1","pageCount":387,"schemaVersion":"1.0",
                         "source":"samples/2025 행정업무운영 편람(최종).hwpx"}}}
```

`docId` 는 **서버 프로세스 수명과 같다.** 재시작하면 사라지고, 저장하지 않은 편집도
함께 사라진다.

### 6-4. 외부 소비자 코드 생성 — 스키마를 먼저 뽑는다

바인딩은 계약을 새로 만들지 않는다. 코드 생성의 단일 출처가 둘이다.

```
$ rhwp export-ir-schema --json | python -c "import sys,json;d=json.load(sys.stdin);
print(d['irSchemaVersion'], d['definitionCount'], d['dialect'])"
1.0 41 https://json-schema.org/draft/2020-12/schema

$ rhwp export-capabilities-schema --json | python -c "import sys,json;d=json.load(sys.stdin);
print(d['capabilitiesSchemaVersion'], d['definitionCount'], list(d.keys()))"
1.1 19 ['capabilitiesSchemaVersion','definitionCount','dialect','mcpSchema','schema','schemaVersion']
```

`--bare` 를 주면 봉투 없이 스키마 본문만 나와 JSON Schema 도구에 바로 먹일 수 있다.
공식 Python·Node 바인딩은 v0.8.4에서 철회됐다([#4655](https://github.com/edwardkim/rhwp/issues/4655)).
외부 소비자는 `capabilities`의 `json:true` 선언과 위 스키마를 권위로 삼고, 자체 래퍼를
다운스트림에서 유지한다.

## 7. 판정 3층 — 실제 응답으로 읽는다

`isError` 하나만 보면 두 방향으로 틀린다. **성공을 실패로 읽거나(exit 3),
실패를 성공으로 읽는다(`notFound` 가 찬 exit 0).**

### 7-1. 1층 — JSON-RPC 오류: 요청 자체가 프로토콜에 맞지 않다

```json
→ {"jsonrpc":"2.0","id":2,"method":"tools/unknown","params":{}}
← {"id":2,"jsonrpc":"2.0","error":{"code":-32601,"message":"지원하지 않는 메서드: tools/unknown"}}
```

`result` 가 아예 없다. **호스트 구현 버그**이므로 재시도하지 않고 코드를 고친다.

### 7-2. 2층 — `isError:true`: 도구는 불렀지만 실행이 실패했다

네 가지를 실제로 찍어 봤다.

**(가) 필수 인자 누락 — 실행 전에 막힌다**

```json
→ {"name":"hwp_info","arguments":{}}
← {"isError":true,"content":[{"type":"text","text":"필수 인자 누락: path"}]}
```

**(나) 없는 파일 — 자식 CLI 의 exit 1 이 그대로 전달된다**

```json
→ {"name":"hwp_info","arguments":{"path":"samples/없는파일.hwp"}}
← {"isError":true,"content":[{"type":"text",
    "text":"종료 코드 1: 오류: 파일을 읽을 수 없습니다 - samples/없는파일.hwp: 지정된 파일을 찾을 수 없습니다. (os error 2)"}]}
```

**(다) 조립 오류 — exit 2. 같은 인자로 재시도하면 또 실패한다**

```json
→ {"name":"hwp_set_cell","arguments":{"path":"samples/table-001.hwp","table":0,"row":0,"col":2,"text":"x"}}
← {"isError":true,"content":[{"type":"text",
    "text":"종료 코드 2: 오류: (0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요."}]}
```

메시지에 **다음 수가 들어 있다**(`앵커 (0,1)`). exit 2 는 인자를 고쳐 다시 부른다.

**(라) 없는 도구 — 교정 후보가 함께 온다**

```json
→ {"name":"hwp_serach","arguments":{ … }}
← {"isError":true,"content":[{"type":"text","text":
    "{\"didYouMean\":[\"hwp_search\"],\"error\":\"알 수 없는 도구: hwp_serach\",
      \"nextCall\":{\"arguments\":{},\"name\":\"hwp_search\",\"why\":\"요청한 이름이 없음 — 가장 가까운 실존 도구로 교정\"}}"}]}
```

`nextCall.name` 을 그대로 다시 부르면 된다. 닫힌 핸들도 같은 모양으로 안내한다:

```json
→ {"name":"hwp_doc_search","arguments":{"docId":"doc-1","query":"결재"}}   # 이미 close 한 뒤
← {"isError":true,"content":[{"type":"text","text":
    "{\"error\":\"열려 있지 않은 핸들: doc-1 (hwp_open 먼저)\",
      \"nextCall\":{\"arguments\":{\"path\":\"<열 문서 경로>\"},\"name\":\"hwp_open\",
                    \"why\":\"핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도\"}}"}]}
```

CLI 에도 같은 힌트가 있다:

```
$ rhwp serach samples/table-001.hwp
오류: 알 수 없는 명령입니다 - serach
힌트: 가장 가까운 명령은 'search' 입니다

$ rhwp search samples/table-001.hwp --jsonn -- 품질
알 수 없는 옵션: --jsonn
힌트: 검색어가 '-' 로 시작한다면 `--` 뒤에 두세요 — rhwp search <파일> --json -- <검색어>
exit=2
```

### 7-3. 3층 — `isError:false` + 봉투 판정: 도구는 성공했고, 답이 부정적이다

**이 층이 가장 자주 오독된다.** 같은 `ir-diff` 가 CLI 에서는 exit 3, MCP 에서는
`isError:false` 다.

```
$ rhwp ir-diff samples/추진일정.hwp out/추진일정.hwpx --json
{"a":"samples/추진일정.hwp","b":"out/추진일정.hwpx",
 "categories":{"cc":1,"char_offsets[0]: A=32 vs B=16":1},
 "diffCount":2,"identical":false,"schemaVersion":"1.0",
 "untrustedContent":true,"untrustedFields":["categories"]}
exit=3
```

```json
→ {"name":"hwp_ir_diff","arguments":{"a":"samples/추진일정.hwp","b":"out/추진일정.hwpx"}}
← {"isError":false,"structuredContent":{"identical":false,"diffCount":2, …}}
```

`isError:false` 를 "문제 없음"으로 읽으면 **차이가 있는 변환을 통과시킨다.**

부정적 판정이 데이터로 오는 나머지 예:

| 상황 | 봉투 | 종료 코드 |
|---|---|---|
| 검색 0건 | `matchCount:0`·`totalMatchCount:0` | 0 |
| 치환 0건(출력 파일 없음) | `replacedCount:0`, `output` 키 자체가 없음 | 0 |
| 없는 필드 이름을 줌 | `notFound:["없는필드"]`, `filledCount` 는 나머지만 | 0 |
| 순번 없이 반복 필드를 줌 | `ambiguous:[{"name":"목차1","matched":1,"total":5}]` | 0 |
| CSV 행·열 불일치 | `invalid:[…]`, `changedCount:0` | **2** |
| 계획 선검증 위반 | `invalid:[{"step":1,"reason":"…"}]`, 실행 0 | **2** |
| 시각 회귀 검출 | `status:"OVER"`·`regression:true` | **3** |

### 7-4. 대응표

| CLI exit | MCP | 뜻 | 에이전트의 다음 수 |
|---|---|---|---|
| 0 | `isError:false` | 실행 성공 | **봉투 판정 필드를 마저 읽는다** |
| 1 | `isError:true` | 런타임 실패(파일·파싱·렌더) | 입력·환경을 고쳐 재시도 가능 |
| 2 | `isError:true` | 호출 조립 버그 | **같은 인자로 재시도 금지** — 인자를 고친다 |
| 3 | `isError:false` | 검증 단언 실패(판정) | `identical`·`regression`·`invalid` 를 읽고 판단 |
| 4 | `isError:false` | 쪽 수 불일치 | `verifyPages` 를 읽는다 |

## 8. 컨텍스트 예산 운용 — 싼 것부터, 좁혀서

`export-text` 로 시작하면 첫 호출에 컨텍스트가 날아간다. **387쪽 문서
(`samples/2025 행정업무운영 편람(최종).hwpx`, 13.6MB) 실측**:

| 호출 | stdout 바이트 | 배수 |
|---|---|---|
| `info --json` | **639** | ×1 |
| `digest --json` | **1,376** | ×2 |
| `digest --pages 0..2 --json` | 1,110 | ×1.7 |
| `export-text -p 30 --json` | 1,794 | ×2.8 |
| `extract-data --kind amount --json` | 1,845 | ×2.9 |
| `digest --sections --json` | 2,693 | ×4 |
| `search --max-matches 5 --json` | 3,015 | ×4.7 |
| `search --max-matches 50 --json` | 25,924 | ×41 |
| `export-text --max-chars 500 --json` | 23,684 | ×37 |
| `extract-data --json` (필터 없음) | 117,870 | ×184 |
| `search --json` (상한 없음) | 150,540 | ×236 |
| `export-structure --json` | 284,893 | ×446 |
| `export-text --json` | **637,085** | **×997** |
| `export-tables --json` | **759,031** | **×1,188** |

`digest` 1,376B 로 시작해 `export-text` 637,085B 를 피한다 — **463배**.

### 8-1. 순서

**① `digest` 로 문서의 성격과 다음 수를 받는다**

```
$ rhwp digest "samples/2025 행정업무운영 편람(최종).hwpx" --json
{"excerpt":"\n\n행정업무운영 편람\n이 편람은 1991년 …",
 "format":"hwpx","nextStep":"더 읽으려면 export-text --json -p <쪽>, 찾으려면 search --json",
 "outline":["제1장 행정업무 운영 개요\t 1","제2장 공문서 관리 등 행정업무의 처리\t 19",
            "제3장 행정업무의 효율적 수행\t 175","제4장 행정업무의 관리\t 243",
            "제5장 질의 및 답변\t 269"],
 "pageCount":387,"paraCount":2618,"schemaVersion":"1.0",
 "source":"samples/2025 행정업무운영 편람(최종).hwpx","truncated":false,
 "untrustedContent":true,"untrustedFields":["outline[]","excerpt"]}
```

`nextStep` 은 **문자열로 다음 호출을 알려 준다.** 소진하면 문구가 바뀐다:

```
$ rhwp digest … --pages 0..2 --json     → nextStep: "이어서 digest --json --pages 3..5"
$ rhwp digest … --pages 385..386 --json → nextStep: "범위 발췌 완료 — 더 찾으려면 search --json"
```

**② 구조가 필요하면 `--sections`, 원문은 `-p N`**

```
$ rhwp digest … --sections --json
{"sectionCount":5,"sectionsMode":"clause","truncated":true,
 "sections":[{"charCount":46,"excerpt":"1. ‘행정업무의 효율적 운영’의 의의\t 3\n2. 행정업무운영 제도의 발전과정\t 5",
              "page":3,"title":"제1장 행정업무 운영 개요\t 1"}, …]}
```

`sections[].page` 가 그대로 다음 `export-text -p` 의 인자다.

```
$ rhwp export-text "samples/2025 행정업무운영 편람(최종).hwpx" -p 3 --json
{"omittedCount":0,"pageCount":1,
 "pages":[{"page":3,"text":"\n\n제1장 행정업무 운영 개요\t 1\n1. ‘행정업무의 효율적 운영’의 의의\t 3\n …"}],
 "schemaVersion":"1.0","truncated":false,
 "untrustedContent":true,"untrustedFields":["pages[].text"]}
```

`-p` 를 주면 `pageCount` 는 **문서 쪽 수가 아니라 반환한 쪽 수(1)** 다. 문서 전체
쪽 수는 `info`·`digest` 에서 얻는다.

**③ 찾을 때는 `--max-matches` 로 상한을 건다**

```
$ rhwp search "samples/2025 …hwpx" --json --max-matches 3 -- 결재
{"caseSensitive":true,"matchCount":3,"omittedCount":310,"query":"결재",
 "totalMatchCount":313,"truncated":true,
 "matches":[{"charOffset":68,"context":"… 보고·결재 단계의 축소, 전자결재의 활성화 …",
             "length":2,"page":12,"paragraph":25,"section":1,"text":"…"} , … ],
 "untrustedContent":true,"untrustedFields":["matches[].text","matches[].context"]}
exit=0
```

**`totalMatchCount` 는 상한과 무관하게 문서 전체 수를 준다.** 313건 중 3건만 받고도
"전체가 313건"임을 안다 — 좁혀도 규모를 잃지 않는다.

**④ 값이 목적이면 `extract-data --kind` 로 축을 좁힌다**

필터 없이 117,870B, `--kind amount` 로 1,845B (64배 절감). 종류별 전체 개수는
좁혀도 `counts` 에 남는다.

**⑤ 표는 마지막에, 필요한 표만**

`export-tables --json` 은 이 문서에서 759KB 다. 표 하나만 필요하면
`table-to-csv --table N` 이 훨씬 싸고, 스프레드시트로 바로 넘길 수 있다.

### 8-2. 반복 조회는 세션으로 (실측 2.6배)

같은 문서를 여러 번 물으면 **파싱 비용이 매번 든다.**

```
# 세션: open + 검색 3회 + info + close (한 프로세스)
$ rhwp mcp-serve < session.ndjson      → 310 ms

# 무상태: 같은 검색 3회 (프로세스 3개, 매번 재파싱)
$ for q in 결재 공문서 기안; do rhwp search "…hwpx" --json --max-matches 1 -- $q; done
                                       → 810 ms
```

호출이 늘수록 격차가 커진다. **3회 이상 물을 문서면 세션을 연다.**

### 8-3. 배치 결과는 `structuredContent` 가 없다

```json
→ {"name":"hwp_batch","arguments":{"subcommand":"info","paths":["a.hwp","b.hwp"]}}
← {"isError":false,"structuredContent":null,
   "content":[{"type":"text","text":"{…}\n{…}"}]}
```

NDJSON 은 객체 하나가 아니라 여러 줄이므로 `structuredContent` 가 `null` 이다.
`content[0].text` 를 **줄 단위로** 파싱한다.

## 9. 편집 안전 절차 — `--dry-run` → `--verify` → `changedPages`

편집은 되돌릴 수 없다. 세 단계를 순서대로 밟는다.

### 9-1. 1단계 — `--dry-run`: 무엇이 바뀌는지 먼저 본다

```
$ rhwp edit fill-fields samples/field-01.hwp \
    --data '{"회사명":"페타플로","목차1":"개요","없는필드":"x"}' --dry-run --json
{"ambiguous":[{"matched":1,"name":"목차1","total":5}],
 "changedPages":null,"confusable":[],"dryRun":true,
 "filled":[{"name":"목차1","occurrence":0,"value":"개요"},
           {"name":"회사명","occurrence":0,"value":"페타플로"}],
 "filledCount":2,"notFound":["없는필드"],"schemaVersion":"1.0",
 "source":"samples/field-01.hwp","untrustedContent":false,"untrustedFields":[]}
exit=0
```

**exit 0 인데 두 개가 잘못됐다.** `notFound` 에 오타가 있고, `ambiguous` 는 `목차1`
이 5곳인데 1곳만 채웠다고 말한다. `--dry-run` 없이 돌렸다면 반쯤 채운 문서가
"성공"으로 저장됐을 것이다.

고치는 법 — 순번으로 지목한다:

```
$ rhwp edit fill-fields samples/field-01.hwp --data '{"목차1[2]":"세번째 항목"}' --dry-run --json
{"ambiguous":[],"changedPages":null,"confusable":[],"dryRun":true,
 "filled":[{"name":"목차1","occurrence":2,"value":"세번째 항목"}],
 "filledCount":1,"notFound":[], …}
```

`ambiguous:[]`·`notFound:[]` 가 **완료 조건**이다.

`--dry-run` 은 파일을 만들지 않는다. 넘침·병합 같은 사전 검사는 dry-run 에서도 돈다:

```
$ rhwp edit set-cell samples/table-001.hwp --table 0 --row 1 --col 0 \
    --text "아주아주 길고 긴 값을 넣어서 칸 폭을 확실히 넘기도록 만든 문자열입니다 계속 이어집니다" --dry-run --json
{"changedPages":null,"col":0,"dryRun":true,"keepStyle":false,
 "newText":"아주아주 길고 …","oldText":"품질관리협의체 운영계획 수립",
 "overflow":[{"cellWidthPx":196.99,"lines":4,"target":"table0[1,0]",
              "text":"아주아주 길고 …","textWidthPx":688.0}],
 "row":1,"table":0,"untrustedContent":true,"untrustedFields":["oldText"]}
exit=0
```

`overflow` 는 **막지 않고 알린다**(196.99px 칸에 688px 글, 4줄). 넘쳐도 좋은지는
호출자가 정한다.

되돌릴 수 없는 명령은 dry-run 을 **구조적으로 강제**한다:

```
$ rhwp edit redact samples/복학원서.hwp --json
오류: 마스킹은 되돌릴 수 없습니다. 산출 경로를 -o <출력> 으로 지정하거나,
      원본을 덮어쓸 의도라면 --in-place 를 명시하세요
      (먼저 --dry-run 으로 무엇이 지워질지 확인하기를 권합니다).
exit=2
```

> `edit redact --dry-run` 의 `findings[].raw` 에는 **원문 개인정보가 그대로** 들어
> 있다. 로그·전송·컨텍스트에 남기지 않는다.

### 9-2. 2단계 — `--verify`: 저장한 것이 의도한 것인지 기계가 판정한다

`--verify` 를 주지 않으면 `verify` 는 **`null`** 이다 — 통과가 아니라 **검증을 안 한
것**이다.

```
$ rhwp edit fill-fields samples/field-01.hwp \
    --data '{"회사명":"페타플로","작성자":"홍길동"}' -o out/field-filled.hwp --json
{ … "output":"out/field-filled.hwp","outputFormat":"hwp5","verify":null}
```

붙이면 판정이 온다:

```
$ rhwp export-hwpx samples/추진일정.hwp out/추진일정.hwpx --verify --json
{"bytes":9852,"format":"hwpx","output":"out/추진일정.hwpx","passwordProtected":false,
 "schemaVersion":"1.0","source":"samples/추진일정.hwp",
 "verify":{"diffCount":0,"identical":true},"verifyPages":null}
exit=0
```

**`--verify` 통과가 "완전히 같다"는 뜻은 아니다.** 같은 쌍을 `ir-diff` 로 다시 보면
차이가 나온다:

```
$ rhwp ir-diff samples/추진일정.hwp out/추진일정.hwpx --json
{"categories":{"cc":1,"char_offsets[0]: A=32 vs B=16":1},"diffCount":2,"identical":false, …}
exit=3
```

두 판정은 **같은 대상을 보지 않는다.** `--verify` 는 저장 직후 산출물을 재파싱해
자기 자신과 대조하는 게이트이고, `ir-diff` 는 두 파일을 직접 비교한다. 무손실이
계약이면 **둘 다** 돌리고 `categories` 를 읽어 판단한다.

`run` 은 단언을 계획서에 넣어 **통과할 때만 저장**한다:

```
$ cat out/plan.json
{"planVersion":"1.0","input":"samples/field-01.hwp","output":"out/plan_result.hwp",
 "steps":[{"action":"fill_fields","data":{"회사명":"페타플로","작성자":"홍길동"}},
          {"action":"fill_fields","data":{"목차1[0]":"개요"}}],
 "assertions":{"verify":true}}

$ rhwp run out/plan.json --json
{"assertions":{"notFoundEmpty":true,"verify":true},"changedPages":[0,1],
 "input":"samples/field-01.hwp","output":"out/plan_result.hwp","outputFormat":"hwp5",
 "planVersion":"1.0",
 "steps":[{"action":"fill_fields","ambiguous":[],"confusable":[],
           "filled":[{"name":"작성자","occurrence":0,"value":"홍길동"},
                     {"name":"회사명","occurrence":0,"value":"페타플로"}],
           "filledCount":2,"notFound":[],"step":0},
          {"action":"fill_fields", … ,"filledCount":1,"step":1}],
 "verify":{"diffCount":0,"identical":true}}
exit=0
```

계획이 잘못되면 **한 step 도 실행하지 않고** 위반을 전부 모아 보고한다:

```
$ rhwp run out/plan.json --dry-run --json   # replace_text 대상이 0건인 계획
{"input":"samples/field-01.hwp",
 "invalid":[{"action":"replace_text","reason":"'여기에 입력' 일치 0건 — 치환할 곳이 없습니다","step":1}],
 "output":"out/plan_result.hwp","planVersion":"1.0","schemaVersion":"1.0"}
exit=2
```

하나 고치면 다음이 나오는 두더지잡기를 피하도록 **위반을 한 번에** 준다.

### 9-3. 3단계 — `changedPages`: 바뀐 쪽만 눈으로 본다

편집 봉투는 재조판 후 **0 기준 쪽 번호**를 준다.

```
$ rhwp edit replace-text samples/hwp3-sample.hwp --find 의 --replace 의 -o out/rep1.hwp --json
{"changedPages":[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14],"replacedCount":276, …}

$ rhwp edit replace-text samples/hwp3-sample.hwp --find 의 --replace ★ --occurrence 3 -o out/rep2.hwp --json
{"changedPages":[0],"occurrence":3,"replacedCount":1, …}
```

전건 치환은 15쪽에 걸치고, `--occurrence 3` 은 0쪽 하나뿐이다. **그 쪽만 렌더한다:**

```
$ rhwp export-svg out/rep2.hwp -o out/svg -p 0 --json
{"format":"svg","outputDir":"out/svg","pageCount":1,
 "pages":[{"bytes":82086,"overflowCellLines":0,"page":0,"path":"out/svg\\추진일정.svg"}],
 "renderedCount":1, …}
```

`changedPages:null` 이면 **확정 불가**다 — 전체를 봐야 한다. dry-run 은 항상 `null`
이므로, 눈검증은 실제 저장 후에 한다.

세션 안에서는 이 루프가 상수 비용으로 닫힌다:

```json
→ hwp_doc_fill_fields {"docId":"doc-1","data":{"회사명":"페타플로","작성자":"홍길동"}}
← {"changedPages":[0],"filledCount":2,"notFound":[], … }
→ hwp_doc_render_page {"docId":"doc-1","page":0,"output":"out/sess_p0"}
← {"bytes":351027,"format":"svg","output":"out/sess_p0","page":0}
→ hwp_doc_save {"docId":"doc-1","output":"out/sess_saved.hwp","verify":true}
← {"bytes":473600,"outputFormat":"hwp5","verify":{"diffCount":0,"identical":true}}
```

**`hwp_doc_save` 전에는 디스크가 바뀌지 않는다.** 저장 후에도 핸들은 열려 있어
이어서 편집·재저장할 수 있다.

### 9-4. 4단계(선택) — 표본을 늘려 회귀를 본다

대량 작업은 `batch fill` 을 dry-run 으로 먼저 전건 돌린다.

```
$ rhwp batch fill --form samples/field-01.hwp --data out/rows.jsonl \
    --out-dir out/merge --name-field 회사명 --dry-run --json
{"dryRun":true,"filledCount":3,"notFound":[],"output":"out/merge\\페타플로.hwp","row":0, …}
{"dryRun":true,"filledCount":3,"notFound":[],"output":"out/merge\\가나다.hwp","row":1, …}
{"dryRun":true,"filledCount":2,"notFound":["없는필드"],"output":"out/merge\\라마바.hwp","row":2, …}
batch fill: 3행 중 3 성공, 0 실패 (2ms, threads=32, dry-run)
exit=0
```

세 번째 행에 `notFound` 가 있는데 **exit 0** 이다 — 데이터 품질 문제는 실행 실패가
아니라 레코드 판정이다. 행 단위로 `notFound` 를 확인하지 않으면 조용히 덜 채운
산출물 100개가 나온다.

## 10. 흔한 실패와 그 자리에서의 처방

### 10-1. `--json` 을 붙였는데 파싱이 깨진다

**증상**: JSON 앞뒤에 사람이 읽는 줄이 섞여 있다.

**원인**: stdout 이 아니라 **stdout+stderr 를 함께 캡처**했다. 진단·진행·요약은
전부 stderr 로 간다.

```
$ rhwp extract-pages "samples/2025 …hwpx" out/p13.hwp --from 14 --to 14 --json
CONVERGENCE: sec1 page 6 수렴 확인 (1페이지 재사용 가능)      ← stderr
CONVERGENCE: sec1 page 5 수렴 확인 (1페이지 재사용 가능)      ← stderr
{"from":14,"output":"out/p13.hwp","pagesAfter":15,"pagesBefore":387, …}   ← stdout
```

**처방**: `2>/dev/null` 로 분리하거나, 파이프에서 stderr 를 따로 받는다. 암호 문서는
stderr 에 레이아웃 경고가 대량으로 나오므로 특히 중요하다.

### 10-2. 실패했는데 stdout 을 파싱하려 한다

**증상**: 빈 문자열을 JSON 으로 파싱하다 예외.

**계약**: 단건 명령이 실패하면 stdout 은 **0바이트**다(부분 매니페스트 금지).

```
$ rhwp edit set-cell samples/table-001.hwp --table 0 --row 0 --col 2 --text x --dry-run --json 2>/dev/null > out/err.json
exit=2  stdout_bytes=0
```

**처방**: **종료 코드를 먼저 보고** 0/3/4 일 때만 파싱한다. 단, 예외가 있다 —
`run` 과 `csv-to-table` 은 **exit 2 에서도 봉투를 낸다**(`invalid[]` 를 전달해야
하므로). `run` 의 계획서 형식 오류도 마찬가지다:

```
$ rhwp run out/plan.json --json    # planVersion 누락
{"error":"planVersion \"1.0\" 이 필요합니다","schemaVersion":"1.0", …}
exit=2   stdout_bytes=120
```

**규칙**: exit 2 라고 stdout 을 버리지 말고, **비어 있지 않으면 읽어 본다.**

### 10-3. batch 가 exit 1 인데 결과는 나왔다

```
$ rhwp batch info --json < out/list.txt
{"format":"hwp5","pageCount":1, … "source":"samples/table-001.hwp"}
{"format":"hwp5","pageCount":3, … "source":"samples/field-01.hwp"}
{"error":"파일을 읽을 수 없습니다: 지정된 파일을 찾을 수 없습니다. (os error 2)",
 "exitClass":"runtime","schemaVersion":"1.0","source":"samples/없는파일.hwp"}
batch: 3건 중 2 성공, 1 실패 (3ms, threads=32)      ← stderr
exit=1
```

**계약**: 한 건이라도 실패하면 최종 exit 1 이지만 **스트림은 끝까지 흐른다.**
집계 규칙은 `error` 있으면 1, 없고 `verifyPages` 불일치면 4, `verify` 차이만 있으면
3, 전부 통과면 0.

**처방**: 종료 코드로 전체를 버리지 말고 **레코드별 `error`/`exitClass` 로 재시도
대상을 고른다.** `source` 가 어느 줄이 어느 파일인지 잇는 유일한 키다.

### 10-4. `filledCount` 가 나왔는데 서식이 덜 찼다

§9-1 참조. **`filledCount` 는 성공 판정이 아니다.** 완료 조건은
`notFound == [] && ambiguous == []` 다. 반복 필드는 `이름[N]`(0 기준)으로 지목한다.

### 10-5. 표 좌표가 계속 튕긴다

세 가지를 순서대로 확인한다.

| 확인 | 방법 |
|---|---|
| 표 번호가 0부터인가 | `export-tables --json` 의 `tables[].index` — **0부터가 아닐 수 있다** |
| 병합으로 덮인 칸인가 | 실패 메시지가 앵커를 알려 준다: `(0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요.` |
| 중첩 표인가 | `cells[].nested` 안의 표는 **v1 범위 밖**이다: `본문 최상위 표 0 번이 없습니다 (최상위 표 0개; 중첩 표는 v1 범위 밖)` |

CSV 로 통째로 바꿀 때는 행·열이 정확히 같아야 한다. 다르면 **한 칸도 쓰지 않는다**:

```
$ rhwp csv-to-table samples/table-001.hwp --csv out/bad.csv --table 0 --dry-run --json
{"changed":[],"changedCount":0,"colCount":9,"rowCount":19,
 "invalid":[{"actual":2,"expected":19,"reason":"rowCountMismatch",
             "message":"CSV 행 수 2 가 표 0 의 행 수 19 와 다릅니다 — 표 크기는 바꾸지 않습니다."},
            {"actual":2,"expected":9,"reason":"colCountMismatch","row":0, …},
            {"actual":2,"expected":9,"reason":"colCountMismatch","row":1, …}]}
exit=2
```

**처방**: `table-to-csv --table N` 으로 **뽑은 CSV 를 고쳐서** 되돌린다. 직접 만들지
않는다 — 병합 칸이 채워진 격자라 손으로 맞추기 어렵다.

### 10-6. 쪽 번호가 한 쪽씩 밀린다

**원인**: `extract-pages` 의 `--from`/`--to` 만 **1 기준**이고, 나머지 축은 전부
0 기준이다.

**처방**: `search` 가 `page:13` 을 줬으면 `--from 14 --to 14`.

**그리고 결과 쪽 수를 확인한다** — 자르기는 쪽 단위로 판정하고 문단 단위로 지운다:

```
$ rhwp extract-pages "samples/2025 …hwpx" out/p13.hwp --from 14 --to 14 --json
{"from":14,"to":14,"pagesBefore":387,"pagesAfter":15,
 "paragraphsKept":21,"paragraphsRemoved":2597}
```

한 쪽을 지정했는데 `pagesAfter:15` 다. 남은 문단이 재조판되며 퍼졌다. 필요하면 다시
좁힌다.

### 10-7. 봉투에 `untrustedContent` 가 없다

**증상**: 표지를 읽는 파서가 키 부재에서 죽거나, `false` 로 단정해 문서 파생 값을
신뢰한다.

**실측(§3-1)**: `edit redact`·`edit sanitize`·`edit insert-image`·`run --dry-run`·
`export-ir-schema`·`export-capabilities-schema` 6종에 키가 없다. 그중 앞의 셋은
문서 파생 값(`findings[].raw`·`removed[].before`·`preview[].targets[].name`)을
실제로 싣는다.

**처방(소비자)**: 키 부재를 `false` 로 읽지 말고 **"미표기"** 로 다룬다. 미표기
봉투는 보수적으로 문서 파생으로 취급한다. 명령별 정본 지도는
`rhwp export-provenance-map --json` 의 `commands.<명령>.untrusted[]`.

**처방(구현자)**: 새 봉투를 만들 때 §3 의 출처 표지 항목을 체크리스트로 쓴다.

### 10-8. 암호 문서를 못 연다

```
$ rhwp info samples/HWP5-password-123456.hwpx --json
오류: 비밀번호가 필요한 암호 문서입니다 (--password <pw> 로 전달).
exit=2
```

**처방**: `--password-stdin` 을 쓴다(전역 옵션이라 **명령 앞**에 온다).

```
$ printf '123456\n' | rhwp --password-stdin info samples/HWP5-password-123456.hwpx --json 2>/dev/null
{"fonts":[…],"format":"hwpx","pageCount":23,"paraCount":365,"sizeBytes":110701,
 "title":"한글과컴퓨터","version":"5.1.0.0", …}
```

`--password <pw>` 는 프로세스 목록에 노출되므로 **stdin 을 쓴다.** MCP 에서는 도구
인자 `password` 를 준다 — 서버는 응답·세션에 저장하지 않고 자식 CLI 의 stdin 으로만
넘긴다.

```json
→ {"name":"hwp_info","arguments":{"path":"samples/…password-123456.hwp","password":"123456"}}
← {"isError":false,"structuredContent":{"format":"hwp5","pageCount":64, … }}
→ {"name":"hwp_info","arguments":{"path":"samples/…password-123456.hwp"}}     # 암호 없이
← {"isError":true,"content":[{"type":"text","text":"종료 코드 2: 오류: 비밀번호가 필요한 암호 문서입니다 …"}]}
```

**batch 는 암호를 지원하지 않는다** — `--password*` 를 주면 usage error 다.

### 10-9. 렌더 파일 경로가 안 맞는다

**증상**: 매니페스트의 `path` 로 파일을 못 연다.

**원인**: Windows 에서 `outputDir` 와 파일명이 `\` 로 이어진다(실측:
`"out/svg\\추진일정.svg"`, `"out/merge\\페타플로.hwp"`). 앞부분은 `/`, 이음새만
`\` 인 혼합 경로다.

**처방**: 경로를 문자열로 비교하지 말고 정규화하거나 basename 으로 다룬다.

### 10-10. 썸네일 확장자와 실제 형식이 다르다

```
$ rhwp thumbnail samples/20250130-hongbo.hwp -o out/thumb.png --json
{"bytes":3569,"format":"gif","height":250,"mime":"image/gif","output":"out/thumb.png","width":177}
```

`-o` 로 `.png` 를 줬지만 **내장 미리보기의 실제 형식은 GIF** 다. 확장자를 믿지 말고
`mime`/`format` 을 읽는다.

### 10-11. 치환했는데 출력 파일이 없다

```
$ rhwp edit replace-text samples/hwp3-sample.hwp --find 존재하지않는문자열ZZZ --replace X -o out/rep0.hwp --json
{"caseSensitive":true,"changedPages":null,"dryRun":false,"find":"존재하지않는문자열ZZZ",
 "occurrence":null,"replace":"X","replacedCount":0, … }
exit=0    파일존재=no
```

**계약**: 치환 0건이면 출력 파일을 만들지 않고, 봉투에 `output` 키도 없다.
후속 단계가 `output` 을 무조건 읽으면 여기서 깨진다. **`replacedCount > 0` 을 먼저
확인한다.**

### 10-12. 보안 조사가 아무것도 안 잡는다

`samples/` 의 문서는 `inspect` 세 축 모두 `clean:true` 다. 이것은 **정상**이다 —
`samples/` 는 오탐 0을 지키는 음성 코퍼스이고, 양성 표본은 계약 테스트가 실행 중에
합성한다([코퍼스 설계](../tech/agent_security/test_corpus.md)).

탐지기가 실제로 돌았는지는 **작업량 필드**로 확인한다:

```
$ rhwp inspect unicode samples/hwp3-sample.hwp --json
{"clean":true,"findingCount":0,
 "kindCounts":{"bidi_override":0,"confusable":0,"tag_char":0,"zero_width":0},
 "kindFilter":"all","scannedChars":20736,
 "severityCounts":{"high":0,"low":0,"medium":0}, … }
```

`scannedChars:20736` 이 "훑었는데 없었다"의 증거다. 0이면 탐지기가 아니라 **입력**을
의심한다.

주입 조사는 기본 9축만 본다. 누름틀 이름·안내문·메모까지 보려면 `--include-fields`:

```
$ rhwp inspect injection samples/field-01.hwp --json --include-fields
{"clean":true,"highestConfidence":null,"includeFields":true,"minConfidence":"low",
 "scanScopes":["body","tableCell","textBox","equation","footnote","endnote","header","footer",
               "caption","fieldName","fieldGuide","fieldCommand","hiddenComment","fieldMemo"],
 "signalCount":0, … }
```

`highestConfidence:null` 은 신호 0건이라는 뜻이다(§7-3 의 "부정적 판정은 데이터").

## 11. 표면을 굴리기 전 자기점검

새 환경에 붙일 때 이 여섯 줄을 돌리면 계약이 살아 있는지 확인된다.

```
rhwp capabilities | python -c "import sys,json;d=json.load(sys.stdin);print(d['version'],len(d['commands']))"
rhwp capabilities --mcp | python -c "import sys,json;print(len(json.load(sys.stdin)['tools']))"
rhwp info <표본> --json 2>/dev/null | python -c "import sys,json;print(json.load(sys.stdin)['pageCount'])"
rhwp digest <표본> --json 2>/dev/null | wc -c        # 1~3KB 면 정상
rhwp export-provenance-map --json 2>/dev/null | python -c "import sys,json;print(len(json.load(sys.stdin)['commands']))"
rhwp <오타명령> ; echo $?                              # 2 + 교정 힌트가 나와야 한다
```

기대값(v0.8.2): `0.8.2 61` / `39` / 표본의 쪽 수 / 1,376B(387쪽 편람) / `31` / `2`.

어긋나면 이 문서가 아니라 **바이너리가 기준**이다 — 어긋난 쪽을 고친다.

## 12. 더 읽을 것

- 표면 전체 지도와 봉투 필드 사전: [에이전트 지식 지도](agent_knowledge_map.md)
- 명령·플래그의 최종 권위: [CLI 명령어 매뉴얼](cli_commands.md)
- 증상별 실패 사전: [문제해결 가이드](agent_troubleshooting_guide.md)
- 스크립트로 엮기: [JSON 파이프라인 가이드](cli_json_pipeline_guide.md)
- 호스트에 붙이기: [MCP 통합 가이드](mcp_integration_guide.md)
- 서식 채움 심화: [서식 가이드](form_filling_guide.md)
- 파괴적 편집 전 선검사: [선검사 가이드](agent_preflight_guide.md)
- 문서가 에이전트를 조종하는 경로: [에이전트 보안 문서 지도](../tech/agent_security/README.md)
