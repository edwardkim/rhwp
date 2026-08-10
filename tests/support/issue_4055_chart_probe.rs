//! #4055 B1 스파이크 통합 테스트의 공용 파서·CFB·변종 생성 helper.

use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};

use rhwp::ooxml_chart::OoxmlChart;
use rhwp::parser::ole_container::parse_ole_container;

/// 레거시 그리드에서 f64 값 뒤에 항상 붙는 트레일러.
pub(super) const VALUE_TRAILER: &[u8] = &[0xFF, 0xFF, 0x06, 0x00, 0x00, 0x00];
const GRID_MARKER: &[u8] = b"VtDataGrid\0";
pub(super) const DOUBLE_MARKER: &[u8] = b"VtDouble\0";

/// `VtDataGrid` 다음에 오는 형제 오브젝트 마커들 — 그리드 구간의 끝을 정한다.
/// `src/ole_chart/parser.rs` 의 `find_next_legacy_object_marker` 와 같은 목록이다.
const OBJECT_MARKERS: &[&[u8]] = &[
    b"VtBackdrop\0",
    b"VtBackDrop\0",
    b"VtChartSection\0",
    b"VtFootnote\0",
    b"VtLegend\0",
    b"VtPlot\0",
    b"VtPrintInformation\0",
    b"VtChartTitle\0",
    b"VtTitle\0",
];

/// sentinel — 원본 최대값이 `5` 라 `91.7` 이면 첫 막대가 차트를 뚫고 솟는다.
pub(super) const SENTINEL: f64 = 91.7;
pub(super) const SENTINEL_TEXT: &str = "91.7";
pub(super) const EMF_STREAM: &str = "/\u{2}OlePres000";

/// 기준 샘플 — 한컴 육안 판정용 변종의 원본.
pub(super) const BASE_SAMPLE: &str = "samples/chart/세로막대형/묶은세로막대형";

pub(super) fn find_from(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from >= haystack.len() || needle.len() > haystack.len() - from {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| from + p)
}

/// `VtDataGrid` 데이터 구간 `[start, end)`.
///
/// 시작은 마커 + `StoredName` NUL + `StoredVersion`(4B) 뒤다
/// (`legacy_chart_object_data_start` 와 같은 규약).
fn grid_window(contents: &[u8]) -> Option<(usize, usize)> {
    let marker = find_from(contents, GRID_MARKER, 0)?;
    let start = marker + GRID_MARKER.len() + 4;
    if start > contents.len() {
        return None;
    }
    let end = OBJECT_MARKERS
        .iter()
        .filter_map(|m| find_from(contents, m, start))
        .min()
        .unwrap_or(contents.len());
    Some((start, end))
}

/// 그리드 값 셀의 `(바이트 오프셋, 값)` 목록.
///
/// **값의 크기·부호·정수 여부를 보지 않는다.** 오직 트레일러 위치로만 찾는다.
pub(super) fn locate_grid_values(contents: &[u8]) -> Vec<(usize, f64)> {
    let Some((start, end)) = grid_window(contents) else {
        return Vec::new();
    };
    // 첫 값은 `VtDouble` 선언 뒤에 온다. 그 앞의 트레일러 유사 바이트는 값이 아니다.
    let Some(anchor) = find_from(&contents[..end], DOUBLE_MARKER, start) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    let mut cursor = anchor;
    while let Some(hit) = find_from(&contents[..end], VALUE_TRAILER, cursor + 1) {
        cursor = hit;
        let Some(value_at) = hit.checked_sub(8) else {
            continue;
        };
        if value_at < anchor {
            continue;
        }
        let bytes: [u8; 8] = contents[value_at..hit].try_into().expect("8바이트");
        out.push((value_at, f64::from_le_bytes(bytes)));
    }
    out
}

/// 문서에서 (레거시 `Contents`, OOXML 차트 XML) 을 꺼낸다.
///
/// HWPX 는 `Chart/chartN.xml` 이 `bin_data_content`(id=60000+N, `ooxml_chart`)로,
/// HWP5 는 중첩 CFB 의 `OOXMLChartContents` 로 들어온다. 레거시 `Contents` 는
/// 양쪽 모두 중첩 CFB 안에 있다.
pub(super) fn chart_streams(doc_bytes: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let doc = rhwp::parse_document(doc_bytes).ok()?;

    let mut legacy = None;
    let mut ooxml = None;

    for content in &doc.bin_data_content {
        if content.extension == "ooxml_chart" {
            ooxml.get_or_insert_with(|| content.data.load());
            continue;
        }
        let bytes = content.data.load();
        if !bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
            continue;
        }
        if let Some(container) = parse_ole_container(&bytes) {
            if let Some(raw) = container.raw_contents {
                legacy.get_or_insert(raw);
            }
            if let Some(xml) = container.ooxml_chart {
                ooxml.get_or_insert(xml);
            }
        }
    }

    Some((legacy?, ooxml?))
}

