use super::{tool, tool_with_optional_args};

pub(super) fn extend(tools: &mut Vec<serde_json::Value>) {
    tools.extend([
        tool_with_optional_args(
            "hwp_batch",
            "여러 문서를 한 프로세스에서 병렬 처리해 NDJSON 스트림으로 받는다. 파일 목록은 stdin 으로 한 줄에 하나씩 넣는다. 읽기 전용 5축만 제공하며, 파일을 쓰는 batch convert 는 CLI 전용이다. 아카이브 전체를 스윕할 때 쓴다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "subcommand": {
                        "type": "string",
                        "enum": ["export-text", "info", "export-structure", "export-tables", "fields"],
                        "description": "각 파일에 적용할 처리"
                    },
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "처리할 문서 경로 목록 (stdin 으로 전달된다)"
                    },
                    "threads": { "type": "integer", "minimum": 1, "description": "병렬 스레드 수. 기본은 CPU 코어 수" }
                },
                "required": ["subcommand", "paths"],
            }),
            "batch",
            serde_json::json!(["batch", "{subcommand}", "--json"]),
            serde_json::json!([
                { "when": "threads", "args": ["--threads", "{threads}"] }
            ]),
            &["schemaVersion", "source", "error", "exitClass"],
        ),
        tool_with_optional_args(
            "hwp_fill_fields",
            "HWP 서식(템플릿)의 누름틀에 값을 채워 새 문서를 만든다. 먼저 hwp_fields 로 어떤 필드가 있는지 확인한 뒤 사용한다. 같은 이름이 여러 번 나오는 서식(규제영향분석서 등)은 이름에 순번을 붙여 지목한다. dryRun 으로 파일을 만들지 않고 변경 예정만 확인할 수 있다. 산출물은 입력 형식을 따른다(HWPX 입력 → HWPX 산출).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "입력 HWP/HWPX 문서 경로" },
                    "data": {
                        "type": "object",
                        "additionalProperties": { "type": "string" },
                        "description": "{\"필드이름\":\"값\"} 형태의 채울 값. 같은 이름이 여러 번 나오면 \"이름[N]\"(0 기준 순번, hwp_fields 목록 순서)으로 N 번째를 지목한다. 순번 없이 주면 첫 번째만 채우고 응답의 ambiguous 에 몇 개 중 몇 개인지 보고한다."
                    },
                    "output": { "type": "string", "description": "출력 파일 경로. 생략하면 <입력명>_filled.hwp (HWPX 입력이면 _filled.hwpx)" },
                    "dryRun": { "type": "boolean", "description": "true 면 파일을 쓰지 않고 변경 예정만 보고" }
                },
                "required": ["path", "data"],
            }),
            "edit",
            serde_json::json!(["edit", "fill-fields", "{path}", "--data", "{data}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &[
                "schemaVersion",
                "source",
                "dryRun",
                "filledCount",
                "filled",
                "notFound",
                "ambiguous",
                "confusable",
                "output",
                "outputFormat",
                "changedPages",
            ],
        ),
        tool_with_optional_args(
            "hwp_batch_search",
            "여러 문서를 한 프로세스에서 병렬 검색해 NDJSON 스트림으로 받는다. 매치마다 구역·문단·페이지 주소가 붙어 '어느 문서 몇 쪽'을 답할 수 있다. 파일 목록은 stdin 으로 한 줄에 하나씩 넣는다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "찾을 문자열 (대소문자 구분)" },
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "검색할 문서 경로 목록 (stdin 으로 전달된다)"
                    },
                    "threads": { "type": "integer", "minimum": 1, "description": "병렬 스레드 수. 기본은 CPU 코어 수" }
                },
                "required": ["query", "paths"],
            }),
            "batch",
            serde_json::json!(["batch", "search", "--json", "--query", "{query}"]),
            serde_json::json!([
                { "when": "threads", "args": ["--threads", "{threads}"] }
            ]),
            &[
                "schemaVersion",
                "source",
                "query",
                "matchCount",
                "totalMatchCount",
                "truncated",
                "matches",
            ],
        ),
        // [#3830] 여러 문서에 걸친 날짜·금액·수량 추출 — hwp_extract_data 가 문서 하나에
        // 대해 하는 일을 아카이브 전체에 대해 한다. --query 가 필수라 hwp_batch 로는 부를
        // 수 없는 hwp_batch_search 와 같은 이유로 전용 도구다(kind·limit 은 선택이지만
        // paths 는 stdin 축이라 마찬가지로 전용 도구로 분리한다).
        tool_with_optional_args(
            "hwp_batch_extract_data",
            "여러 문서에서 날짜·금액·수량을 한 프로세스에서 병렬로 뽑아 NDJSON 스트림으로 받는다. 레코드마다 단건 hwp_extract_data 와 같은 봉투(items·counts·totalItemCount)가 실린다. 파일 목록은 stdin 으로 한 줄에 하나씩 넣는다. limit 은 배치 전체가 아니라 문서마다 적용되는 상한이다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "처리할 문서 경로 목록 (stdin 으로 전달된다)"
                    },
                    "kind": {
                        "type": "string",
                        "enum": ["date", "amount", "number", "all"],
                        "description": "뽑을 종류. 기본 all"
                    },
                    "limit": { "type": "integer", "minimum": 1, "description": "문서당 최대 반환 건수(컨텍스트 절약, 배치 전체가 아니라 문서마다 적용). 총량은 totalItemCount 로 온다" },
                    "threads": { "type": "integer", "minimum": 1, "description": "병렬 스레드 수. 기본은 CPU 코어 수" }
                },
                "required": ["paths"],
            }),
            "batch",
            serde_json::json!(["batch", "extract-data", "--json"]),
            serde_json::json!([
                { "when": "kind", "args": ["--kind", "{kind}"] },
                { "when": "limit", "args": ["--limit", "{limit}"] },
                { "when": "threads", "args": ["--threads", "{threads}"] }
            ]),
            &[
                "schemaVersion",
                "source",
                "kind",
                "itemCount",
                "totalItemCount",
                "truncated",
                "counts",
                "items",
                "error",
                "exitClass",
            ],
        ),
        // [#3719 §6-6] 진짜 메일머지. hwp_fill_fields 는 서식 1 → 산출 1 이라, 100명분을
        // 만들려면 도구를 100번 부르고 그 사이 상태를 에이전트가 들고 있어야 한다. 이
        // 도구는 서식 1 + 데이터 N행 → 산출 N개를 한 번의 호출로 끝낸다.
        tool_with_optional_args(
            "hwp_batch_fill",
            "서식 하나에 데이터 여러 행을 채워 산출 문서 N개를 한 번에 만든다 (메일머지). 데이터는 .jsonl(한 줄 한 객체) 또는 .csv(첫 줄 헤더 = 누름틀 이름) **파일 경로**로 준다 — 다른 batch 도구와 달리 stdin 파일 목록이 아니다. 먼저 hwp_fields 로 서식의 필드 이름을 확인한다. 응답은 행마다 한 줄인 NDJSON 이며, 실패한 행도 error 레코드로 남으므로 처리 누락을 셀 수 있다. dryRun 으로 파일을 만들지 않고 각 행이 채워지는지만 선검증할 수 있다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "form": { "type": "string", "description": "서식 HWP/HWPX 문서 경로 (누름틀이 있는 템플릿 1개)" },
                    "data": { "type": "string", "description": "데이터 행 파일 경로. .jsonl 이면 한 줄에 {\"필드이름\":\"값\"} 객체 하나, .csv 면 첫 줄 헤더가 누름틀 이름(BOM·따옴표 허용)" },
                    "outDir": { "type": "string", "description": "산출 문서를 모을 폴더. 없으면 만든다" },
                    "nameField": { "type": "string", "description": "산출 파일 이름으로 쓸 데이터 필드 이름. 생략하면 0001·0002 순번. 파일명 금지 문자는 _ 로 치환하고 이름이 겹치면 뒤에 _2 를 붙인다" },
                    "verify": { "type": "boolean", "description": "true 면 행마다 저장 직후 자기검증(저장본 재파싱 IR 대조). 차이가 있으면 CLI 종료 코드 3" },
                    "dryRun": { "type": "boolean", "description": "true 면 파일을 쓰지 않고 각 행이 채울 수 있는지만 판정" }
                },
                "required": ["form", "data", "outDir"],
            }),
            "batch",
            serde_json::json!(["batch", "fill", "--json", "--form", "{form}", "--data", "{data}", "--out-dir", "{outDir}"]),
            serde_json::json!([
                { "when": "nameField", "args": ["--name-field", "{nameField}"] },
                { "when": "verify", "args": ["--verify"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &[
                "schemaVersion",
                "source",
                "row",
                "dryRun",
                "output",
                "outputFormat",
                "filledCount",
                "filled",
                "notFound",
                "ambiguous",
                "confusable",
                "changedPages",
                "verify",
                "error",
                "exitClass",
            ],
        ),
        tool_with_optional_args(
            "hwp_replace_text",
            "HWP 문서 전체에서 문자열을 일괄 치환해 새 문서를 만든다 (기관명 변경·연도 갱신·용어 정비). dryRun 으로 파일을 만들지 않고 치환 예정 건수만 확인할 수 있다. 치환 0건이면 출력 파일을 만들지 않는다. 산출물은 입력 형식을 따른다(HWPX 입력 → HWPX 산출).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "입력 HWP/HWPX 문서 경로" },
                    "find": { "type": "string", "description": "찾을 문자열 (빈 문자열 불가)" },
                    "replace": { "type": "string", "description": "바꿀 문자열 (빈 문자열이면 삭제)" },
                    "output": { "type": "string", "description": "출력 파일 경로. 생략하면 <입력명>_replaced.hwp (HWPX 입력이면 _replaced.hwpx)" },
                    "dryRun": { "type": "boolean", "description": "true 면 파일을 쓰지 않고 치환 예정 건수만 보고" }
                },
                "required": ["path", "find", "replace"],
            }),
            "edit",
            serde_json::json!(["edit", "replace-text", "{path}", "--find", "{find}", "--replace", "{replace}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "find", "replace", "caseSensitive", "dryRun", "replacedCount", "output", "outputFormat", "changedPages"],
        ),
        tool(
            "hwp_set_checkbox",
            "실물 양식의 k번째(0 기준, hwp_search 문서 순서) 체크박스 문자를 체크한다(기본 □→☑). 전량 치환이 아니라 지정한 하나만 바꾼다 — 정부 서식의 해당 항목 체크용. 산출물은 입력 형식을 따른다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "입력 HWP/HWPX 문서 경로" },
                    "occurrence": { "type": "integer", "minimum": 0, "description": "몇 번째 □ 인가 (0 기준, hwp_search 로 확인)" },
                    "output": { "type": "string", "description": "출력 경로" }
                },
                "required": ["path", "occurrence", "output"],
            }),
            "edit",
            serde_json::json!(["edit", "replace-text", "{path}", "--find", "□", "--replace", "☑", "--occurrence", "{occurrence}", "-o", "{output}", "--json"]),
            &["schemaVersion", "source", "find", "replace", "occurrence", "dryRun", "replacedCount", "output", "outputFormat", "changedPages"],
        ),
        tool_with_optional_args(
            "hwp_set_cell",
            "HWP 표의 격자 좌표(hwp_export_tables 와 동일)로 셀 값을 바꿔 새 문서를 만든다 — 누름틀 없는 실물 표 양식 채우기. 먼저 hwp_export_tables 로 좌표를 확인한 뒤 사용한다. 병합으로 덮인 칸은 앵커 좌표를 안내하며 실패한다. 산출물은 입력 형식을 따른다(HWPX 입력 → HWPX 산출).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "입력 HWP/HWPX 문서 경로" },
                    "table": { "type": "integer", "minimum": 0, "description": "본문 최상위 표 번호 (export-tables 의 index)" },
                    "row": { "type": "integer", "minimum": 0, "description": "행 (0부터)" },
                    "col": { "type": "integer", "minimum": 0, "description": "열 (0부터)" },
                    "text": { "type": "string", "description": "셀에 넣을 값 (빈 문자열이면 비우기)" },
                    "output": { "type": "string", "description": "출력 파일 경로. 생략하면 <입력명>_cell.hwp (HWPX 입력이면 _cell.hwpx)" },
                    "dryRun": { "type": "boolean", "description": "true 면 파일을 쓰지 않고 old→new 만 보고" }
                },
                "required": ["path", "table", "row", "col", "text"],
            }),
            "edit",
            serde_json::json!(["edit", "set-cell", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--text", "{text}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "oldText", "newText", "dryRun", "overflow", "output", "outputFormat", "changedPages"],
        ),
        tool_with_optional_args(
            "hwp_insert_text_in_cell",
            "표 셀 문단의 문자 오프셋에 텍스트를 끼운다. set-cell 과 달리 칸 전체를 덮지 않는다. 코어 insert_text_in_cell_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "text": { "type": "string" },
                    "offset": { "type": "integer", "minimum": 0 },
                    "cellPara": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row", "col", "text"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-text-in-cell", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--text", "{text}", "--json"]),
            serde_json::json!([
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "cellPara", "args": ["--cell-para", "{cellPara}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "cellPara", "offset", "text", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_text_in_cell",
            "표 셀 문단의 문자 오프셋에서 글자를 지운다. 코어 delete_text_in_cell_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "count": { "type": "integer", "minimum": 1 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "cellPara": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row", "col", "count"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-text-in-cell", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--count", "{count}", "--json"]),
            serde_json::json!([
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "cellPara", "args": ["--cell-para", "{cellPara}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "cellPara", "offset", "count", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_export_ir_schema",
            "[#3762] 공개 문서 IR 의 JSON Schema 를 돌려준다. capabilities 가 *명령 표면*의 자기서술이라면 이것은 *문서 모델*의 자기서술이다 — 표·문단·누름틀·컨트롤이 어떤 모양인지 기계가 읽을 수 있다. 문서를 입력으로 받지 않는다(타입의 서술이지 특정 문서의 속성이 아니다). 외부 바인딩·코드 생성기가 단일 출처로 쓴다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "bare": {
                        "type": "boolean",
                        "description": "참이면 봉투 없이 스키마 본문만 (JSON Schema 도구에 바로 먹일 때)"
                    }
                },
                // 문서를 받지 않으므로 필수 인자가 없다 — 그래도 빈 배열을 선언한다.
                // 소비자가 required 의 부재와 "필수 없음"을 구분할 수 없으면 안 된다.
                "required": [],
            }),
            "export-ir-schema",
            serde_json::json!(["export-ir-schema", "--json"]),
            serde_json::json!([{ "when": "bare", "args": ["--bare"] }]),
            &["schemaVersion", "irSchemaVersion", "dialect", "definitionCount", "schema"],
        ),
    ]);
}
