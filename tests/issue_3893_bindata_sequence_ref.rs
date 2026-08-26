//! [Issue #3893/#4049] HWP5 그림 BinData 참조는 BIN_DATA **레코드 순번**(1-based)
//! 이고, 스트림 이름을 정하는 storage_id 와는 별개 축이다. 편집으로 storage id 에
//! 구멍이 난 문서에서 두 축이 어긋나면 rhwp(storage_id 정본 키)가 참조를 못 풀어
//! HWPX 저장 시 pic 컨트롤이 통째로 드롭됐다("binaryItemIDRef 미등록") — 파스
//! 말미의 순번→storage_id 재사상으로 닫는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::parse_document;

fn read_sample(rel: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn count_pics(doc: &rhwp::model::document::Document) -> usize {
    fn walk(paragraphs: &[rhwp::model::paragraph::Paragraph], n: &mut usize) {
        for p in paragraphs {
            for c in &p.controls {
                match c {
                    Control::Picture(_) => *n += 1,
                    Control::Table(t) => {
                        for cell in &t.cells {
                            walk(&cell.paragraphs, n);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let mut n = 0;
    for sec in &doc.sections {
        walk(&sec.paragraphs, &mut n);
    }
    n
}

/// 어긋난 문서의 HWPX 저장이 참조를 전부 해소해야 한다 — 종전에는
/// "binaryItemIDRef 미등록" 으로 pic 이 드롭됐다. 모델 IR 은 손대지 않고
/// (h2h 왕복 대칭성 보존) 저장 시점의 순번 축 폴백으로 푼다.
#[test]
fn gapped_storage_ids_resolve_at_export() {
    for rel in ["samples/pic-crop-01.hwp", "samples/exam_social.hwp"] {
        let bytes = read_sample(rel);
        let doc = parse_document(&bytes).expect("parse");
        // 샘플 전제: 순번↔storage 가 실제로 어긋나 있어야 한다.
        let storage_ids: Vec<u16> = doc
            .doc_info
            .bin_data_list
            .iter()
            .map(|b| b.storage_id)
            .collect();
        assert!(
            storage_ids
                .iter()
                .enumerate()
                .any(|(i, &sid)| sid as usize != i + 1),
            "{rel}: 샘플 전제 — storage id 구멍"
        );

        let core = DocumentCore::from_bytes(&bytes).expect("open");
        let hwpx = core.export_hwpx_native().expect("export");
        // 저장본의 모든 binaryItemIDRef 가 manifest 항목으로 해소돼야 한다.
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&hwpx)).expect("zip");
        let mut hpf = String::new();
        {
            use std::io::Read;
            zip.by_name("Contents/content.hpf")
                .expect("hpf")
                .read_to_string(&mut hpf)
                .expect("read");
        }
        let names: Vec<String> = zip.file_names().map(str::to_string).collect();
        for name in names.iter().filter(|n| n.starts_with("Contents/section")) {
            use std::io::Read;
            let mut xml = String::new();
            zip.by_name(name).unwrap().read_to_string(&mut xml).unwrap();
            for m in xml.split("binaryItemIDRef=\"").skip(1) {
                let idref = m.split('"').next().unwrap_or("");
                assert!(
                    !idref.is_empty(),
                    "{rel}: 빈 binaryItemIDRef — 참조 미해소 (#3893)"
                );
                assert!(
                    hpf.contains(&format!("id=\"{idref}\"")),
                    "{rel}: 본문 참조 {idref} 가 manifest 에 없어야 할 이유가 없다"
                );
            }
        }
    }
}

/// HWPX 왕복에서 pic 컨트롤이 드롭되지 않아야 한다 (#3893/#4049 의 실증상).
#[test]
fn pics_survive_hwpx_roundtrip() {
    for rel in ["samples/pic-crop-01.hwp", "samples/exam_social.hwp"] {
        let bytes = read_sample(rel);
        let original = parse_document(&bytes).expect("parse");
        let before = count_pics(&original);
        assert!(before > 0, "{rel}: 샘플 전제 — pic 존재");

        let core = DocumentCore::from_bytes(&bytes).expect("open");
        let hwpx = core.export_hwpx_native().expect("export");
        let roundtripped = parse_document(&hwpx).expect("reparse");
        assert_eq!(
            count_pics(&roundtripped),
            before,
            "{rel}: pic 컨트롤이 왕복에서 드롭되면 안 된다 (#3893/#4049)"
        );
    }
}