/// 그리드 값이 놓이는 두 가지 순서.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Orientation {
    /// 한 행 = 한 계열 (계열의 값들이 연속)
    SeriesMajor,
    /// 한 행 = 한 카테고리 (같은 카테고리의 계열별 값이 연속)
    CategoryMajor,
    /// 값이 하나뿐이라 두 순서가 구분되지 않음
    Degenerate,
}

pub(super) fn classify(found: &[f64], series: &[Vec<f64>]) -> Option<Orientation> {
    let series_major: Vec<f64> = series.iter().flatten().copied().collect();
    if series_major.len() == 1 && found == series_major.as_slice() {
        return Some(Orientation::Degenerate);
    }
    let cols = series.first()?.len();
    let category_major: Vec<f64> = (0..cols)
        .flat_map(|c| series.iter().map(move |s| s[c]))
        .collect();

    if found == series_major.as_slice() {
        Some(Orientation::SeriesMajor)
    } else if found == category_major.as_slice() {
        Some(Orientation::CategoryMajor)
    } else {
        None
    }
}

/// 정답지 — OOXML `c:val`(분산형은 `c:yVal`)의 계열별 값.
pub(super) fn ground_truth(ooxml: &[u8]) -> Option<Vec<Vec<f64>>> {
    let chart = OoxmlChart::parse(ooxml)?;
    let series: Vec<Vec<f64>> = chart
        .series
        .iter()
        .map(|s| s.values.clone())
        .filter(|v| !v.is_empty())
        .collect();
    if series.is_empty() {
        return None;
    }
    // 계열별 길이가 다르면 직사각 그리드가 아니므로 이 프로브의 대상이 아니다.
    let cols = series[0].len();
    if series.iter().any(|s| s.len() != cols) {
        return None;
    }
    Some(series)
}

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

pub(super) fn corpus() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/chart");
    let mut out = Vec::new();
    chart_samples(&root, &mut out);
    out.sort();
    assert!(!out.is_empty(), "samples/chart 코퍼스가 비어 있다");
    out
}

pub(super) fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// 중첩 CFB 의 **모든** 스트림을 열거한다.
///
/// `parse_ole_container` 는 아는 4종만 뽑으므로 재포장에 쓸 수 없다 — 나머지가
/// 소실된다. S4 는 "그 4종 밖에 무엇이 있는가"를 세는 것이기도 하다.
pub(super) fn all_streams(cfb_bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut comp =
        cfb::CompoundFile::open(std::io::Cursor::new(cfb_bytes.to_vec())).expect("CFB 열기");
    let paths: Vec<std::path::PathBuf> = comp
        .walk()
        .filter(|e| e.is_stream())
        .map(|e| e.path().to_path_buf())
        .collect();
    let mut out = Vec::new();
    for path in paths {
        let mut buf = Vec::new();
        comp.open_stream(&path)
            .expect("스트림 열기")
            .read_to_end(&mut buf)
            .expect("스트림 읽기");
        // Windows 에서 `cfb` 는 `/BinData\BIN0001.OLE` 처럼 구분자를 섞어 돌려준다.
        // `mini_cfb` 는 이제 스스로 정규화하지만(#4097), 이 함수의 반환값을 **이름으로
        // 비교**하는 쪽(`mutate_nested` 의 `== "/OOXMLChartContents"`, bundle 생성기의
        // `starts_with("/BinData/")`)이 있어 여기서도 플랫폼 무관 표기로 맞춘다.
        out.push((path.to_string_lossy().replace('\\', "/"), buf));
    }
    out
}

/// 스트림 목록으로 CFB 를 다시 만든다. `mini_cfb` 를 쓰는 이유는
/// `cfb::CompoundFile::create()` 가 `SystemTime::now()` 로 wasm32 에서 panic 하기
/// 때문이다 (`src/parser/hwp3/ole.rs` 선례와 같다).
pub(super) fn rebuild_cfb(streams: &[(String, Vec<u8>)]) -> Vec<u8> {
    let refs: Vec<(&str, &[u8])> = streams
        .iter()
        .map(|(n, d)| (n.as_str(), d.as_slice()))
        .collect();
    rhwp::serializer::mini_cfb::build_cfb(&refs).expect("CFB 재포장")
}

