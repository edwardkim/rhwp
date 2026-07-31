//! Task #3315: 본문(flow) 그림의 배치 정보만 받는 좁은 질의.
//!
//! studio 는 flow 그림을 DOM `<img>` 로 내보내려고 편집마다 전체 레이어 트리 JSON 을 받아
//! 왔다. 이 질의가 그것을 대체하려면 **전체 트리에서 뽑아낸 것과 같은 것을 말해야** 한다 —
//! 개수·순서·bbox·잘림·효과가 어긋나면 화면이 달라진다. 여기서 고정하는 것은 크기 이득이
//! 아니라 그 일치다.
//!
//! 캐시가 아니라 질의를 좁힌 이유도 함께 고정한다: 본문이 흐르면 그림이 그대로여도 bbox 가
//! 움직이므로 `(page, imageKeys)` 캐시로는 풀 수 없다.

use serde_json::Value;

/// 그림 op 의 replay plane — studio `layerPaintOpReplayPlane` 을 그대로 옮긴 것.
///
/// `layer.textWrap` 이 있으면 그것이 우선이고, 없을 때만 op 의 `wrap` 을 본다. `behindText`·
/// `inFrontOfText` 만 flow 가 아니다 — `square` 같은 본문 배치 wrap 은 flow 다. 그리고
/// 바탕쪽 유래 개체는 항상 본문 뒤로 눌린다(`capMasterPagePlane`).
fn is_flow_image(op: &Value, layer: Option<&Value>) -> bool {
    let master_page = layer
        .and_then(|l| l.get("masterPage"))
        .and_then(Value::as_bool)
        == Some(true);
    if master_page {
        return false;
    }
    let wrap = match layer
        .and_then(|l| l.get("textWrap"))
        .and_then(Value::as_str)
    {
        Some(layer_wrap) => Some(layer_wrap),
        None => op.get("wrap").and_then(Value::as_str),
    };
    !matches!(wrap, Some("behindText") | Some("inFrontOfText"))
}

/// 전체 트리에서 뽑은 flow 그림 — op 과 **조상 clip 교차 결과**를 함께 든다.
///
/// clip 은 op 안에 있지 않고 조상 `ClipRect` 계보에서 나오므로, 좁은 질의의 `clip` 필드와
/// 대조하려면 트리 쪽에서도 같은 값을 계산해 들고 있어야 한다.
struct TreeFlowImage {
    op: Value,
    clip: Option<[f64; 4]>,
}

/// 전체 레이어 트리에서 flow plane 그림 op 을 studio 의 `collectFlowImagePaintOps` 와 같은
/// 계약으로 뽑는다 — pre-order, `layer` 상속, `clipRect` 조상 교차.
fn collect_flow_images_from_tree<'a>(
    node: &'a Value,
    inherited_layer: Option<&'a Value>,
    clip: Option<[f64; 4]>,
    out: &mut Vec<TreeFlowImage>,
) {
    let active_layer = node.get("layer").or(inherited_layer);
    let next_clip = if node.get("kind").and_then(Value::as_str) == Some("clipRect") {
        match bbox_of(node.get("clip")) {
            Some(node_clip) => match intersect(clip, node_clip) {
                Some(next) => Some(next),
                // 교차가 비면 이 아래는 보이지 않는다.
                None => return,
            },
            None => clip,
        }
    } else {
        clip
    };

    if let Some(ops) = node.get("ops").and_then(Value::as_array) {
        for op in ops {
            if op.get("type").and_then(Value::as_str) != Some("image") {
                continue;
            }
            if !is_flow_image(op, active_layer) {
                continue;
            }
            let Some(bbox) = bbox_of(op.get("bbox")) else {
                continue;
            };
            if intersect(next_clip, bbox).is_none() {
                continue;
            }
            out.push(TreeFlowImage {
                op: op.clone(),
                clip: next_clip,
            });
        }
    }
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            collect_flow_images_from_tree(child, active_layer, next_clip, out);
        }
    }
    if let Some(child) = node.get("child") {
        collect_flow_images_from_tree(child, active_layer, next_clip, out);
    }
}

