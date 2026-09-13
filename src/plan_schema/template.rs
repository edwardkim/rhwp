//! Template request schema. Native serde requests and behavioral contracts are authoritative.
use super::*;

pub(super) const ACTIONS: [&str; 4] = [
    "fill_template",
    "repeat_and_fill_paragraph_block",
    "repeat_and_fill_table_rows",
    "import_paragraph_block",
];

pub(super) fn extend(defs: &mut serde_json::Map<String, Value>) {
    let mut limits = serde_json::Map::new();
    for (name, max) in [
        ("maxCopies", 1000),
        ("maxParagraphs", 10000),
        ("maxNodes", 100000),
        ("maxDepth", 64),
        ("maxStructureBytes", 33554432),
        ("maxMappingBytes", 8388608),
        ("maxDocumentNodes", 1000000),
    ] {
        limits.insert(name.into(), json!({"type":"integer","minimum":1,"maximum":max,"description":"호출자가 낮출 수 있는 작업량 상한."}));
    }
    // A limits object is optional; if provided all ceilings are explicit.
    let names: Vec<&str> = limits.keys().map(String::as_str).collect();
    defs.insert(
        "TemplateLimits".into(),
        closed_object(
            json!(limits),
            &names,
            "요청별 작업량 상한. 생략 시 기본 상한, 증가는 거부한다.",
        ),
    );
    let index_step = closed_object(
        json!({
            "kind":{"type":"string","enum":["paragraph","control","cell","groupChild"],"description":"소유 경로 종류."},
            "index":uint("해당 소유 배열의 0 기준 인덱스.")
        }),
        &["kind", "index"],
        "인덱스가 있는 소유 경로 요소.",
    );
    let unit_step = closed_object(
        json!({
            "kind":{"type":"string","enum":["shape","textBox","caption"],"description":"단일 소유 노드 종류."}
        }),
        &["kind"],
        "인덱스가 없는 소유 경로 요소.",
    );
    defs.insert("TemplatePath".into(), json!({"type":"array","maxItems":64,"items":{"oneOf":[index_step,unit_step]},"description":"요청 원형 기준 소유 경로. 페이지 번호나 영구 ID가 아니다."}));
    let text = closed_object(
        json!({"kind":constant("textRange","텍스트 범위."),
        "path":r_doc("TemplatePath","대상 문단의 경로."), "start":uint("시작 Unicode scalar 인덱스."),"end":uint("끝 제외 Unicode scalar 인덱스.")}),
        &["kind", "path", "start", "end"],
        "서식과 범위 밖 구조를 보존하는 텍스트 채우기.",
    );
    let field = closed_object(
        json!({"kind":constant("field","닫힌 ClickHere 필드."),"path":r_doc("TemplatePath","필드 소유 문단 경로."),"fieldRangeIndex":uint("문단 내 field_ranges 인덱스. 전역 occurrence가 아니다.")}),
        &["kind", "path", "fieldRangeIndex"],
        "같은 문단에서 닫힌 필드 전체 값 채우기.",
    );
    defs.insert("TemplateBindings".into(), json!({"type":"array","maxItems":10000,"description":"key별 명시 대상. 중복 key와 겹친 대상은 거부한다.","items":closed_object(json!({
        "key":prim("string","record에서 정확히 대응할 key."),"target":{"oneOf":[text,field],"description":"문단 텍스트 범위 또는 닫힌 필드."}
    }), &["key","target"], "한 데이터 key와 대상의 연결.")}));
    defs.insert("TemplateRecord".into(), json!({"type":"object","additionalProperties":prim("string","대상에 넣을 값."),"description":"binding key와 정확히 일치하는 문자열 값. 빈 문자열은 허용하고 누락/여분은 거부한다."}));
    defs.insert("TemplateRecords".into(), json!({"type":"array","maxItems":1000,"items":r("TemplateRecord"),"description":"복사본당 한 record. 빈 배열은 주소·옵션 검사 후 무변경이다."}));
    defs.insert("TemplateScope".into(), closed_object(json!({"sectionIndex":uint("구역 인덱스."),"start":uint("본문 시작 문단."),"end":uint("본문 끝 제외 문단."),"limits":r_doc("TemplateLimits","선택 작업량 상한.")}), &["sectionIndex","start","end"], "고정 양식에서 찾을 명시 범위."));
    defs.insert("TemplateBlock".into(), closed_object(json!({"sectionIndex":uint("구역 인덱스."),"sourceStart":uint("원형 시작 문단."),"sourceEnd":uint("원형 끝 제외 문단."),"insertBefore":uint("원본 기준 삽입 경계."),"count":uint("records 길이와 같은 복사 수."),"limits":r_doc("TemplateLimits","선택 작업량 상한.")}), &["sectionIndex","sourceStart","sourceEnd","insertBefore","count"], "완결 문단 블록 복사 주소."));
    let form = closed_object(
        json!({"scope":r_doc("TemplateScope","대상 검색 범위."),"bindings":r_doc("TemplateBindings","채우기 대상."),"record":r_doc("TemplateRecord","양식에 쓸 값.")}),
        &["scope", "bindings", "record"],
        "고정 양식 채우기 요청.",
    );
    let block = closed_object(
        json!({"block":r_doc("TemplateBlock","원형과 삽입 주소."),"bindings":r_doc("TemplateBindings","복사본 상대 대상."),"records":r_doc("TemplateRecords","복사본별 값.")}),
        &["block", "bindings", "records"],
        "문단 블록 복제·채우기 요청.",
    );
    let row = |doc: &str| json!({"type":"integer","minimum":0,"maximum":65535,"description":doc});
    let rows = closed_object(
        json!({"sectionIndex":uint("구역 인덱스."),"paragraphIndex":uint("본문 표 소유 문단."),"controlIndex":uint("본문 표 컨트롤 인덱스."),"startRow":row("원형 시작 행."),"endRow":row("원형 끝 제외 행."),"insertBefore":row("원본 기준 삽입 행 경계."),"bindings":r_doc("TemplateBindings","Paragraph(0)/Control(0)/Cell(n)/Paragraph(p) 상대 대상. n은 선택 셀의 행·열 정렬 인덱스."),"records":r_doc("TemplateRecords","새 행 묶음별 값."),"limits":r_doc("TemplateLimits","선택 작업량 상한.")}),
        &[
            "sectionIndex",
            "paragraphIndex",
            "controlIndex",
            "startRow",
            "endRow",
            "insertBefore",
            "bindings",
            "records",
        ],
        "본문 최상위 표의 완결 행 복제. 제목 및 교차 병합/zone 거부.",
    );
    for ((name, action), request) in [
        "FillTemplateStep",
        "RepeatAndFillParagraphBlockStep",
        "RepeatAndFillTableRowsStep",
    ]
    .into_iter()
    .zip(ACTIONS)
    .zip([form, block, rows])
    {
        defs.insert(
            name.into(),
            step_variant(
                action,
                "원자 템플릿 action. 단독 step만 지원.",
                json!({"request":request}),
                &["request"],
                "동일 native 준비/반영 엔진. dry-run은 준비까지만 실행한다.",
            ),
        );
    }
    let import_limits = closed_object(
        json!({
            "block":r_doc("TemplateLimits","블록 구조 상한. 생략 시 기본값."),
            "maxResourceMetadataBytes":{"type":"integer","minimum":1,"maximum":33554432,"description":"자원 metadata 준비 바이트 상한."},
            "maxBinaryBytes":{"type":"integer","minimum":1,"maximum":67108864,"description":"디코딩·비교하는 바이너리 합계 상한."}
        }),
        &["maxResourceMetadataBytes", "maxBinaryBytes"],
        "가져오기 자원 상한. 생략 시 native 기본값.",
    );
    let import_request = closed_object(
        json!({
            "sourceSection":uint("원본 구역."),"sourceStart":uint("원본 시작 문단."),"sourceEnd":uint("원본 끝 제외 문단."),
            "targetSection":uint("대상 구역."),"insertBefore":uint("대상 문단 앞 경계."),"count":uint("가져올 복사 수. 0도 주소를 검증한다."),
            "limits":import_limits
        }),
        &[
            "sourceSection",
            "sourceStart",
            "sourceEnd",
            "targetSection",
            "insertBefore",
            "count",
        ],
        "다른 문서의 완결 블록 가져오기 요청.",
    );
    let source = closed_object(
        json!({
            "path":{"type":"string","minLength":1,"description":"로컬 일반 파일, 최대 64 MiB. 상대 경로는 프로세스 작업 폴더 기준. 대상 입력/출력과 별도 파일이어야 한다."},
            "sha256":{"type":"string","pattern":"^[0-9a-fA-F]{64}$","description":"실제 한 번 읽은 원본 바이트의 SHA-256. 불일치는 exit 3, 실행·저장 없음."}
        }),
        &["path", "sha256"],
        "원본 snapshot 식별. 지문 검증과 파싱이 같은 바이트를 소비한다.",
    );
    defs.insert(
        "ImportParagraphBlockStep".into(),
        step_variant(
            "import_paragraph_block",
            "다른 문서의 블록 가져오기. 단독 step만 지원.",
            json!({"request":import_request,"source":source}),
            &["request", "source"],
            "WASM/native와 같은 가져오기 엔진. 원본 용지 설정을 대상에 덮어쓰지 않는다.",
        ),
    );
    defs.get_mut("ImportParagraphBlockStep")
        .expect("import step")["additionalProperties"] = json!(false);
}

pub(super) fn restrict_single_step(plan: &mut Value) {
    plan["allOf"] = json!([{"if":{"properties":{"steps":{"contains":{"properties":{"action":{"enum":ACTIONS}},"required":["action"]}}}},"then":{"properties":{"steps":{"maxItems":1}}}}]);
}
