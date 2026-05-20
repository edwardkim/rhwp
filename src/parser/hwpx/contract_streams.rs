//! [Task #852] HWPX → HWP OLE contract 스트림 변환
//!
//! 한컴 HWP 5.0 정답지는 다음 9 스트림 contract 를 요구한다:
//!
//! ```text
//! FileHeader, DocInfo, BodyText/Section0,        // 본문 (rhwp 기본 작성)
//! HwpSummaryInformation, DocOptions/_LinkDoc,    // 메타 (본 모듈)
//! Scripts/DefaultJScript, Scripts/JScriptVersion, // 스크립트 (본 모듈)
//! PrvImage, PrvText                              // 미리보기 (본 모듈)
//! ```
//!
//! HWPX 컨테이너에 동등 데이터가 있으면 (Preview, Scripts) 변환·passthrough,
//! 없으면 (HwpSummary, DocOptions/_LinkDoc, Scripts/JScriptVersion) 정적
//! fallback (`saved/blank2010.hwp` 추출) 사용.
//!
//! Stage 2.1 = HWPX 컨테이너 → extra_streams (정공법, Preview/Scripts).
//! Stage 2.2 (본 모듈) = 정적 fallback (HwpSummary/DocOptions/JScriptVersion).

use super::reader::HwpxReader;

// [Task #852 Stage 2.2] 정적 fallback 자산.
//
// HWPX 컨테이너에 동등 데이터가 없는 contract 스트림 3 개를 `samples/form-01.hwp`
// (한컴 정답지) 및 `saved/blank2010.hwp` 에서 사전 추출. 변환 결과가 한컴
// 호환을 보장하기 위한 최소 contract.
//
// - hwp_summary_information.bin (461 B) — `samples/form-01.hwp` 추출.
//   title/creator/subject 등 OLE Property Set. HWPX content.hpf opf:metadata
//   기반 패치는 후속 task.
// - doc_options_link_doc.bin (524 B) — `saved/blank2010.hwp` 추출. UTF-16 LE
//   임시 파일 경로 메타.
// - scripts_jscript_version.bin (13 B) — `saved/blank2010.hwp` 추출.
//   `cd 64 80 00 ...` 13 바이트 헤더.
const FALLBACK_HWP_SUMMARY: &[u8] = include_bytes!("blank2010_assets/hwp_summary_information.bin");
const FALLBACK_DOC_OPTIONS_LINK_DOC: &[u8] =
    include_bytes!("blank2010_assets/doc_options_link_doc.bin");
const FALLBACK_SCRIPTS_JSCRIPT_VERSION: &[u8] =
    include_bytes!("blank2010_assets/scripts_jscript_version.bin");
/// [Task #852 Stage 2.2] HWPX 컨테이너에 `Scripts/sourceScripts` 가 없는 경우
/// (form-002.hwpx 등 다수 케이스) 의 빈 스크립트 fallback. `saved/blank2010.hwp`
/// 추출 — 한컴 정답지가 모든 HWP 에 Scripts/DefaultJScript 요구하므로 contract
/// 보장용. 16 byte 빈 스크립트.
const FALLBACK_SCRIPTS_DEFAULT_JSCRIPT: &[u8] =
    include_bytes!("blank2010_assets/scripts_default_jscript.bin");
/// HWPX 컨테이너에 `Preview/PrvText.txt` 가 없을 때 fallback (4 byte).
const FALLBACK_PRV_TEXT: &[u8] = include_bytes!("blank2010_assets/prvtext.bin");
/// HWPX 컨테이너에 `Preview/PrvImage.png` 가 없을 때 fallback (5126 byte, blank PNG).
const FALLBACK_PRV_IMAGE: &[u8] = include_bytes!("blank2010_assets/prvimage.bin");

/// HWPX 컨테이너 → HWP OLE 스트림 매핑 결과
pub(super) struct ContractStreams {
    /// `Vec<(path, data)>` 형태 — Document::extra_streams 에 그대로 주입 가능
    pub streams: Vec<(String, Vec<u8>)>,
}