/// `Option<&Value>` 를 거치지 않는 형태 — `clip` 필드가 있으면 반드시 유효한 bbox 여야 한다.
fn bbox_of_value(value: &Value) -> [f64; 4] {
    bbox_of(Some(value)).expect("clip 필드는 유효한 bbox 여야 한다")
}

/// 직렬화 정밀도(`{:.3}`)로 접어 비교한다.
///
/// 두 생산자는 교차를 **다른 순서**로 한다 — Rust 는 교차한 뒤 `{:.3}` 으로 쓰고, 트리 경로는
/// 이미 `{:.3}` 인 값들을 받아 소비자 쪽에서 교차한다. 그래서 `211.987` 과
/// `211.98699999999997` 처럼 마지막 비트만 다른 값이 나온다. 이 차이는 소비자의 wrapper 를
/// bbox 와 사실상 같은 크기로 만들 뿐이라 화면에 나타나지 않는다. 이 테스트가 잡아야 하는 것은
/// **clip 의 존재 여부와 위치**이므로 직렬화 정밀도에서 비교한다.
fn rounded(bbox: Option<[f64; 4]>) -> Option<[i64; 4]> {
    bbox.map(|values| values.map(|value| (value * 1000.0).round() as i64))
}

fn bbox_of(value: Option<&Value>) -> Option<[f64; 4]> {
    let value = value?;
    Some([
        value.get("x")?.as_f64()?,
        value.get("y")?.as_f64()?,
        value.get("width")?.as_f64()?,
        value.get("height")?.as_f64()?,
    ])
}

fn intersect(first: Option<[f64; 4]>, second: [f64; 4]) -> Option<[f64; 4]> {
    let Some(first) = first else {
        return Some(second);
    };
    let left = first[0].max(second[0]);
    let top = first[1].max(second[1]);
    let right = (first[0] + first[2]).min(second[0] + second[2]);
    let bottom = (first[1] + first[3]).min(second[1] + second[3]);
    if right <= left || bottom <= top {
        return None;
    }
    Some([left, top, right - left, bottom - top])
}

/// 본문 흐름에 실제로 얽힌 flow 그림이 있는 표본들.
///
/// `insert_picture_native` 로 넣은 그림은 (0,0) 에 고정된 부동 개체라 본문이 흘러도 움직이지
/// 않는다 — 이 트랙이 다루는 "본문 인라인 그림"이 아니다. 그래서 등가성은 실문서로 잰다.
const FLOW_IMAGE_SAMPLES: &[&str] = &[
    "samples/143E433F503322BD33.hwp",
    "samples/3-10월_교육_통합_2022.hwp",
    "samples/20250130-hongbo.hwp",
    "samples/3-09월_교육_통합_2023.hwpx",
];

fn open(relative: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(std::path::Path::new(repo_root).join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("parse {relative}: {error:?}"))
}

/// 대형 JPEG 을 세션 중에 넣은 문서 — 크기 이득을 재는 데 쓴다.
fn open_with_large_inserted_image() -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let jpeg = std::fs::read(std::path::Path::new(repo_root).join("samples/images/tiger01.jpg"))
        .expect("read tiger01.jpg");
    let mut doc = open("samples/hwpx/issue_241.hwpx");
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
    doc
}

fn narrow(doc: &rhwp::wasm_api::HwpDocument, page: u32) -> Value {
    let json = doc
        .get_page_flow_image_ops_native(page)
        .expect("flow image ops");
    serde_json::from_str(&json).expect("좁은 질의 JSON 이 유효해야 한다")
}

fn tree_flow_images(doc: &rhwp::wasm_api::HwpDocument, page: u32) -> Vec<TreeFlowImage> {
    let json = doc
        .get_page_layer_tree_with_profile_native(page, rhwp::paint::RenderProfile::Screen)
        .expect("layer tree");
    let tree: Value = serde_json::from_str(&json).expect("레이어 JSON 이 유효해야 한다");
    let mut out = Vec::new();
    collect_flow_images_from_tree(&tree["root"], None, None, &mut out);
    out
}

