//! [Issue #4669 / #5450] HWPX 저장이 `hp:ole` 의 shape-component 자식
//! (`offset`·`orgSz`·`curSz`·`flip`·`rotationInfo`·`renderingInfo`·`lineShape`)과
//! `id` 속성을 원문 보존해야 한다.
//!
//! 종전 결함: `parse_common_shape_children` 에 arm 이 없고 `id` 는 instid 만
//! 읽어, 무편집 저장만으로 `id="0"` 재부여·`curSz=0` 재유도가 일어났다.
//! 한글 표시에는 영향이 없어도 offset·행렬을 읽는 다른 소비자는 오배치를 얻는다.
//!
//! 이 시험은 실물 `samples/한셀OLE.hwpx` 와
//! `tests/fixtures/issue_4669_ole_shape_component/` 코퍼스(XML+봉투)를
//! `DocumentCore::export_hwpx_native` 저장 경로로 왕복한다.
//! pic offset(#4668)·쪽수(#3737)·char_shapes 는 단언하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use rhwp::document_core::DocumentCore;
use serde_json::Value;

const SAMPLE: &str = "samples/한셀OLE.hwpx";
const FIXTURE_ROOT: &str = "tests/fixtures/issue_4669_ole_shape_component";

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn section_xml_of(bytes: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("ZIP 열기");
    let names: Vec<String> = zip.file_names().map(str::to_string).collect();
    let mut xml = String::new();
    for name in names {
        if name.starts_with("Contents/section") && name.ends_with(".xml") {
            zip.by_name(&name)
                .expect("section 엔트리")
                .read_to_string(&mut xml)
                .expect("section XML 은 UTF-8 이어야 한다");
        }
    }
    xml
}

fn extract_oles(section: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = section;
    while let Some(start) = rest.find("<hp:ole") {
        let tail = &rest[start..];
        let end = tail
            .find("</hp:ole>")
            .map(|i| i + "</hp:ole>".len())
            .or_else(|| tail.find("/>").map(|i| i + 2))
            .expect("hp:ole 닫기");
        out.push(tail[..end].to_string());
        rest = &tail[end..];
    }
    out
}

fn start_tag(ole: &str) -> &str {
    ole.split('>').next().unwrap_or(ole)
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let i = tag.find(&key)?;
    let rest = &tag[i + key.len()..];
    let j = rest.find('"')?;
    Some(rest[..j].to_string())
}

fn splice_oles_into_hansel_section(template_section: &str, fixture_section: &str) -> String {
    let oles = extract_oles(fixture_section);
    assert!(!oles.is_empty(), "픽스처에 hp:ole 이 없다");
    let start = template_section
        .find("<hp:ole")
        .expect("한셀 템플릿에 hp:ole");
    let tail = &template_section[start..];
    let end_rel = tail
        .find("</hp:ole>")
        .map(|i| i + "</hp:ole>".len())
        .expect("한셀 템플릿 ole 닫기");
    let mut out = String::with_capacity(template_section.len() + 256);
    out.push_str(&template_section[..start]);
    out.push_str(&oles.join(""));
    out.push_str(&tail[end_rel..]);
    out
}

fn pack_section_into_hansel(fixture_section: &str) -> Vec<u8> {
    let template =
        std::fs::read(repo_path(SAMPLE)).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let template_section = section_xml_of(&template);
    let spliced = splice_oles_into_hansel_section(&template_section, fixture_section);
    let mut src = zip::ZipArchive::new(std::io::Cursor::new(template)).expect("template ZIP");
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let deflated = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let names: Vec<String> = src.file_names().map(str::to_string).collect();
    for name in names {
        let mut data = Vec::new();
        src.by_name(&name)
            .expect("엔트리")
            .read_to_end(&mut data)
            .expect("읽기");
        let opts = if name == "mimetype" { stored } else { deflated };
        out.start_file(&name, opts).expect("start_file");
        if name == "Contents/section0.xml" {
            out.write_all(spliced.as_bytes()).expect("section 교체");
        } else {
            out.write_all(&data).expect("copy");
        }
    }
    out.finish().expect("finish").into_inner()
}

