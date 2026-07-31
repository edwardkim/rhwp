//! Task #3315: 그림 바이트를 레이어 트리 JSON 에서 빼고 신원 키로 따로 받는다.
//!
//! 이 옵션이 성립하려면 **생략본 + 키 조회가 인라인 base64 와 같은 것을 말해야** 한다.
//! 바이트가 한 바이트라도 다르면 소비자는 같은 그림을 다르게 그린다. 그래서 여기서 고정하는
//! 것은 크기 이득이 아니라 등가성이다 — 크기는 그 등가성이 성립한 뒤의 부산물이다.
//!
//! 변환 사슬(BMP/PCX/TIFF/회색 JPEG → PNG, JPEG 워터마크 bake)이 JSON 경로와 키 조회 경로에
//! 각각 사본으로 존재하면 이 등가성이 조용히 깨진다. 두 경로가 같은 함수를 쓰는지 확인하는
//! 자리도 이 테스트다.

use base64::Engine;
use serde_json::Value;

use rhwp::paint::{parse_source_image_key, LayerJsonOptions, SourceImageVariant};

/// 그림 op 을 등장 순서대로 모은다. 인라인본과 생략본을 짝지어 비교하려면 순서가 같아야 한다.
fn collect_image_ops(node: &Value, out: &mut Vec<Value>) {
    if let Some(ops) = node.get("ops").and_then(Value::as_array) {
        for op in ops {
            if op.get("type").and_then(Value::as_str) == Some("image") {
                out.push(op.clone());
            }
        }
    }
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            collect_image_ops(child, out);
        }
    }
    // clipRect 노드는 단일 `child` 를 갖는다.
    if let Some(child) = node.get("child") {
        collect_image_ops(child, out);
    }
}

fn image_ops(json: &str) -> Vec<Value> {
    let value: Value = serde_json::from_str(json).expect("레이어 JSON 이 유효해야 한다");
    let mut ops = Vec::new();
    collect_image_ops(&value["root"], &mut ops);
    ops
}

fn open_sample() -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(std::path::Path::new(repo_root).join("samples/hwpx/issue_241.hwpx"))
        .expect("read samples/hwpx/issue_241.hwpx");
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse issue_241.hwpx")
}

fn inline_json(doc: &rhwp::wasm_api::HwpDocument) -> String {
    doc.get_page_layer_tree_native(0)
        .expect("inline layer tree")
}

fn omitted_json(doc: &rhwp::wasm_api::HwpDocument) -> String {
    doc.get_page_layer_tree_with_options_native(
        0,
        rhwp::paint::RenderProfile::Screen,
        LayerJsonOptions {
            omit_image_bytes: true,
        },
    )
    .expect("omitted layer tree")
}

#[test]
fn issue_3315_omitted_bytes_are_recoverable_by_key() {
    let doc = open_sample();
    let inline_ops = image_ops(&inline_json(&doc));
    let omitted_ops = image_ops(&omitted_json(&doc));

    assert!(!inline_ops.is_empty(), "도장 그림이 있어야 한다");
    assert_eq!(
        inline_ops.len(),
        omitted_ops.len(),
        "생략은 op 을 지우는 게 아니라 payload 만 뺀다"
    );

    for (inline_op, omitted_op) in inline_ops.iter().zip(&omitted_ops) {
        let encoded = inline_op["base64"]
            .as_str()
            .expect("인라인본에는 base64 가 있어야 한다");
        let expected = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .expect("인라인 base64 왕복");

        let key = omitted_op["sourceImageKey"]
            .as_str()
            .expect("생략된 op 에는 키가 있어야 한다");
        let (mime, actual) = doc
            .get_source_image_bytes_native(key)
            .expect("키로 바이트를 받을 수 있어야 한다");

        assert_eq!(
            actual, expected,
            "키로 받은 바이트가 인라인 base64 와 달라졌다 (key={key})"
        );
        assert_eq!(
            Some(mime),
            inline_op["mime"].as_str(),
            "키 조회의 mime 이 JSON 의 mime 과 달라졌다 (key={key})"
        );
    }
}