/// 이 질의가 전체 트리 경로를 대체할 수 있다는 것 — 이 파일의 핵심 단언.
#[test]
fn issue_3315_narrow_query_matches_full_tree() {
    let mut total_images = 0;
    for sample in FLOW_IMAGE_SAMPLES {
        let doc = open(sample);
        let pages = doc.page_count().min(3);
        for page in 0..pages {
            let narrow = narrow(&doc, page);
            let from_tree = tree_flow_images(&doc, page);
            let images = narrow["images"].as_array().expect("images 배열");

            assert_eq!(
                images.len(),
                from_tree.len(),
                "{sample} p{page}: 좁은 질의와 전체 트리의 flow 그림 개수가 다르다"
            );
            total_images += images.len();

            for (index, (narrow_op, tree)) in images.iter().zip(&from_tree).enumerate() {
                let at = format!("{sample} p{page} #{index}");
                let tree_op = &tree.op;
                // clip 은 op 밖(조상 ClipRect)에서 오므로 트리 쪽 계산값과 대조한다.
                //
                // **존재 여부까지** 맞아야 한다. `page-renderer.ts` 의 `needsClipWrapper` 는
                // `clip !== null` 을 먼저 보고 그다음 `rotation !== 0` 을 보므로, bbox 를 줄이지
                // 않는 clip 을 생략하면 회전 그림이 좁은 질의 경로에서만 모서리를 노출한다.
                assert_eq!(
                    rounded(narrow_op.get("clip").map(bbox_of_value)),
                    rounded(tree.clip),
                    "{at}: 조상 clip 이 다르다 — 회전 그림의 wrapper 판정이 갈린다"
                );
                assert_eq!(
                    narrow_op["bbox"], tree_op["bbox"],
                    "{at}: bbox 가 다르다 — 소비자가 다른 자리에 그린다"
                );
                assert_eq!(narrow_op["mime"], tree_op["mime"], "{at}: mime 이 다르다");
                assert_eq!(
                    narrow_op["effect"], tree_op["effect"],
                    "{at}: effect 가 다르다"
                );
                assert_eq!(
                    narrow_op["brightness"], tree_op["brightness"],
                    "{at}: brightness 가 다르다"
                );
                assert_eq!(
                    narrow_op["contrast"], tree_op["contrast"],
                    "{at}: contrast 가 다르다"
                );
                assert_eq!(
                    narrow_op["transform"], tree_op["transform"],
                    "{at}: transform 이 다르다 — 회전·반전이 어긋난다"
                );
                assert_eq!(narrow_op["crop"], tree_op["crop"], "{at}: crop 이 다르다");
                assert_eq!(
                    narrow_op["originalSizeHu"], tree_op["originalSizeHu"],
                    "{at}: originalSizeHu 가 다르다"
                );
                assert_eq!(
                    narrow_op["bakedWatermark"], tree_op["bakedWatermark"],
                    "{at}: bakedWatermark 가 다르다 — CSS filter 적용 여부가 갈린다"
                );
                assert_eq!(
                    narrow_op["sourceImageKey"], tree_op["sourceImageKey"],
                    "{at}: 신원 키가 다르다 — 바이트를 못 찾거나 남의 바이트를 받는다"
                );
            }
        }
    }
    assert!(
        total_images >= 4,
        "표본이 flow 그림을 실제로 담고 있어야 이 테스트가 의미가 있다 (총 {total_images}장)"
    );
}

#[test]
fn issue_3315_narrow_query_keys_resolve_to_bytes() {
    for sample in FLOW_IMAGE_SAMPLES {
        let doc = open(sample);
        let narrow = narrow(&doc, 0);
        if narrow["cacheable"].as_bool() != Some(true) {
            // 합성 그림이 섞인 페이지는 키를 못 내므로 이 단언의 대상이 아니다.
            continue;
        }
        for image in narrow["images"].as_array().expect("images") {
            let key = image["sourceImageKey"]
                .as_str()
                .expect("cacheable 페이지의 모든 그림은 키를 가져야 한다");
            let (mime, bytes) = doc
                .get_source_image_bytes_native(key)
                .unwrap_or_else(|| panic!("{sample}: 키로 바이트를 받을 수 없다 ({key})"));
            assert!(!bytes.is_empty(), "{sample}: 빈 바이트 ({key})");
            assert_eq!(
                Some(mime),
                image["mime"].as_str(),
                "{sample}: 좁은 질의의 mime 과 키 조회의 mime 이 다르다 ({key})"
            );
        }
    }
}