/// 루트 스토리지의 CLSID — **판정 오라클**.
///
/// 일부러 `cfb` 크레이트로 읽는다. rhwp 의 `ole_root_clsid` 로 판정하면
/// `assert_eq!(read(write(x)), read(x))` 가 되어, 읽기와 쓰기가 같은 오프셋 오해를
/// 공유해도 통과해 버린다. 쓰기는 프로덕션, 채점은 독립 구현이어야 한다. (#4097)
///
/// `cfb` 는 CLSID 를 (LE u32, LE u16, LE u16, 8B) 로 읽어 `Uuid::from_fields` 하므로
/// (`cfb/internal/direntry.rs:88`), `to_bytes_le()` 가 파일 원시 16바이트를 복원한다.
pub(super) fn root_clsid(cfb: &[u8]) -> [u8; 16] {
    cfb::CompoundFile::open(std::io::Cursor::new(cfb.to_vec()))
        .expect("CFB 열기")
        .root_entry()
        .clsid()
        .to_bytes_le()
}

/// 원본과 같은 루트 CLSID 를 유지하며 재포장한다. 중첩 OLE CFB 는 반드시 이 쪽을 쓴다.
///
/// #4055 스파이크에서는 `build_cfb` 출력에 CLSID 를 사후 스탬프하는 테스트 로컬 바이트
/// 수술이었다. #4097 이 `build_cfb_with_root_clsid` 를 넣으면서 **프로덕션 경로**가 됐다 —
/// 그 덕에 `mini_cfb` 를 되돌리면 이 helper 를 쓰는 단언들이 red 가 된다.
pub(super) fn rebuild_cfb_preserving_clsid(
    original: &[u8],
    streams: &[(String, Vec<u8>)],
) -> Vec<u8> {
    let clsid = rhwp::parser::ole_container::ole_root_clsid(original)
        .expect("원본 중첩 CFB 의 루트 CLSID 를 읽을 수 있어야 한다");
    let refs: Vec<(&str, &[u8])> = streams
        .iter()
        .map(|(n, d)| (n.as_str(), d.as_slice()))
        .collect();
    rhwp::serializer::mini_cfb::build_cfb_with_root_clsid(&refs, clsid).expect("CFB 재포장")
}

/// OOXML 의 **첫 `c:val` 안 첫 `c:v`** 텍스트만 바꾼다.
///
/// 전체 재직렬화를 하지 않는 이유: `src/ooxml_chart/parser.rs` 는 `c:pt idx`·
/// `c:f`·`c:externalData`·`extLst` 를 읽지 않는다. 파싱→재방출로 왕복시키면
/// 모델에 없는 것이 전부 사라진다. 바이트 수술이 최소 diff 다.
pub(super) fn patch_ooxml_first_value(xml: &[u8], sentinel: &str) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(xml).ok()?;
    let val_at = text.find("<c:val>")?;
    let v_open = text[val_at..].find("<c:v>")? + val_at + "<c:v>".len();
    let v_close = text[v_open..].find("</c:v>")? + v_open;
    let mut out = String::with_capacity(text.len() + 4);
    out.push_str(&text[..v_open]);
    out.push_str(sentinel);
    out.push_str(&text[v_close..]);
    Some(out.into_bytes())
}

/// 레거시 `Contents` 의 **첫 그리드 값**을 8바이트 제자리 덮어쓰기 한다.
pub(super) fn patch_legacy_first_value(contents: &[u8], sentinel: f64) -> Option<Vec<u8>> {
    let (offset, _) = *locate_grid_values(contents).first()?;
    let mut out = contents.to_vec();
    out[offset..offset + 8].copy_from_slice(&sentinel.to_le_bytes());
    Some(out)
}

/// HWPX zip 을 엔트리 단위로 다시 쓴다. 손대지 않는 엔트리는 `raw_copy_file` 로
/// 압축 방식까지 보존한다 (`mimetype` 은 stored 여야 한다).
pub(super) fn rewrite_hwpx(original: &[u8], replacements: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut src =
        zip::ZipArchive::new(std::io::Cursor::new(original.to_vec())).expect("원본 HWPX 열기");
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let entry = src.by_index(i).expect("zip 엔트리");
        let name = entry.name().to_string();
        match replacements.iter().find(|(n, _)| *n == name) {
            Some((_, bytes)) => {
                out.start_file(&name, zip::write::SimpleFileOptions::default())
                    .expect("start_file");
                out.write_all(bytes).expect("엔트리 쓰기");
            }
            None => {
                out.raw_copy_file(entry).expect("raw copy");
            }
        }
    }
    out.finish().expect("zip finish").into_inner()
}