#[test]
fn issue_3315_omitted_ops_keep_mime_and_declare_omission() {
    let doc = open_sample();
    let omitted = omitted_json(&doc);
    let ops = image_ops(&omitted);

    assert!(
        omitted.contains("\"imageBytes\":\"byKey\""),
        "문서 단위로 생략본임을 알려야 한다"
    );
    for op in &ops {
        assert!(
            op.get("base64").is_none(),
            "생략본에 base64 가 남아 있다: {op}"
        );
        assert_eq!(
            op["imageBytesOmitted"].as_bool(),
            Some(true),
            "op 마다 생략을 명시해야 한다 — 소비자가 부재를 추측하게 두지 않는다"
        );
        assert!(
            op.get("mime").and_then(Value::as_str).is_some(),
            "mime 은 남는다 — 소비자가 Blob 타입을 정하는 데 쓴다"
        );
        // bbox·effect 같은 배치 정보는 생략과 무관하게 그대로다.
        assert!(op.get("bbox").is_some(), "bbox 가 사라졌다: {op}");
    }
}

/// 기본 경로의 계약은 **additive** 다 — "바이트 동일"이 아니다.
///
/// 이 테스트의 이름은 원래 `..._is_byte_identical` 이었는데, 그 이름이 주장하는 성질은 **거짓**
/// 이다. 이 기능이 최상위 `imageBytes` 를 더하고 schema minor 를 20 → 21 로 올렸으므로 기본
/// 호출의 JSON 은 종전과 바이트 단위로 같지 않다. 이름만 보고 "schema 가 안 바뀌었다"고
/// 판단하면 소비자 호환 결정을 잘못 내린다.
///
/// 실제로 지켜야 하는 것은 둘이다 — ①그림 op 의 payload(`mime`·`base64`)가 종전과 같다
/// ②추가된 것은 문서화된 두 필드뿐이고 생략 표식은 나타나지 않는다.
#[test]
fn issue_3315_default_serialization_keeps_payloads_and_only_adds_documented_fields() {
    let doc = open_sample();
    let inline = inline_json(&doc);
    let value: Value = serde_json::from_str(&inline).expect("레이어 JSON 이 유효해야 한다");

    // 추가된 계약을 명시적으로 고정한다 — schema minor 를 올렸다는 사실이 계약의 일부다.
    assert_eq!(
        value["schemaMinorVersion"].as_u64(),
        Some(u64::from(rhwp::paint::PAGE_LAYER_TREE_SCHEMA_MINOR_VERSION)),
        "schema minor 는 상수와 같아야 한다"
    );
    // 컴파일 시점 하한 — 이 기능은 minor 21 에서 들어왔다. 내려가면 소비자 협상이 깨진다.
    // (런타임 `assert!` 는 상수라 clippy 가 거부한다 — const 단언이 더 이르게 잡는다.)
    const _: () = assert!(rhwp::paint::PAGE_LAYER_TREE_SCHEMA_MINOR_VERSION >= 21);
    assert_eq!(
        value["imageBytes"].as_str(),
        Some("inline"),
        "옵션을 지정하지 않은 호출의 모드는 inline 이다"
    );
    assert!(
        !inline.contains("\"imageBytesOmitted\""),
        "기본 경로에는 생략 표식이 없어야 한다"
    );

    for op in image_ops(&inline) {
        assert!(
            op.get("base64").is_some(),
            "옵션을 켜지 않은 호출은 종전대로 바이트를 싣는다"
        );
        assert!(
            op.get("mime").and_then(Value::as_str).is_some(),
            "payload 의 mime 도 종전대로다"
        );
    }
}

