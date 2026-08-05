// 스윕 2차: 그림 op 바이트의 실제 매직 스니핑 + pageBackground 이미지 검사.
//
// (1) 페이지의 source image key 로 studio 가 내보내는 바이트를 받아 매직을 직접 판별한다
//     — detect_image_mime_type 이 octet-stream 으로 떨어뜨린 바이트의 정체 확인용.
// (2) layer JSON 의 pageBackground.image.base64 는 mime 없이 원본 그대로 나가므로
//     base64 앞부분을 복호해 포맷을 판별한다.
//
// 사용: audit_image_magic <file>...   → 문서당 JSONL 1줄
use std::collections::BTreeMap;

fn sniff(data: &[u8]) -> String {
    if data.len() >= 8 && data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return "png".into();
    }
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return "jpeg".into();
    }
    if data.starts_with(b"GIF8") {
        return "gif".into();
    }
    if data.starts_with(b"BM") {
        return "bmp".into();
    }
    if data.starts_with(&[0xD7, 0xCD, 0xC6, 0x9A]) || data.starts_with(&[0x01, 0x00, 0x09, 0x00]) {
        return "wmf".into();
    }
    if data.len() >= 44 && data.starts_with(&[0x01, 0x00, 0x00, 0x00]) && &data[40..44] == b" EMF" {
        return "emf".into();
    }
    if data.starts_with(&[0x49, 0x49, 0x2A, 0x00]) || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A]) {
        return "tiff".into();
    }
    if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        return "webp".into();
    }
    if data.starts_with(&[0x0A, 0x05]) {
        return "pcx".into();
    }
    let head = data
        .iter()
        .take(12)
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    // SVG/XML 텍스트 여부
    let text = String::from_utf8_lossy(&data[..data.len().min(64)]);
    if text.trim_start().starts_with("<?xml") || text.trim_start().starts_with("<svg") {
        return "svg".into();
    }
    format!("unknown:{head}")
}

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    use base64::Engine;
    for path in &files {
        let run = std::panic::catch_unwind(|| -> Result<String, String> {
            let bytes = std::fs::read(path).map_err(|e| format!("read: {e}"))?;
            let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
                .map_err(|e| format!("parse: {e:?}"))?;
            let mut op_magics: BTreeMap<String, u32> = BTreeMap::new();
            let mut bg_magics: BTreeMap<String, u32> = BTreeMap::new();
            for page in 0..doc.page_count() {
                if let Ok(keys_json) = doc.get_page_source_image_keys(page) {
                    // {"cacheable":..,"keys":["k1","k2",null,...]}
                    for key in keys_json.split('"').skip(1).step_by(2) {
                        if key == "cacheable" || key == "keys" {
                            continue;
                        }
                        if let Ok(data) = doc.get_source_image_bytes(key) {
                            *op_magics.entry(sniff(&data)).or_insert(0) += 1;
                        }
                    }
                }
                if let Ok(json) = doc.get_page_layer_tree_with_profile(page, "screen", Some(true)) {
                    let needle = "\"type\":\"pageBackground\"";
                    let mut idx = 0;
                    while let Some(p) = json[idx..].find(needle) {
                        let start = idx + p;
                        // 이 op 객체 범위 안의 image base64 를 찾는다 (다음 op 전까지).
                        let end = json[start + 1..]
                            .find("\"type\":\"")
                            .map(|e| start + 1 + e)
                            .unwrap_or(json.len());
                        let scope = &json[start..end];
                        if let Some(bp) = scope.find("\"base64\":\"") {
                            let b64 = &scope[bp + 10..];
                            let prefix: String =
                                b64.chars().take(64).take_while(|c| *c != '"').collect();
                            let take = prefix.len() - prefix.len() % 4;
                            match base64::engine::general_purpose::STANDARD.decode(&prefix[..take])
                            {
                                Ok(decoded) => *bg_magics.entry(sniff(&decoded)).or_insert(0) += 1,
                                Err(_) => *bg_magics.entry("b64err".into()).or_insert(0) += 1,
                            }
                        }
                        idx = end;
                    }
                }
            }
            let fmt = |m: &BTreeMap<String, u32>| {
                m.iter()
                    .map(|(k, v)| format!("\"{k}\":{v}"))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            Ok(format!(
                "\"ok\":true,\"ops\":{{{}}},\"pageBg\":{{{}}}",
                fmt(&op_magics),
                fmt(&bg_magics)
            ))
        });
        let body = match run {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => format!("\"ok\":false,\"error\":\"{}\"", e.replace('"', "'")),
            Err(_) => "\"ok\":false,\"error\":\"panic\"".to_string(),
        };
        println!("{{\"file\":\"{}\",{body}}}", path.replace('"', "'"));
    }
}