/// HWPX zip 에 **없던** 엔트리를 뒤에 덧붙인다. 기존 엔트리는 `raw_copy_file` 로
/// 그대로 옮긴다.
///
/// `rewrite_hwpx` 는 이름이 이미 있는 엔트리만 갈아끼우므로 새 `BinData/*` 를 넣을 수
/// 없다. 차트 문서에 그림을 얹어 `bin_count > 1` 을 만들려면(#4099 수용 기준 5) 둘 다
/// 필요하다 — 코퍼스 28종이 전부 BinData 1개라 합성 외에는 그 경로를 밟을 방법이 없다.
pub(super) fn append_hwpx_entries(original: &[u8], additions: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut src =
        zip::ZipArchive::new(std::io::Cursor::new(original.to_vec())).expect("원본 HWPX 열기");
    let existing: Vec<String> = (0..src.len())
        .map(|i| src.by_index(i).expect("zip 엔트리").name().to_string())
        .collect();
    for (name, _) in additions {
        assert!(
            !existing.contains(name),
            "{name} 은 이미 있다 — 교체는 rewrite_hwpx 를 쓴다"
        );
    }

    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let entry = src.by_index(i).expect("zip 엔트리");
        out.raw_copy_file(entry).expect("raw copy");
    }
    for (name, bytes) in additions {
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .expect("start_file");
        out.write_all(bytes).expect("엔트리 쓰기");
    }
    out.finish().expect("zip finish").into_inner()
}

/// HWP5 바깥 CFB 를 다시 쓴다. 지정한 스트림만 갈고 나머지는 바이트 그대로 옮긴다.
pub(super) fn rewrite_hwp(original: &[u8], replacements: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut streams = all_streams(original);
    for (name, bytes) in replacements {
        let slot = streams
            .iter_mut()
            .find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("교체 대상 스트림 {name} 없음"));
        slot.1 = bytes.clone();
    }
    rebuild_cfb(&streams)
}

/// HWP5 `BinData` OLE Storage 스트림에서 중첩 CFB 를 꺼낸다.
pub(super) fn hwp_nested_cfb(stream: &[u8]) -> Vec<u8> {
    let inflated = rhwp::parser::cfb_reader::decompress_stream(stream).expect("BinData 압축 해제");
    inflated[4..].to_vec()
}

/// 중첩 CFB 를 HWP5 `BinData` OLE Storage 스트림으로 되싼다 — 4바이트 LE size
/// prefix 를 붙이고 raw deflate 로 압축한다
/// (`src/serializer/cfb_writer.rs:232-255` 와 같은 규약).
pub(super) fn hwp_ole_stream(nested_cfb: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(nested_cfb.len() + 4);
    payload.extend_from_slice(&(nested_cfb.len() as u32).to_le_bytes());
    payload.extend_from_slice(nested_cfb);

    let mut encoder =
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&payload).expect("deflate 쓰기");
    encoder.finish().expect("deflate 완료")
}

/// 한 파일 안 각 표현의 **첫 값**과 EMF 유무. 변종이 라벨대로 조립됐는지 검증한다.
#[derive(Debug, PartialEq)]
pub(super) struct Representations {
    /// ① HWPX `Chart/chartN.xml` — HWP5 에는 없다.
    pub(super) zip_part: Option<f64>,
    /// ② 중첩 CFB `OOXMLChartContents`
    pub(super) nested_ooxml: Option<f64>,
    /// ③ 레거시 `Contents`
    pub(super) legacy: Option<f64>,
    /// ④ `\x02OlePres000` EMF 프리뷰
    pub(super) has_emf: bool,
}

pub(super) fn representations(doc_bytes: &[u8]) -> Representations {
    let doc = rhwp::parse_document(doc_bytes).expect("문서 파싱");
    let mut out = Representations {
        zip_part: None,
        nested_ooxml: None,
        legacy: None,
        has_emf: false,
    };
    for content in &doc.bin_data_content {
        let bytes = content.data.load();
        if content.extension == "ooxml_chart" {
            out.zip_part = ground_truth(&bytes).map(|g| g[0][0]);
            continue;
        }
        if !bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
            continue;
        }
        if let Some(container) = parse_ole_container(&bytes) {
            if let Some(xml) = &container.ooxml_chart {
                out.nested_ooxml = ground_truth(xml).map(|g| g[0][0]);
            }
            if let Some(raw) = &container.raw_contents {
                out.legacy = locate_grid_values(raw).first().map(|(_, v)| *v);
            }
            out.has_emf = container.preview_emf.is_some() || container.preview_wmf.is_some();
        }
    }
    out
}

/// 중첩 CFB 안에서 OOXML 사본·레거시를 패치하고 EMF 를 뺄지 정한다.
pub(super) fn mutate_nested(
    nested: &[u8],
    patch_ooxml: bool,
    patch_legacy: bool,
    drop_emf: bool,
) -> Vec<u8> {
    let mut streams = all_streams(nested);
    streams.retain(|(name, _)| !(drop_emf && name == EMF_STREAM));
    for (name, data) in streams.iter_mut() {
        if patch_ooxml && name == "/OOXMLChartContents" {
            *data = patch_ooxml_first_value(data, SENTINEL_TEXT).expect("중첩 OOXML 패치");
        }
        if patch_legacy && name == "/Contents" {
            *data = patch_legacy_first_value(data, SENTINEL).expect("레거시 패치");
        }
    }
    rebuild_cfb_preserving_clsid(nested, &streams)
}