/// 크기 이득은 이 트랙의 대상 시나리오 — **본문에 인라인된 대형 JPEG** — 에서만 의미가 있다.
///
/// 도장처럼 작은 그림만 있는 문서에서는 JSON 이 텍스트·글리프로 지배되므로 생략해도 몇 %만
/// 줄어든다(실측 248KB → 233KB). 그 숫자로 이 옵션을 정당화하면 안 되므로, 여기서는 #2520 이
/// 문제로 지목한 크기의 그림을 실제로 넣고 잰다.
#[test]
fn issue_3315_omitting_bytes_shrinks_the_payload_for_large_images() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let jpeg = std::fs::read(std::path::Path::new(repo_root).join("samples/images/tiger01.jpg"))
        .expect("read tiger01.jpg");
    let mut doc = open_sample();
    doc.insert_picture_native(
        0,
        0,
        0,
        &[],
        &jpeg,
        400,
        300,
        2400,
        1800,
        "jpg",
        "tiger",
        None,
        None,
    )
    .expect("insert picture");

    let inline = inline_json(&doc);
    let omitted = omitted_json(&doc);

    // 생략본에는 그림 바이트가 없으므로 원본 크기에 비례하지 않는다.
    assert!(
        omitted.len() * 4 < inline.len(),
        "생략본이 충분히 작지 않다 (원본 그림 {} bytes, inline={} bytes, omitted={} bytes)",
        jpeg.len(),
        inline.len(),
        omitted.len()
    );
    println!(
        "[#3315] 원본 그림 {} bytes / inline JSON {} bytes / 생략 JSON {} bytes ({:.1}배 축소)",
        jpeg.len(),
        inline.len(),
        omitted.len(),
        inline.len() as f64 / omitted.len() as f64
    );

    // 크기가 줄었어도 바이트를 되찾을 수 있어야 의미가 있다.
    for op in image_ops(&omitted) {
        let key = op["sourceImageKey"].as_str().expect("키");
        assert!(
            doc.get_source_image_bytes_native(key).is_some(),
            "생략했는데 키로 되찾을 수 없다 (key={key})"
        );
    }
}

#[test]
fn issue_3315_inline_and_omitted_are_separate_cache_variants() {
    let doc = open_sample();

    // 같은 페이지를 번갈아 물어본다. 두 모양이 캐시 슬롯을 공유하면 뒤의 호출이 앞의 모양을
    // 돌려받는다 — #2222 JSON 캐시가 지문에 생략 여부를 접지 않으면 나는 결함이다.
    let first_inline = inline_json(&doc);
    let omitted = omitted_json(&doc);
    let second_inline = inline_json(&doc);
    let second_omitted = omitted_json(&doc);

    assert_eq!(first_inline, second_inline, "인라인본이 캐시에서 오염됐다");
    assert_eq!(omitted, second_omitted, "생략본이 캐시에서 오염됐다");
    assert_ne!(first_inline, omitted, "두 모양이 같을 수 없다");
}

#[test]
fn issue_3315_unresolvable_keys_are_refused() {
    let doc = open_sample();
    let key = image_ops(&omitted_json(&doc))[0]["sourceImageKey"]
        .as_str()
        .expect("키")
        .to_string();
    let (epoch, bin_data_id, _variant) = parse_source_image_key(&key).expect("키 해석");

    // 세대가 다른 키 — 스냅샷 복원 뒤에 옛 키로 물어본 경우. 낡은 바이트를 주면 안 된다.
    let stale = format!("bin:{}:{bin_data_id}:src", epoch.wrapping_add(1));
    assert!(
        doc.get_source_image_bytes_native(&stale).is_none(),
        "세대가 다른 키를 받아들였다"
    );

    // 없는 그림.
    assert!(
        doc.get_source_image_bytes_native(&format!("bin:{epoch}:60000:src"))
            .is_none(),
        "없는 bin_data_id 를 받아들였다"
    );

    for malformed in [
        "",
        "bin",
        "bin:0:1",
        "bin:0:1:src:extra",
        "img:0:1:src",
        "bin:x:1:src",
        // 모르는 variant 를 src 로 흘리면 워터마크 그림에 원본 JPEG 을 주고도 성공해 보인다.
        "bin:0:1:png",
    ] {
        assert!(
            parse_source_image_key(malformed).is_none(),
            "형식이 틀린 키를 받아들였다: {malformed:?}"
        );
        assert!(
            doc.get_source_image_bytes_native(malformed).is_none(),
            "형식이 틀린 키로 바이트를 돌려줬다: {malformed:?}"
        );
    }
}

#[test]
fn issue_3315_key_round_trips_through_parser() {
    for (epoch, id, variant) in [
        (0u32, 1u16, SourceImageVariant::Source),
        (7, 60001, SourceImageVariant::BakedWatermarkPng),
    ] {
        let key = format!("bin:{epoch}:{id}:{}", variant.as_str());
        assert_eq!(
            parse_source_image_key(&key),
            Some((epoch, id, variant)),
            "발급 포맷과 해석이 어긋난다: {key}"
        );
    }
    assert!(!SourceImageVariant::Source.bakes_watermark());
    assert!(SourceImageVariant::BakedWatermarkPng.bakes_watermark());
}