/// HWPX ZIP reader 로부터 contract 스트림 4 개를 추출/변환.
///
/// - `Preview/PrvText.txt` (UTF-8) → `/PrvText` (UTF-16 LE)
/// - `Preview/PrvImage.png` → `/PrvImage` (passthrough)
/// - `Scripts/sourceScripts` → `/Scripts/DefaultJScript` (zlib deflate)
/// - `Scripts/headerScripts` → 정적 fallback 사용 (Stage 2.2)
///
/// HWPX 에 동등 파일이 없으면 해당 스트림은 생략. cfb_writer 가 한컴 정답지
/// 와 비교하여 추가 fallback (HwpSummary / DocOptions/_LinkDoc) 가 필요.
pub(super) fn extract_contract_streams(reader: &mut HwpxReader) -> ContractStreams {
    let mut streams = Vec::new();

    // PrvText.txt (UTF-8) → /PrvText (UTF-16 LE, HWP5 spec).
    // HWPX 에 Preview 가 없으면 fallback (blank2010 추출).
    let prv_text = match reader.read_file("Preview/PrvText.txt") {
        Ok(utf8) => utf8.encode_utf16().flat_map(|c| c.to_le_bytes()).collect(),
        Err(_) => FALLBACK_PRV_TEXT.to_vec(),
    };
    streams.push(("/PrvText".to_string(), prv_text));

    // PrvImage.png → /PrvImage (PNG passthrough). 미존재 시 blank2010 fallback.
    let prv_image = reader
        .read_file_bytes("Preview/PrvImage.png")
        .unwrap_or_else(|_| FALLBACK_PRV_IMAGE.to_vec());
    streams.push(("/PrvImage".to_string(), prv_image));

    // [Task #852 Stage 2.5] HWPX Scripts/{headerScripts, sourceScripts} →
    // /Scripts/DefaultJScript (HWP5 spec, raw deflate).
    //
    // 정답지 (samples/form-01.hwp) reverse engineering 결과 구조 (raw uncompressed):
    //   0..4        u32 LE = headerScripts wchar count
    //   4..L1+4     headerScripts (UTF-16 LE, L1 bytes) — HWPX 와 byte-identical
    //   L1+4..L1+8  u32 LE = sourceScripts wchar count
    //   L1+8..L1+L2+8  sourceScripts (UTF-16 LE, L2 bytes) — HWPX 와 byte-identical
    //   tail        zero(8) + 0xffffffff(4) = 12 bytes
    //
    // 압축은 raw deflate (zlib 헤더 없음). Stage 2.1 의 단순 zlib_deflate(sourceScripts) 는
    // 한컴이 받아들이지만 var 선언 부재로 Form 컨트롤 JS 핸들러 미동작.
    let header_scripts = reader.read_file_bytes("Scripts/headerScripts").ok();
    let source_scripts = reader.read_file_bytes("Scripts/sourceScripts").ok();
    let scripts_default_jscript = match (header_scripts, source_scripts) {
        (Some(hs), Some(ss)) if !hs.is_empty() || !ss.is_empty() => {
            let mut combined = Vec::with_capacity(4 + hs.len() + 4 + ss.len() + 12);
            combined.extend_from_slice(&((hs.len() / 2) as u32).to_le_bytes());
            combined.extend_from_slice(&hs);
            combined.extend_from_slice(&((ss.len() / 2) as u32).to_le_bytes());
            combined.extend_from_slice(&ss);
            combined.extend_from_slice(&[0u8; 8]);
            combined.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]);
            raw_deflate(&combined).unwrap_or_else(|| FALLBACK_SCRIPTS_DEFAULT_JSCRIPT.to_vec())
        }
        _ => FALLBACK_SCRIPTS_DEFAULT_JSCRIPT.to_vec(),
    };
    streams.push((
        "/Scripts/DefaultJScript".to_string(),
        scripts_default_jscript,
    ));

    // [Stage 2.2] HWPX 컨테이너에 동등 데이터가 없는 contract 3 스트림 fallback.
    // 한컴 정답지가 모든 HWP 파일에 요구하는 최소 OLE Property Set / 메타.
    //
    // 주의: 스트림 경로는 OLE 표준의 `\x05` 선두 prefix (Property Set 표시) 가
    // 정답지의 실제 이름이나, mini_cfb 가 path 형식으로 처리하므로 일반
    // ASCII path 로 작성 후 한컴이 정상 인식. form-01 정답지의 실제 경로
    // `\x05HwpSummaryInformation` 와 byte-level 차이는 Stage 2.3 검증.
    streams.push((
        "/HwpSummaryInformation".to_string(),
        FALLBACK_HWP_SUMMARY.to_vec(),
    ));
    streams.push((
        "/DocOptions/_LinkDoc".to_string(),
        FALLBACK_DOC_OPTIONS_LINK_DOC.to_vec(),
    ));
    streams.push((
        "/Scripts/JScriptVersion".to_string(),
        FALLBACK_SCRIPTS_JSCRIPT_VERSION.to_vec(),
    ));

    ContractStreams { streams }
}

/// 단순 zlib deflate 헬퍼. 실패 시 None.
#[allow(dead_code)]
fn zlib_deflate(input: &[u8]) -> Option<Vec<u8>> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input).ok()?;
    encoder.finish().ok()
}

/// Raw deflate 헬퍼 (zlib 헤더 없음). HWP5 의 Scripts/DefaultJScript 압축에 사용.
fn raw_deflate(input: &[u8]) -> Option<Vec<u8>> {
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input).ok()?;
    encoder.finish().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zlib_deflate_roundtrip() {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let input = b"hello rhwp scripts test";
        let compressed = zlib_deflate(input).expect("zlib deflate failed");
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut decoded = Vec::new();
        decoder
            .read_to_end(&mut decoded)
            .expect("zlib inflate failed");
        assert_eq!(decoded, input);
    }
}
