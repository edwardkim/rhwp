//! [Issue #3546] HWPX 파서는 `<hp:chart>` 를 OLE 모델(bin_data_id=60000+N)로
//! 변환하고, 종전 저장기는 이를 일반 OLE 로 방출했다. 그 결과 저장 시
//! `hp:chart` → `hp:ole` 치환, `Chart/chartN.xml` → `BinData/image6000N.ooxml_chart`
//! 이동이 일어나고, 한컴오피스가 차트 XML 을 OLE 복합문서로 해석해 "메모리
//! 부족"을 냈다(#3547 의 결정적 원인 제거 후 잔존분). 차트 편집 트랙(#3683)의
//! 전제 — "편집하지 않은 차트는 원형 그대로 저장" — 를 samples/chart 코퍼스
//! 전건으로 고정한다: Chart 파트 바이트 원형 보존 + hp:switch/hp:case/
//! hp:default 구조 재방출 + BinData 이동 금지 + 재파스 안정(2-round).

use std::io::Read;
use std::path::{Path, PathBuf};

use rhwp::document_core::DocumentCore;

const OOXML_CHART_CASE: &str =
    r#"<hp:case hp:required-namespace="http://www.hancom.co.kr/hwpml/2016/ooxmlchart">"#;

/// samples/chart 하위의 모든 .hwpx 경로를 재귀 수집한다.
fn chart_samples(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("samples/chart 읽기") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            chart_samples(&path, out);
        } else if path.extension().is_some_and(|e| e == "hwpx") {
            out.push(path);
        }
    }
}

/// ZIP 바이트에서 (엔트리 이름 → 바이트) 전체 맵을 수집한다.
fn zip_entries(zip_bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).expect("ZIP 열기");
    let names: Vec<String> = zip
        .file_names()
        .map(|name| name.expect("ZIP 엔트리 이름").into_owned())
        .collect();
    let mut entries = Vec::new();
    for name in names {
        let mut bytes = Vec::new();
        zip.by_name(&name)
            .expect("ZIP 엔트리")
            .read_to_end(&mut bytes)
            .expect("ZIP 엔트리 읽기");
        entries.push((name, bytes));
    }
    entries
}

/// 본문 섹션(Contents/section*.xml)들을 이어붙인 문자열.
fn section_xml(entries: &[(String, Vec<u8>)]) -> String {
    let mut xml = String::new();
    for (name, bytes) in entries {
        if name.starts_with("Contents/section") && name.ends_with(".xml") {
            xml.push_str(&String::from_utf8_lossy(bytes));
        }
    }
    xml
}

