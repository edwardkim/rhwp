// 실제 원본 주소를 검증한다. 공개 런의 논리 위치를 원문의 스칼라 오프셋으로 단정하지 않는다.
// CharOverlap의 제한된 주소와 내부 머리말/꼬리말은 원본·공개 런의 실제 경계로만 인정한다.

fn assert_stage4_profile(trace: &serde_json::Value, profile: &serde_json::Value) {
    let records = trace["records"].as_array().expect("trace records");
    let address = profile["sourceAddress"]
        .as_object()
        .expect("원본 문자 주소");
    let matches: Vec<_> = records
        .iter()
        .filter(|record| {
            address
                .iter()
                .all(|(key, value)| record["source"].get(key) == Some(value))
        })
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "같은 원본 문자의 프로필은 유일하다: {}",
        profile["id"]
    );
    let record = matches[0];
    let expected = &profile["expected"];
    assert_eq!(record["source"]["status"], expected["sourceStatus"]);
    assert_eq!(record["source"]["character"], expected["character"]);
    assert_eq!(record["document"]["face"], expected["documentFace"]);
    assert_eq!(record["document"]["substFont"], expected["substFont"]);
    assert_eq!(
        record["layoutName"]["normalizedFace"],
        expected["normalizedFace"]
    );
    let expected_step = &expected["layoutStepKind"];
    if expected_step.is_null() {
        assert!(record["layoutName"]["steps"]
            .as_array()
            .expect("layout steps")
            .is_empty());
    } else {
        assert!(record["layoutName"]["steps"]
            .as_array()
            .expect("layout steps")
            .iter()
            .any(|step| step.get("kind") == Some(expected_step)));
    }
    assert_eq!(
        record["layoutMetric"]["matchKind"],
        expected["metricMatchKind"]
    );
    assert_eq!(
        record["layoutMetric"]["characterMatch"], expected["metricCharacterMatch"],
        "{}: 전용 폭 표의 문자 hit와 공통 메트릭 DB의 matchKind는 별개다",
        profile["id"]
    );
    assert_eq!(
        record["layoutMetric"]["widthSource"],
        expected["widthSource"]
    );
}

