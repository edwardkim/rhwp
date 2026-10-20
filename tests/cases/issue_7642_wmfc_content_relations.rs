//! 원문 WMFC 내용과 독립 Print·Native/fresh WASM 전체 15쪽을 확인한 그림 계약.
//! 증적: mydocs/pr/assets/issue7642-wmfc-content-20261012/README.md
#![cfg(not(target_arch = "wasm32"))]

use base64::Engine;
use std::collections::BTreeSet;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn emf_in_wmf(bytes: &[u8]) -> Vec<u8> {
    let mut at = if bytes.starts_with(&0x9ac6cdd7u32.to_le_bytes()) {
        40
    } else {
        18
    };
    let mut emf = Vec::new();
    while at + 6 <= bytes.len() {
        let end = at + usize::try_from(u32_at(bytes, at)).unwrap() * 2;
        assert!(end > at && end <= bytes.len());
        if u16_at(bytes, at + 4) == 0x626 && end >= at + 10 && u16_at(bytes, at + 6) == 15 {
            let size = usize::from(u16_at(bytes, at + 8));
            let body = at + 10;
            assert!(body + size <= end);
            if size >= 34 && &bytes[body..body + 4] == b"WMFC" {
                emf.extend_from_slice(&bytes[body + 34..body + size]);
            }
        }
        at = end;
    }
    emf
}

struct SourceContent {
    pictures: usize,
    metafiles: usize,
    bitmaps: BTreeSet<Vec<u8>>,
}

fn source_content(sample: &Path) -> SourceContent {
    let mut archive = zip::ZipArchive::new(Cursor::new(std::fs::read(sample).unwrap())).unwrap();
    let mut result = SourceContent {
        pictures: 0,
        metafiles: 0,
        bitmaps: BTreeSet::new(),
    };
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if !entry
            .name()
            .expect("원본 WMFC ZIP의 유효한 파일명")
            .ends_with(".wmf")
        {
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        let emf = emf_in_wmf(&bytes);
        if emf.is_empty() {
            continue;
        }
        result.pictures += 1;
        let mut at = 0;
        while at + 8 <= emf.len() {
            let end = at + usize::try_from(u32_at(&emf, at + 4)).unwrap();
            assert!(end > at && end <= emf.len());
            let body = &emf[at + 8..end];
            if u32_at(&emf, at) == 70 && body.get(4..8) == Some(b"EMF+") {
                let comment = &body[4..4 + usize::try_from(u32_at(body, 0)).unwrap()];
                let mut pos = 4;
                while pos + 12 <= comment.len() {
                    let length = usize::try_from(u32_at(comment, pos + 4)).unwrap();
                    let size = usize::try_from(u32_at(comment, pos + 8)).unwrap();
                    let flags = u16_at(comment, pos + 2);
                    assert!(length >= 12 && size <= length - 12 && pos + length <= comment.len());
                    if u16_at(comment, pos) == 0x4008 && (flags >> 8) & 127 == 5 {
                        let data = &comment[pos + 12..pos + 12 + size];
                        match u32_at(data, 4) {
                            2 => result.metafiles += 1,
                            1 => {
                                assert_eq!(u32_at(data, 24), 1, "원문의 압축 bitmap 계약");
                                result.bitmaps.insert(data[28..].to_vec());
                            }
                            other => panic!("원문의 이미지 종류 변경: {other}"),
                        }
                    }
                    pos += length;
                }
            }
            at = end;
        }
    }
    assert_eq!(result.pictures, 3, "원문 도해의 소유 수량");
    assert_eq!(result.metafiles, 10, "원문이 담은 내부 벡터 그림 수량");
    assert_eq!(result.bitmaps.len(), 4, "원문 PNG/JPEG의 고유 내용");
    result
}