#[test]
fn issue_3315_narrow_query_is_much_smaller_than_the_tree() {
    let doc = open_with_large_inserted_image();
    let tree = doc.get_page_layer_tree_native(0).expect("layer tree");
    let narrow = doc
        .get_page_flow_image_ops_native(0)
        .expect("flow image ops");

    assert!(
        narrow.len() * 100 < tree.len(),
        "좁은 질의가 충분히 작지 않다 (tree={} bytes, narrow={} bytes)",
        tree.len(),
        narrow.len()
    );
    println!(
        "[#3315] 전체 트리 {} bytes / 좁은 질의 {} bytes ({:.0}배)",
        tree.len(),
        narrow.len(),
        tree.len() as f64 / narrow.len() as f64
    );
}

/// 캐시가 아니라 좁은 질의여야 하는 이유를 고정한다.
///
/// 그림 바이트가 그대로면 `sourceImageKey` 도 그대로다. 그런데 본문 앞에 글자를 넣으면 그림이
/// 밀려 bbox 가 달라진다 — 즉 `(page, imageKeys)` 를 키로 한 캐시는 **틀린 배치를 재사용**한다.
///
/// 부동 개체로 삽입한 그림은 (0,0) 에 고정돼 이 성질을 보이지 않는다. 본문 흐름에 얽힌 실문서가
/// 필요하다 — `143E433F503322BD33.hwp` 는 글자를 넣으면 y 가 468.6 → 503.3 으로 밀린다.
#[test]
fn issue_3315_bbox_moves_while_key_stays_so_caching_would_be_wrong() {
    let mut doc = open("samples/143E433F503322BD33.hwp");
    let before = narrow(&doc, 0);

    for _ in 0..3 {
        doc.insert_text_native(0, 0, 0, "가나다라마바사아자차")
            .expect("insert text");
    }
    let after = narrow(&doc, 0);

    let before_images = before["images"].as_array().expect("images");
    let after_images = after["images"].as_array().expect("images");
    assert!(
        !before_images.is_empty(),
        "표본에 flow 그림이 있어야 한다 — 없으면 이 테스트가 대상을 못 재고 있다"
    );
    assert_eq!(
        before_images.len(),
        after_images.len(),
        "이 테스트는 그림이 같은 쪽에 남는 조건에서만 의미가 있다"
    );

    let keys = |ops: &Vec<Value>| -> Vec<Value> {
        ops.iter().map(|op| op["sourceImageKey"].clone()).collect()
    };
    assert_eq!(
        keys(before_images),
        keys(after_images),
        "본문 편집은 그림 바이트를 바꾸지 않으므로 키가 유지돼야 한다"
    );

    let bboxes =
        |ops: &Vec<Value>| -> Vec<Value> { ops.iter().map(|op| op["bbox"].clone()).collect() };
    assert_ne!(
        bboxes(before_images),
        bboxes(after_images),
        "키가 같은데 bbox 도 같으면 캐시로 풀 수 있다는 뜻이 된다 — \
         이 단언이 깨졌다면 좁은 질의의 근거를 다시 확인해야 한다"
    );
}

#[test]
fn issue_3315_narrow_query_survives_documents_without_flow_images() {
    let doc = open("samples/hwpx/issue_241.hwpx");

    let narrow = narrow(&doc, 0);
    let from_tree = tree_flow_images(&doc, 0);
    assert_eq!(
        narrow["images"].as_array().map(Vec::len),
        Some(from_tree.len()),
        "flow 그림이 없는 페이지에서도 전체 트리와 같은 답이어야 한다"
    );
    assert_eq!(
        narrow["cacheable"].as_bool(),
        Some(true),
        "빈 목록은 cacheable 이다"
    );
}