fn export_section(bytes: &[u8]) -> String {
    let doc = DocumentCore::from_bytes(bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    let exported = doc
        .export_hwpx_native()
        .unwrap_or_else(|e| panic!("export: {e:?}"));
    section_xml_of(&exported)
}

#[test]
fn issue_4669_hansel_sample_preserves_ole_id_and_shape_component() {
    let bytes = std::fs::read(repo_path(SAMPLE)).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let original = section_xml_of(&bytes);
    let oles = extract_oles(&original);
    assert_eq!(oles.len(), 1, "한셀OLE 전제: hp:ole 1개");
    let ole0 = &oles[0];
    assert_eq!(
        attr_value(start_tag(ole0), "id").as_deref(),
        Some("2141242094"),
        "샘플 전제: id≠instid"
    );
    assert_eq!(
        attr_value(start_tag(ole0), "instid").as_deref(),
        Some("1067500271")
    );
    assert!(
        ole0.contains(r#"<hp:orgSz width="42001" height="13501"/>"#),
        "샘플 전제 orgSz"
    );
    assert!(
        ole0.contains(r#"<hp:curSz width="29999" height="4051"/>"#),
        "샘플 전제 curSz (비-0)"
    );
    assert!(
        ole0.contains(r#"<hp:offset x="0" y="0"/>"#),
        "샘플 전제 offset"
    );
    assert!(ole0.contains(r#"e1="0.714245""#), "샘플 전제 scaMatrix");

    let saved = export_section(&bytes);
    let saved_oles = extract_oles(&saved);
    assert_eq!(saved_oles.len(), 1, "저장 후에도 hp:ole 1개");
    let s = &saved_oles[0];
    assert_eq!(
        attr_value(start_tag(s), "id").as_deref(),
        Some("2141242094"),
        "id 원문 보존: {s}"
    );
    assert_eq!(
        attr_value(start_tag(s), "instid").as_deref(),
        Some("1067500271"),
        "instid 원문 보존: {s}"
    );
    assert!(
        s.contains(r#"<hp:orgSz width="42001" height="13501"/>"#),
        "orgSz: {s}"
    );
    assert!(
        s.contains(r#"<hp:curSz width="29999" height="4051"/>"#),
        "curSz 재유도 금지: {s}"
    );
    assert!(s.contains(r#"<hp:offset x="0" y="0"/>"#), "offset: {s}");
    assert!(
        s.contains(r#"<hp:flip horizontal="0" vertical="0"/>"#),
        "flip: {s}"
    );
    assert!(
        s.contains(r#"rotateimage="1""#),
        "rotationInfo rotateimage: {s}"
    );
    assert!(
        s.contains(r#"e1="0.714245""#),
        "renderingInfo scaMatrix: {s}"
    );
    assert!(s.contains(r#"style="NONE""#), "lineShape: {s}");
}

#[test]
fn issue_4669_fixture_envelopes_roundtrip_shape_component() {
    let env_dir = repo_path(&format!("{FIXTURE_ROOT}/envelopes"));
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&env_dir)
        .unwrap_or_else(|e| panic!("envelopes: {e}"))
        .map(|e| e.expect("dirent").path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    paths.sort();
    assert!(
        paths.len() >= 100,
        "코퍼스 하한: envelopes {} < 100",
        paths.len()
    );

    let mut checked = 0usize;
    for env_path in &paths {
        let fid = env_path.file_stem().unwrap().to_string_lossy();
        let env: Value = serde_json::from_str(
            &std::fs::read_to_string(env_path).unwrap_or_else(|e| panic!("read {fid}: {e}")),
        )
        .unwrap_or_else(|e| panic!("json {fid}: {e}"));
        assert_eq!(env["issue"], 4669, "{fid}");
        assert_eq!(
            env["schema"], "rhwp.issue4669.ole-shape-component.v1",
            "{fid}"
        );

        let xml_path = repo_path(&format!("{FIXTURE_ROOT}/xml/{fid}.xml"));
        let section =
            std::fs::read_to_string(&xml_path).unwrap_or_else(|e| panic!("xml {fid}: {e}"));
        let packed = pack_section_into_hansel(&section);
        let saved = export_section(&packed);
        let saved_oles = extract_oles(&saved);
        let expected = env["oles"].as_array().expect("oles");
        assert_eq!(
            saved_oles.len(),
            expected.len(),
            "{fid}: ole 개수 saved={} expected={}",
            saved_oles.len(),
            expected.len()
        );

        for (i, (ole, spec)) in saved_oles.iter().zip(expected.iter()).enumerate() {
            let tag = start_tag(ole);
            let want_id = spec["save_id"]
                .as_u64()
                .or_else(|| spec["save_id"].as_i64().map(|v| v as u64))
                .expect("save_id");
            let got_id = attr_value(tag, "id").unwrap_or_default();
            assert_eq!(
                got_id,
                want_id.to_string(),
                "{fid}[{i}] id 원문 보존: {ole}"
            );
            let want_inst = spec["instance_id"]
                .as_u64()
                .or_else(|| spec["instance_id"].as_i64().map(|v| v as u64))
                .expect("instance_id");
            let got_inst = attr_value(tag, "instid").unwrap_or_default();
            assert_eq!(
                got_inst,
                want_inst.to_string(),
                "{fid}[{i}] instid 원문 보존: {ole}"
            );
            if spec["forbid_id_zero"].as_bool() == Some(true) {
                assert_ne!(got_id, "0", "{fid}[{i}] id 가 0 으로 재부여됨: {ole}");
            }
            for frag in spec["expect_xml"].as_array().expect("expect_xml") {
                let frag = frag.as_str().expect("expect frag");
                if frag.starts_with("id=\"") || frag.starts_with("instid=\"") {
                    assert!(
                        tag.contains(frag),
                        "{fid}[{i}] 시작 태그에 {frag} 없음: {tag}"
                    );
                } else {
                    assert!(
                        ole.contains(frag),
                        "{fid}[{i}] 기대 조각 없음 {frag}: {ole}"
                    );
                }
            }
            for frag in spec["forbid_xml"].as_array().expect("forbid_xml") {
                let frag = frag.as_str().expect("forbid frag");
                assert!(
                    !ole.contains(frag),
                    "{fid}[{i}] 금지 조각이 저장됨 {frag}: {ole}"
                );
            }
            checked += 1;
        }
    }
    assert!(checked >= 100, "검사한 ole {checked} < 100");
}

#[test]
fn issue_4669_issue_body_cursz_zero_is_not_materialized_on_save() {
    let xml = std::fs::read_to_string(repo_path(&format!(
        "{FIXTURE_ROOT}/xml/152_combo_issue_body_xml.xml"
    )))
    .expect("issue-body fixture");
    let saved = export_section(&pack_section_into_hansel(&xml));
    let ole = &extract_oles(&saved)[0];
    assert_eq!(
        attr_value(start_tag(ole), "id").as_deref(),
        Some("2141242094")
    );
    assert_eq!(
        attr_value(start_tag(ole), "instid").as_deref(),
        Some("1067500271")
    );
    assert!(
        ole.contains(r#"<hp:curSz width="0" height="0"/>"#),
        "이슈 본문 재현: curSz=0 복원 {ole}"
    );
    assert!(
        !ole.contains(r#"<hp:curSz width="42001" height="13501"/>"#),
        "curSz=0 이 orgSz 로 재유도되면 안 된다: {ole}"
    );
    assert!(
        ole.contains(r#"<hp:offset x="12" y="34"/>"#),
        "offset 12,34: {ole}"
    );
}

#[test]
fn issue_4669_catalog_and_envelopes_are_paired() {
    let xml_dir = repo_path(&format!("{FIXTURE_ROOT}/xml"));
    let env_dir = repo_path(&format!("{FIXTURE_ROOT}/envelopes"));
    let catalog = std::fs::read_to_string(repo_path(&format!("{FIXTURE_ROOT}/catalog.tsv")))
        .expect("catalog.tsv");
    let mut xmls: Vec<String> = std::fs::read_dir(xml_dir)
        .unwrap()
        .map(|e| {
            e.unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let mut envs: Vec<String> = std::fs::read_dir(env_dir)
        .unwrap()
        .map(|e| {
            e.unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    xmls.sort();
    envs.sort();
    assert_eq!(xmls, envs, "xml 과 envelope 파일 stem 이 1:1 이어야 한다");
    for id in &xmls {
        assert!(catalog.contains(id), "catalog.tsv 에 {id} 가 없다");
    }
}