struct OutputDir(PathBuf);
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn export() -> (SourceContent, OutputDir, Vec<String>) {
    let sample =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue6469/wmf_fill_shapes.hwpx");
    let source = source_content(&sample);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = OutputDir(std::env::temp_dir().join(format!(
        "rhwp-wmfc-relations-{}-{nonce}",
        std::process::id()
    )));
    std::fs::create_dir(&dir.0).unwrap();
    // 수정 전 증명에서도 동일 검사로 실제 그 코드의 CLI 출력을 검사한다.
    let binary = std::env::var("RHWP_METAFILE_RELATIONS_BIN")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let output = std::process::Command::new(binary)
        .arg("export-svg")
        .arg(&sample)
        .args(["--profile", "print", "--json", "-o"])
        .arg(&dir.0)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "실제 그림 SVG 내보내기 실패: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["pageCount"], 15, "독립 Print 전체 쪽수");
    let mut roots = Vec::new();
    for entry in std::fs::read_dir(&dir.0).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "svg") {
            continue;
        }
        let svg = std::fs::read_to_string(path).unwrap();
        let doc = roxmltree::Document::parse(&svg).unwrap();
        for node in doc.descendants().filter(|n| n.has_tag_name("image")) {
            if let Some(uri) = node
                .attribute("href")
                .or_else(|| node.attribute(("http://www.w3.org/1999/xlink", "href")))
            {
                if let Some(data) = uri.strip_prefix("data:image/svg+xml;base64,") {
                    let inner = String::from_utf8(
                        base64::engine::general_purpose::STANDARD
                            .decode(data)
                            .unwrap(),
                    )
                    .unwrap();
                    if gray_nodes(&inner) > 0 {
                        roots.push(inner);
                    }
                }
            }
        }
    }
    assert_eq!(
        roots.len(),
        source.pictures,
        "본문 도해 소유의 누락·중복 없음"
    );
    (source, dir, roots)
}

fn gray_nodes(svg: &str) -> usize {
    let doc = roxmltree::Document::parse(svg).unwrap();
    doc.descendants()
        .filter(|node| {
            node.attribute("fill")
                .and_then(|s| s.strip_prefix('#'))
                .and_then(|s| u32::from_str_radix(s, 16).ok())
                == Some(0xd9d9d9)
        })
        .count()
}

fn image_contents(svg: &str, vectors: &mut BTreeSet<String>, bitmaps: &mut BTreeSet<Vec<u8>>) {
    let doc = roxmltree::Document::parse(svg).unwrap();
    for node in doc.descendants().filter(|n| n.has_tag_name("image")) {
        let Some(uri) = node
            .attribute("href")
            .or_else(|| node.attribute(("http://www.w3.org/1999/xlink", "href")))
        else {
            continue;
        };
        let Some((kind, data)) = uri.split_once(";base64,") else {
            continue;
        };
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(data)
            .unwrap();
        if kind == "data:image/svg+xml" {
            let inner = String::from_utf8(bytes).unwrap();
            let parsed = roxmltree::Document::parse(&inner).unwrap();
            if parsed.descendants().any(|n| n.has_tag_name("path")) {
                vectors.insert(inner.clone());
            }
            image_contents(&inner, vectors, bitmaps);
        } else if matches!(kind, "data:image/png" | "data:image/jpeg") {
            bitmaps.insert(bytes);
        }
    }
}

#[test]
fn wmfc_print_surface_preserves_its_source_vector_and_encoded_bitmap_contents() {
    let (source, _directory, roots) = export();
    let mut vectors = BTreeSet::new();
    let mut bitmaps = BTreeSet::new();
    for root in &roots {
        image_contents(root, &mut vectors, &mut bitmaps);
    }
    assert_eq!(
        vectors.len(),
        source.metafiles,
        "도해 내부 벡터 그림의 누락 없는 소유"
    );
    for bitmap in &source.bitmaps {
        assert!(
            bitmaps.contains(bitmap),
            "원문 PNG/JPEG의 색상·알파·encoded 내용 보존"
        );
    }
}

#[test]
fn source_gray_panels_remain_owned_once_regardless_of_svg_hex_letter_case() {
    let (_source, _directory, roots) = export();
    for root in &roots {
        assert_eq!(gray_nodes(root), 1, "원문 회색 패널의 단일 소유");
    }
}
