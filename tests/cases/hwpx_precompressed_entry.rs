//! 미리 deflate 한 데이터로 쓴 HWPX ZIP 엔트리는, 같은 압축 바이트라면 일반 경로로 쓴
//! 엔트리와 바이트 단위로 같다(결정론 메타데이터 — mtime·호스트·권한 — 를 공유).

use std::io::{Cursor, Read};

use rhwp::serializer::hwpx::writer::HwpxZipWriter;

fn payload() -> Vec<u8> {
    // 압축이 되는 내용과 안 되는 내용을 섞는다.
    let mut data = b"<hp:p>".repeat(2000);
    data.extend((0..20_000u32).map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8));
    data
}

fn normal_archive(name: &str, data: &[u8]) -> Vec<u8> {
    let mut zip = HwpxZipWriter::new();
    zip.write_stored("mimetype", b"application/hwp+zip")
        .unwrap();
    zip.write_deflated(name, data).unwrap();
    zip.finish().unwrap()
}

#[test]
fn precompressed_entry_matches_normal_entry_bytes() {
    let name = "BinData/image1.bmp";
    let data = payload();
    let normal = normal_archive(name, &data);

    // 일반 경로가 만든 엔트리의 압축 바이트·CRC 를 그대로 다시 쓴다.
    let mut archive = zip::ZipArchive::new(Cursor::new(normal.clone())).unwrap();
    let (crc32, size, raw) = {
        let index = archive.index_for_name(name).unwrap();
        let mut entry = archive.by_index_raw(index).unwrap();
        let mut raw = Vec::new();
        entry.read_to_end(&mut raw).unwrap();
        (entry.crc32(), entry.size(), raw)
    };
    assert!(raw.len() < data.len(), "검사 데이터가 실제로 압축돼야 한다");

    let mut zip = HwpxZipWriter::new();
    zip.write_stored("mimetype", b"application/hwp+zip")
        .unwrap();
    zip.write_precompressed_deflated(name, crc32, size, &raw)
        .unwrap();
    let precompressed = zip.finish().unwrap();

    assert!(
        precompressed == normal,
        "미리 압축한 엔트리의 ZIP 바이트가 일반 경로와 다르다"
    );

    let mut reread = zip::ZipArchive::new(Cursor::new(precompressed)).unwrap();
    let mut entry = reread.by_name(name).unwrap();
    let mut back = Vec::new();
    entry.read_to_end(&mut back).unwrap();
    assert_eq!(back, data);
}