fn assert_profile_run_ownership(document: &serde_json::Value, trace: &serde_json::Value) {
    use rhwp::model::{control::Control, paragraph::Paragraph};
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(document["path"].as_str().expect("source path"));
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("동일 원본")).expect("원본 문서 열기");
    let public: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(document["page"].as_u64().expect("page") as u32)
            .expect("공개 문자 배치"),
    )
    .expect("문자 배치 JSON");
    let runs = public["runs"].as_array().expect("공개 소유 런");
    assert_eq!(
        trace["counts"]["runsSeen"].as_u64(),
        Some(runs.len() as u64)
    );
    let count: usize = runs
        .iter()
        .map(|run| run["text"].as_str().expect("내용").chars().count())
        .sum();
    assert_eq!(
        trace["counts"]["charactersSeen"].as_u64(),
        Some(count as u64)
    );
    let records = trace["records"].as_array().expect("문자 기록");
    let mut record_ids = std::collections::HashSet::new();
    let mut raw_addresses = std::collections::HashSet::new();
    let mut previous_in_run = None;
    fn source_paragraph<'a>(core: &'a DocumentCore, source: &serde_json::Value) -> &'a Paragraph {
        let section = source["sectionIndex"].as_u64().expect("원본 구역") as usize;
        let root_para = source["paragraphIndex"].as_u64().expect("원본 문단") as usize;
        let mut para = core
            .document()
            .sections
            .get(section)
            .expect("원본 구역 범위")
            .paragraphs
            .get(root_para)
            .expect("원본 문단 범위");
        let path = source["nestedPath"].as_array().expect("원본 경로");
        assert_eq!(path.len() % 3, 0);
        for entry in path.chunks_exact(3) {
            let ci = entry[0].as_u64().expect("원본 개체") as usize;
            let cei = entry[1].as_u64().expect("원본 셀") as usize;
            let cpi = entry[2].as_u64().expect("원본 셀 문단") as usize;
            para = match para.controls.get(ci).expect("원본 개체 범위") {
                Control::Table(table) => table
                    .cells
                    .get(cei)
                    .expect("원본 셀 범위")
                    .paragraphs
                    .get(cpi)
                    .expect("원본 셀 문단 범위"),
                Control::Shape(shape) if cei == 0 => shape
                    .drawing()
                    .expect("원본 그리기 개체")
                    .text_box
                    .as_ref()
                    .expect("원본 글상자")
                    .paragraphs
                    .get(cpi)
                    .expect("원본 글상자 문단 범위"),
                _ => panic!("원본 문단 소유 컨테이너가 아니다"),
            };
        }
        para
    }
    for record in records {
        let id = record["recordId"].as_str().expect("기록 ID");
        assert!(record_ids.insert(id), "기록을 중복하지 않는다");
        let source = &record["source"];
        let owner = source["runIndex"].as_u64().expect("소유 런") as usize;
        let run = runs.get(owner).expect("실제 소유 런");
        let local = id
            .rsplit(":char:")
            .next()
            .expect("런 내부 문자 번호")
            .parse::<usize>()
            .expect("문자 번호");
        let character = run["text"]
            .as_str()
            .expect("소유 런 내용")
            .chars()
            .nth(local)
            .expect("문자는 소유 런 안에 포함");
        assert_eq!(source["character"], character.to_string());
        assert_eq!(source["charShapeId"], run["charShapeId"]);
        assert_eq!(source["sectionIndex"], run["secIdx"]);
        let nested: Vec<_> = run["cellPath"]
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|entry| {
                [
                    entry["controlIndex"].clone(),
                    entry["cellIndex"].clone(),
                    entry["cellParaIndex"].clone(),
                ]
            })
            .collect();
        assert_eq!(source["nestedPath"], serde_json::Value::Array(nested));
        if source["paragraphIndex"].is_null() {
            assert_eq!(source["status"], "unavailable");
            assert!(source["charOffset"].is_null());
            let virtual_owner = run["paraIdx"]
                .as_u64()
                .is_some_and(|pi| pi >= usize::MAX.saturating_sub(4096) as u64);
            assert!(
                virtual_owner || run["charStart"].is_null(),
                "실물 본문 소유자를 임의로 누락하지 않는다"
            );
            previous_in_run = None;
            continue;
        }
        assert_eq!(
            &source["paragraphIndex"],
            if run["cellPath"].is_null() {
                &run["paraIdx"]
            } else {
                &run["parentParaIdx"]
            }
        );
        let para = source_paragraph(&core, source);
        if source["status"] == "unavailable" {
            assert!(
                source["charOffset"].is_null(),
                "추정 주소를 확정 원본 주소로 남기지 않는다"
            );
            let logical = run["charStart"].as_u64().expect("공개 논리 위치") as usize + local;
            assert!(
                para.controls
                    .iter()
                    .zip(para.logical_control_positions())
                    .any(
                        |(control, at)| matches!(control, Control::CharOverlap(_)) && at <= logical
                    ),
                "주소를 모르는 범위는 원본의 글자 겹침 이후 경계여야 한다"
            );
            previous_in_run = None;
            continue;
        }
        assert_eq!(source["status"], "complete");
        let raw = source["charOffset"].as_u64().expect("원문 스칼라 주소") as usize;
        assert_eq!(
            para.text.chars().nth(raw),
            Some(character),
            "파싱한 원문의 실제 문자"
        );
        assert_eq!(
            source["charShapeId"].as_u64(),
            para.char_shape_id_at(raw).map(u64::from),
            "원문 주소의 실제 글자 모양"
        );
        let address = (
            source["sectionIndex"].to_string(),
            source["paragraphIndex"].to_string(),
            source["nestedPath"].to_string(),
            raw,
        );
        assert!(
            raw_addresses.insert(address),
            "같은 원문 문자를 다른 출력 문자에 중복 연결하지 않는다"
        );
        if let Some((previous_owner, previous_local, previous_raw)) = previous_in_run {
            if previous_owner == owner && previous_local + 1 == local {
                assert_eq!(
                    raw,
                    previous_raw + 1,
                    "같은 텍스트 런의 원문 순서와 연속성을 보존한다"
                );
            }
        }
        previous_in_run = Some((owner, local, raw));
        if para
            .controls
            .iter()
            .all(|c| !c.is_logical_inline() && !matches!(c, Control::CharOverlap(_)))
        {
            assert_eq!(
                raw as u64,
                run["charStart"].as_u64().expect("원문과 같은 축") + local as u64
            );
        }
    }
}