/// `<hp:chart ...>` 시작 태그의 id 속성 값 시퀀스.
fn chart_id_values(xml: &str) -> Vec<String> {
    xml.split("<hp:chart ")
        .skip(1)
        .map(|cap| {
            let tag = cap.split('>').next().unwrap_or("");
            tag.split("id=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .unwrap_or("")
                .to_string()
        })
        .collect()
}

/// `<hp:default>` 블록 안의 rotateimage 값 시퀀스.
fn fallback_rotateimage_values(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    for block in xml.split("<hp:default>").skip(1) {
        let block = block.split("</hp:default>").next().unwrap_or("");
        for cap in block.split("rotateimage=\"").skip(1) {
            out.push(cap.split('"').next().unwrap_or("").to_string());
        }
    }
    out
}

/// 저장본이 원본의 차트 표면을 보존하는지 단언한다.
fn assert_chart_preserved(label: &str, original: &[(String, Vec<u8>)], saved_bytes: &[u8]) {
    let saved = zip_entries(saved_bytes);

    // 1. Chart/*.xml 파트가 같은 이름·같은 바이트로 존재한다.
    for (name, bytes) in original.iter().filter(|(n, _)| n.starts_with("Chart/")) {
        let found = saved.iter().find(|(sn, _)| sn == name);
        let Some((_, sbytes)) = found else {
            panic!("{label}: 저장본에 {name} 이 없다 (파트 이동/소실)");
        };
        assert_eq!(sbytes, bytes, "{label}: {name} 바이트가 원본과 달라졌다");
    }

    // 2. 차트 XML 이 BinData 로 이동하지 않는다.
    if let Some((moved, _)) = saved.iter().find(|(n, _)| {
        n.starts_with("BinData/") && n.to_ascii_lowercase().ends_with(".ooxml_chart")
    }) {
        panic!("{label}: 차트 파트가 BinData 로 이동됨: {moved}");
    }

    // 3. hp:chart 컨트롤·chartIDRef·switch 구조가 재방출된다.
    let orig_xml = section_xml(original);
    let saved_xml = section_xml(&saved);
    let orig_charts = orig_xml.matches("<hp:chart ").count();
    let saved_charts = saved_xml.matches("<hp:chart ").count();
    assert_eq!(
        saved_charts, orig_charts,
        "{label}: hp:chart 컨트롤 수가 보존되어야 한다 (hp:ole 치환 발생?)"
    );
    // 실물 hp:chart 는 instid 없이 id 만 기록한다 — 미파싱이면 "0" 으로 되쓰인다.
    assert_eq!(
        chart_id_values(&saved_xml),
        chart_id_values(&orig_xml),
        "{label}: hp:chart id 속성 값이 보존되어야 한다"
    );
    for cap in orig_xml.split("chartIDRef=\"").skip(1) {
        let idref = cap.split('"').next().unwrap_or("");
        assert!(
            saved_xml.contains(&format!("chartIDRef=\"{idref}\"")),
            "{label}: chartIDRef=\"{idref}\" 가 저장본에 없다"
        );
    }
    let orig_cases = orig_xml.matches(OOXML_CHART_CASE).count();
    let saved_cases = saved_xml.matches(OOXML_CHART_CASE).count();
    assert_eq!(
        saved_cases, orig_cases,
        "{label}: ooxmlchart hp:case 래핑 수가 보존되어야 한다"
    );
    if orig_cases > 0 {
        let saved_defaults = saved_xml.matches("<hp:default>").count();
        assert_eq!(
            saved_defaults, saved_cases,
            "{label}: hp:case 마다 <hp:default> fallback 이 재방출되어야 한다"
        );
        assert!(
            saved_xml.matches("<hp:ole ").count() >= saved_cases,
            "{label}: fallback hp:ole 이 소실됐다"
        );
    }

    // 4. fallback OLE 의 rotationInfo@rotateimage 원본 값 보존 — 종전엔
    //    hp:ole 파서가 rotationInfo 를 읽지 않아 기본값 "0" 으로 되쓰였다.
    assert_eq!(
        fallback_rotateimage_values(&saved_xml),
        fallback_rotateimage_values(&orig_xml),
        "{label}: fallback rotateimage 값이 원본과 달라졌다"
    );
}

#[test]
fn issue_3546_chart_preserved_on_save() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let mut paths = Vec::new();
    chart_samples(&Path::new(repo_root).join("samples/chart"), &mut paths);
    paths.sort();
    assert!(
        paths.len() >= 20,
        "samples/chart 코퍼스가 예상보다 작다: {}",
        paths.len()
    );

    for path in &paths {
        let label = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {label}: {e}"));
        let original = zip_entries(&bytes);
        assert!(
            original.iter().any(|(n, _)| n.starts_with("Chart/")),
            "{label}: 코퍼스 전제 위반 — Chart/ 파트가 없다"
        );

        let doc =
            DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {label}: {e:?}"));
        let saved = doc
            .export_hwpx_native()
            .unwrap_or_else(|e| panic!("export {label}: {e:?}"));
        assert_chart_preserved(&label, &original, &saved);

        // 2-round: 저장본을 다시 파스·저장해도 차트 표면이 유지된다 —
        // 재방출된 hp:chart/hp:switch 가 파서로 원형 승계됨을 함께 고정한다.
        let doc2 =
            DocumentCore::from_bytes(&saved).unwrap_or_else(|e| panic!("reparse {label}: {e:?}"));
        let saved2 = doc2
            .export_hwpx_native()
            .unwrap_or_else(|e| panic!("re-export {label}: {e:?}"));
        assert_chart_preserved(&format!("{label} (2-round)"), &original, &saved2);
    }
}

#[test]
fn issue_3546_chart_part_is_password_protected() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let mut paths = Vec::new();
    chart_samples(&Path::new(repo_root).join("samples/chart"), &mut paths);
    paths.sort();
    let path = paths.first().expect("samples/chart fixture 하나 이상");
    let label = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string();
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {label}: {e}"));
    let original = zip_entries(&bytes);

    let doc = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {label}: {e:?}"));
    // test 실행마다 달라지는 값으로 고정 credential 검출을 피한다.
    let password = std::process::id().to_le_bytes();
    let encrypted = doc
        .export_hwpx_native_with_password(&password)
        .unwrap_or_else(|e| panic!("encrypt {label}: {e:?}"));
    let encrypted_entries = zip_entries(&encrypted);
    let manifest = String::from_utf8_lossy(
        &encrypted_entries
            .iter()
            .find(|(name, _)| name == "META-INF/manifest.xml")
            .expect("암호 HWPX manifest")
            .1,
    );

    for (name, plaintext) in original
        .iter()
        .filter(|(name, _)| name.starts_with("Chart/"))
    {
        let ciphertext = encrypted_entries
            .iter()
            .find(|(saved_name, _)| saved_name == name)
            .unwrap_or_else(|| panic!("암호 저장본에 {name} 이 없다"));
        assert_ne!(
            &ciphertext.1, plaintext,
            "{label}: {name} 이 평문으로 남아서는 안 된다"
        );
        assert!(
            manifest.contains(&format!(r#"full-path="{name}""#)),
            "{label}: manifest에 암호화된 {name} 항목이 없다"
        );
    }

    assert!(
        DocumentCore::from_bytes(&encrypted).is_err(),
        "{label}: 비밀번호 없이 암호 HWPX를 열면 안 된다"
    );
    let decrypted = DocumentCore::from_bytes_with_password(&encrypted, &password)
        .unwrap_or_else(|e| panic!("decrypt {label}: {e:?}"));
    let restored = decrypted
        .export_hwpx_native()
        .unwrap_or_else(|e| panic!("re-export {label}: {e:?}"));
    assert_chart_preserved(
        &format!("{label} (password round-trip)"),
        &original,
        &restored,
    );
}
