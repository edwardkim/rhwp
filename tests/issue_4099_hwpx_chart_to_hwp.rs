//! [Issue #4099] HWPX→HWP5 변환이 차트 참조를 끊는 결함의 회귀 게이트.
//!
//! ## 결함
//!
//! HWPX 차트는 `<hp:switch>` 의 `<hp:case>` 브랜치로 파싱돼 **가상 id**
//! `bin_data_id = 60000+N` 을 갖는다(`parser/hwpx/section.rs`, Task #195 규약). 그 id 는
//! HWPX zip 파트 `Chart/chartN.xml` 을 가리키며 **HWP5 에는 대응물이 없다.**
//! `<hp:default>` 에 있는 진짜 OLE(`BinData/ole1.ole`, 중첩 CFB)는
//! `chart_switch_fallback` 에 매달려만 있고 HWP5 저장 경로가 그것을 보지 않는다.
//!
//! 결과는 세 갈래다.
//!
//! - `serialize_ole_data`(`serializer/control.rs`)가 `60001` 을 그대로 기록 → **dangling**.
//!   HWP5 DocInfo 의 BinData 는 `storage_id = 1` 하나뿐이다.
//! - `find_bin_data_info_with_compress`(`serializer/cfb_writer.rs`) 폴백이
//!   `(60001, "ooxml_chart")` 를 돌려줘 **DocInfo 미등록 정크 스트림**
//!   `/BinData/BINEA61.ooxml_chart` 가 생긴다(0xEA61 = 60001).
//! - `bin_data_content` 의 차트 항목이 재파싱에서 사라져 `--verify` 가
//!   `bin_data_content count: expected=2 actual=1` 로 exit 3 을 낸다.
//!
//! **바이트는 멀쩡히 보존된다** — 끊어진 것은 참조뿐이다. 그래서 #4055 스파이크의
//! 바이트 단언(`observation_hwpx_to_hwp_conversion_keeps_the_chart`)은 통과했고,
//! rhwp 자신의 렌더가 회색 상자를 그리는 것을 아무도 보지 않았다.
//!
//! ## 정답지
//!
//! 한컴이 만든 `samples/chart/**/*.hwp` 의 CFB 는 `BinData/BIN0001.OLE` **하나뿐**이고
//! `.ooxml_chart` 스트림이 없다. GenShape 의 `instance_id` 는 **0** 인데, 이는
//! `<hp:default><hp:ole>`(instid="0") 의 값이지 `<hp:chart>`(@id="1117817146") 의 값이
//! 아니다 — **한컴 자신의 HWPX→HWP5 변환도 fallback 브랜치를 채택한다.** 그러므로
//! 수정 방향은 "차트 OleShape 를 fallback 으로 접는다" 이고, T4 가 그 근거를 고정한다.

#[path = "support/issue_4055_chart_probe.rs"]
mod chart_probe_support;

use chart_probe_support::{all_streams, corpus, manifest, rewrite_hwpx};

use std::io::{Cursor, Read};

use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::BinDataType;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::shape::{OleShape, ShapeObject};
use rhwp::parser::cfb_reader::decompress_stream;
use rhwp::serializer::hwpx::roundtrip::{diff_documents, strip_hwpx_to_hwp_noise, IrDiff};

/// 코퍼스 하한. `samples/chart` 는 28종이고 `issue_3546_chart_preserved_on_save.rs` 도
/// 같은 방식으로 하한을 건다 — 수집이 조용히 비면 전 단언이 공회전한다.
const CORPUS_LEN: usize = 28;

const BASE_SAMPLE: &str = "samples/chart/세로막대형/묶은세로막대형";

// ---------------------------------------------------------------------------
// 공용 헬퍼
// ---------------------------------------------------------------------------

/// HWPX 바이트를 HWP5 로 변환한다 — CLI `rhwp convert` 와 같은 경로
/// (`main.rs` → `DocumentCore::export_hwp_with_adapter`).
fn convert_to_hwp(hwpx: &[u8]) -> Vec<u8> {
    let mut core = DocumentCore::from_bytes(hwpx).expect("HWPX 로드");
    core.export_hwp_with_adapter().expect("HWP 변환")
}

/// `convert --verify` 와 같은 판정 — 변환 후 재파싱해 IR 을 대조한다.
///
/// `main.rs` 가 어댑터로 in-place 변형된 live IR 을 expected 로 쓰므로 여기서도
/// 변환 후의 `core.document()` 를 기준으로 삼는다.
fn verify_diff(hwpx: &[u8]) -> IrDiff {
    let mut core = DocumentCore::from_bytes(hwpx).expect("HWPX 로드");
    let out = core.export_hwp_with_adapter().expect("HWP 변환");
    let reloaded = DocumentCore::from_bytes(&out).expect("변환본 재파싱");
    let diff = diff_documents(core.document(), reloaded.document());
    strip_hwpx_to_hwp_noise(diff)
}

/// 문서 트리에서 첫 OLE 도형을 찾는다. 코퍼스는 문서당 차트 1개다.
fn first_ole(doc: &Document) -> Option<&OleShape> {
    for section in &doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Shape(shape) = ctrl {
                    if let ShapeObject::Ole(ole) = shape.as_ref() {
                        return Some(ole);
                    }
                }
            }
        }
    }
    None
}

fn read_zip_entry(hwpx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(hwpx.to_vec())).expect("HWPX zip 열기");
    let mut entry = zip
        .by_name(name)
        .unwrap_or_else(|e| panic!("zip 엔트리 {name}: {e}"));
    let mut s = String::new();
    entry.read_to_string(&mut s).expect("엔트리 읽기");
    s
}

fn base_hwpx() -> Vec<u8> {
    std::fs::read(manifest(&format!("{BASE_SAMPLE}.hwpx"))).expect("base HWPX 읽기")
}

// ---------------------------------------------------------------------------
// T1 — 수용 기준 1: 변환본 렌더에 placeholder 가 없다
// ---------------------------------------------------------------------------

/// 이슈 재현 명령(`convert` → `export-svg` → `grep "OLE 개체"`)의 in-process 등가물.
///
/// 서브프로세스 대신 `render_page_svg_native` 를 쓴다 — 같은 렌더 경로이고
/// 실패 시 어느 샘플인지 즉시 나온다.
#[test]
fn issue4099_converted_hwp_renders_the_chart() {
    let paths = corpus();
    assert!(
        paths.len() >= CORPUS_LEN,
        "samples/chart 코퍼스가 예상보다 작다: {}",
        paths.len()
    );

    let mut checked = 0usize;
    for path in &paths {
        let label = path.file_name().unwrap().to_string_lossy().to_string();
        let hwpx = std::fs::read(path).unwrap_or_else(|e| panic!("{label}: 읽기 {e}"));
        let hwp = convert_to_hwp(&hwpx);

        let core = DocumentCore::from_bytes(&hwp).unwrap_or_else(|e| panic!("{label}: 재파싱 {e}"));
        let svg = core
            .render_page_svg_native(0)
            .unwrap_or_else(|e| panic!("{label}: 렌더 {e}"));

        assert!(
            !svg.contains("OLE 개체 (BinData #"),
            "{label}: 변환본이 OLE placeholder 를 그린다 — 차트 참조가 끊겼다"
        );
        // `hwp-ooxml-chart-fallback` 은 차트 파싱 실패 시의 회색 상자다.
        // 뒤에 `"` 를 붙여 진짜 차트 <g> 만 센다.
        assert!(
            svg.contains("hwp-ooxml-chart\""),
            "{label}: 변환본에 OOXML 차트가 렌더되지 않았다"
        );
        assert!(
            !svg.contains("hwp-ooxml-chart-fallback"),
            "{label}: 차트가 fallback placeholder 로 그려졌다"
        );
        checked += 1;
    }
    assert_eq!(checked, paths.len(), "코퍼스를 전건 검사해야 한다");
}

// ---------------------------------------------------------------------------
// T2 — 수용 기준 2: convert --verify 가 통과한다
// ---------------------------------------------------------------------------

/// 이 축은 결함 발생 시점부터 계속 red 였다. 코퍼스가 래칫에 없었을 뿐이다
/// (`convert_verify_corpus_ratchet.rs` 의 `read_dir(samples/)` 는 비재귀).
#[test]
fn issue4099_convert_verify_passes_for_chart_corpus() {
    let paths = corpus();
    assert!(paths.len() >= CORPUS_LEN, "코퍼스 하한");

    let mut checked = 0usize;
    for path in &paths {
        let label = path.file_name().unwrap().to_string_lossy().to_string();
        let hwpx = std::fs::read(path).unwrap_or_else(|e| panic!("{label}: 읽기 {e}"));
        let diff = verify_diff(&hwpx);
        assert!(
            diff.is_empty(),
            "{label}: convert --verify 차이 {}건\n{}",
            diff.differences.len(),
            diff.differences
                .iter()
                .map(|d| format!("  [차이] {d}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        checked += 1;
    }
    assert_eq!(checked, paths.len());
}

// ---------------------------------------------------------------------------
// T3 — 수용 기준 3·4: 정크 스트림이 없고 OLE 참조가 실재한다
// ---------------------------------------------------------------------------

#[test]
fn issue4099_converted_cfb_has_no_junk_and_ole_ref_resolves() {
    let paths = corpus();
    assert!(paths.len() >= CORPUS_LEN, "코퍼스 하한");

    let mut checked = 0usize;
    for path in &paths {
        let label = path.file_name().unwrap().to_string_lossy().to_string();
        let hwpx = std::fs::read(path).unwrap_or_else(|e| panic!("{label}: 읽기 {e}"));
        let hwp = convert_to_hwp(&hwpx);

        // ① 정크 스트림 0건 (수용 기준 3)
        let streams = all_streams(&hwp);
        if let Some((junk, _)) = streams
            .iter()
            .find(|(n, _)| n.to_ascii_lowercase().ends_with(".ooxml_chart"))
        {
            panic!(
                "{label}: DocInfo 미등록 정크 스트림이 생겼다: {junk} \
                 (cfb_writer 폴백이 60000+N 을 스토리지 id 로 오해한다)"
            );
        }
        assert!(
            streams.iter().any(|(n, _)| n == "/BinData/BIN0001.OLE"),
            "{label}: 한컴 정답지와 같은 /BinData/BIN0001.OLE 이 없다 — 스트림 {:?}",
            streams.iter().map(|(n, _)| n).collect::<Vec<_>>()
        );

        // ② OLE 참조가 DocInfo 에 실재 (수용 기준 4)
        let doc = rhwp::parse_document(&hwp).unwrap_or_else(|e| panic!("{label}: 재파싱 {e:?}"));
        let ole = first_ole(&doc).unwrap_or_else(|| panic!("{label}: OLE 도형 없음"));
        assert_eq!(
            ole.bin_data_id, 1,
            "{label}: OLE 가 여전히 가상 id 를 가리킨다 (한컴 정답지는 1)"
        );
        assert!(
            doc.doc_info.bin_data_list.iter().any(|b| {
                u32::from(b.storage_id) == ole.bin_data_id && b.data_type == BinDataType::Storage
            }),
            "{label}: bin_data_id={} 가 DocInfo 에 Storage 로 등록돼 있지 않다 — 목록 {:?}",
            ole.bin_data_id,
            doc.doc_info
                .bin_data_list
                .iter()
                .map(|b| (b.storage_id, b.data_type))
                .collect::<Vec<_>>()
        );

        // ③ OLE 스트림 바이트 규약 — #3547 축 동승 보호
        let raw = read_cfb_stream(&hwp, "/BinData/BIN0001.OLE");
        let payload = decompress_stream(&raw).unwrap_or(raw);
        assert!(payload.len() > 12, "{label}: OLE 페이로드가 너무 짧다");
        let prefix = u32::from_le_bytes(payload[..4].try_into().unwrap()) as usize;
        assert_eq!(
            prefix,
            payload.len() - 4,
            "{label}: 4바이트 size prefix 가 CFB 길이를 가리켜야 한다 (#3547)"
        );
        assert_eq!(
            &payload[4..12],
            &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
            "{label}: prefix 뒤가 CFB 매직이어야 한다"
        );
        checked += 1;
    }
    assert_eq!(checked, paths.len());
}

fn read_cfb_stream(bytes: &[u8], path: &str) -> Vec<u8> {
    let mut cfb = cfb::CompoundFile::open(Cursor::new(bytes)).expect("CFB 열기");
    let mut stream = cfb
        .open_stream(path)
        .unwrap_or_else(|e| panic!("스트림 {path}: {e}"));
    let mut data = Vec::new();
    stream.read_to_end(&mut data).expect("스트림 읽기");
    data
}

// ---------------------------------------------------------------------------
// T4 — fold 방향의 근거 고정 (한컴 정답지 대조)
// ---------------------------------------------------------------------------

/// **이 테스트가 없으면 다음 사람이 "instance_id 를 chart 쪽에서 살려야 하지 않나"로
/// 되돌린다.** 실측값(2026-08-10):
///
/// ```text
/// 오라클 .hwp        bin_data_id=1      instance_id=0           attr=0x140A2210
/// HWPX chart 브랜치  bin_data_id=60001  instance_id=1117817146  attr=0x140A2210
/// HWPX fallback      bin_data_id=1      instance_id=0           attr=0x140A2210
/// ```
///
/// `instance_id` 가 유일한 판별자다 — 한컴은 `<hp:chart @id>` 가 아니라
/// `<hp:default><hp:ole @instid>` 를 쓴다. 따라서 fold 는 fallback 을 통째로 채택해야
/// 하고, chart 브랜치의 `instance_id` 를 승계하면 오라클과 어긋난다.
#[test]
fn issue4099_folded_ole_matches_hancom_oracle() {
    let hwpx = base_hwpx();
    let hwp = convert_to_hwp(&hwpx);
    let doc = rhwp::parse_document(&hwp).expect("변환본 재파싱");
    let ole = first_ole(&doc).expect("OLE 도형");

    let oracle_bytes = std::fs::read(manifest(&format!("{BASE_SAMPLE}.hwp"))).expect("오라클 읽기");
    let oracle_doc = rhwp::parse_document(&oracle_bytes).expect("오라클 파싱");
    let oracle = first_ole(&oracle_doc).expect("오라클 OLE");

    assert_eq!(
        ole.common.instance_id, oracle.common.instance_id,
        "instance_id 가 한컴 정답지와 달라졌다 — fallback 브랜치를 채택하지 않았다는 뜻이다 \
         (chart 브랜치 값은 1117817146, fallback·오라클은 0)"
    );
    assert_eq!(ole.common.instance_id, 0, "오라클 실측값");
    assert_eq!(ole.common.attr, oracle.common.attr);
    assert_eq!(ole.common.attr, 0x140A_2210, "오라클 실측값");
    assert_eq!(
        (ole.extent_x, ole.extent_y),
        (oracle.extent_x, oracle.extent_y)
    );
    assert_eq!((ole.extent_x, ole.extent_y), (7200, 7200));
    assert_eq!(
        (
            ole.drawing.shape_attr.original_width,
            ole.drawing.shape_attr.original_height
        ),
        (7200, 7200)
    );
    assert!(
        ole.chart_id_ref.is_none() && ole.chart_switch_fallback.is_none(),
        "HWP5 재파싱본에 HWPX 전용 표식이 남아 있을 수 없다"
    );
}

// ---------------------------------------------------------------------------
// T4b — fold 불가 경로와 캡션 이월 (코퍼스 0건이라 합성)
// ---------------------------------------------------------------------------

/// `<hp:switch>` 를 벗겨 `<hp:chart>` 단독으로 만든다 — `section.rs` 의 `b"chart"` arm
/// 경로("아직 보지 못한 변형. 안전 경로")를 태운다. `<hp:default>` 없는 case-only
/// switch 도 `parse_switch_chart_or_ole` 의 `chart.or(ole)` 폴스루로 같은 상태
/// (`chart_id_ref.is_some() && chart_switch_fallback.is_none()`)에 수렴한다.
fn synth_chart_without_fallback(hwpx: &[u8]) -> Vec<u8> {
    let xml = read_zip_entry(hwpx, "Contents/section0.xml");
    let sw_start = xml.find("<hp:switch>").expect("샘플에 <hp:switch> 가 있어야 한다");
    let sw_end = xml.find("</hp:switch>").expect("</hp:switch>") + "</hp:switch>".len();
    let seg = &xml[sw_start..sw_end];
    let c_start = seg.find("<hp:chart").expect("<hp:chart");
    let c_end = seg.find("</hp:chart>").expect("</hp:chart>") + "</hp:chart>".len();
    let chart_only = seg[c_start..c_end].to_string();
    let patched = format!("{}{}{}", &xml[..sw_start], chart_only, &xml[sw_end..]);
    assert_ne!(patched, xml, "치환이 실제로 일어나야 한다");
    rewrite_hwpx(
        hwpx,
        &[("Contents/section0.xml".to_string(), patched.into_bytes())],
    )
}

/// `<hp:chart>` 에만 `<hp:caption>` 을 넣는다(fallback `<hp:ole>` 에는 없음).
/// #4319 로 파서가 양쪽 캡션을 읽게 됐으므로, fold 가 chart 쪽 캡션을 버리면
/// 조용히 소실된다. 코퍼스 28종은 캡션이 0건이라 합성해야만 이 축을 잴 수 있다.
fn synth_chart_with_caption_only_on_chart(hwpx: &[u8]) -> Vec<u8> {
    let xml = read_zip_entry(hwpx, "Contents/section0.xml");
    let caption = r#"<hp:caption side="BOTTOM" fullSz="0" width="4000" gap="850" lastWidth="4000"><hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0"><hp:p id="0" paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>차트 1. 분기 매출</hp:t></hp:run></hp:p></hp:subList></hp:caption>"#;
    let close = "</hp:chart>";
    let at = xml.find(close).expect("</hp:chart>");
    let patched = format!("{}{}{}", &xml[..at], caption, &xml[at..]);
    assert_ne!(patched, xml, "치환이 실제로 일어나야 한다");
    rewrite_hwpx(
        hwpx,
        &[("Contents/section0.xml".to_string(), patched.into_bytes())],
    )
}

/// fallback 이 없으면 접을 대상이 없다. 그래도 **정크 스트림과 dangling 참조는
/// 만들지 않는다** — 그 둘이 이 이슈의 실제 피해다.
///
/// placeholder 렌더는 **허용**한다. HWP5 에는 평문 차트 XML 을 담을 자리가 없고,
/// 이 경로에서 OLE CFB 를 합성하려면 참조할 원본 CLSID 가 없어
/// `{4C3DA137-DC90-47B9-9BED-59DAE352A280}` 를 하드코딩해야 하는데 한컴이 그런 CFB 를
/// 받아들이는지 미검증이다(#4055 는 기존 CFB 를 수정했을 뿐 새로 만들지 않았다).
/// 도구는 #4097 의 `mini_cfb::build_cfb_with_root_clsid` 로 이미 갖춰져 있으므로,
/// 실물 변종이 관측되면 그때 이 자리를 채우면 된다.
#[test]
fn issue4099_chart_without_fallback_produces_no_junk_and_no_dangling_ref() {
    let synth = synth_chart_without_fallback(&base_hwpx());

    // 합성 검증 — 이 전제가 깨지면 아래 단언이 다른 것을 재게 된다.
    let src = rhwp::parse_document(&synth).expect("합성본 파싱");
    let src_ole = first_ole(&src).expect("합성본 OLE");
    assert!(
        src_ole.chart_id_ref.is_some() && src_ole.chart_switch_fallback.is_none(),
        "합성이 fallback 없는 차트를 만들어야 한다"
    );

    let hwp = convert_to_hwp(&synth);

    let streams = all_streams(&hwp);
    assert!(
        !streams
            .iter()
            .any(|(n, _)| n.to_ascii_lowercase().ends_with(".ooxml_chart")),
        "fallback 이 없어도 정크 스트림을 만들면 안 된다 — 스트림 {:?}",
        streams.iter().map(|(n, _)| n).collect::<Vec<_>>()
    );

    let doc = rhwp::parse_document(&hwp).expect("변환본 재파싱");
    let ole = first_ole(&doc).expect("변환본 OLE");
    assert_eq!(
        ole.bin_data_id, 0,
        "접을 수 없으면 참조를 비운다 — 없는 storage 를 가리키면 한컴이 오해한다"
    );

    assert!(
        verify_diff(&synth).is_empty(),
        "fold 불가 경로도 --verify 는 통과해야 한다"
    );
}

/// fold 는 fallback 을 통째로 채택하므로, chart 쪽에만 있던 캡션은 명시적으로
/// 이월하지 않으면 사라진다 (#4319 로 파서가 양쪽을 읽게 된 뒤 생긴 축).
#[test]
fn issue4099_fold_carries_over_chart_only_caption() {
    let synth = synth_chart_with_caption_only_on_chart(&base_hwpx());

    // 합성 검증 — chart 에만 캡션, fallback 에는 없음
    let src = rhwp::parse_document(&synth).expect("합성본 파싱");
    let src_ole = first_ole(&src).expect("합성본 OLE");
    let src_caption = src_ole
        .caption
        .as_ref()
        .expect("합성본 chart 브랜치에 캡션이 있어야 한다 (#4319 파서)");
    assert_eq!(src_caption.paragraphs[0].text, "차트 1. 분기 매출");
    assert!(
        src_ole
            .chart_switch_fallback
            .as_deref()
            .expect("fallback")
            .caption
            .is_none(),
        "합성은 fallback 에 캡션을 넣지 않는다"
    );

    let hwp = convert_to_hwp(&synth);
    let doc = rhwp::parse_document(&hwp).expect("변환본 재파싱");
    let ole = first_ole(&doc).expect("변환본 OLE");

    let caption = ole
        .caption
        .as_ref()
        .expect("chart 브랜치에만 있던 캡션이 fold 로 사라졌다 — 이월 규칙 누락");
    assert_eq!(
        caption.paragraphs[0].text, "차트 1. 분기 매출",
        "캡션 내용이 보존돼야 한다"
    );
}

// ---------------------------------------------------------------------------
// T6 — 멱등성
// ---------------------------------------------------------------------------

/// 어댑터는 live IR 을 in-place 로 바꾼다. fold 와 `ooxml_chart` 제거가 2회차에
/// 다른 결과를 내면 저장 버튼을 두 번 누른 사용자가 다른 파일을 얻는다.
/// `hwpx_to_hwp_adapter.rs` 의 같은 축은 차트 없는 문서만 재고 있다.
#[test]
fn issue4099_adapter_is_idempotent_on_chart_document() {
    let hwpx = base_hwpx();
    let mut core = DocumentCore::from_bytes(&hwpx).expect("HWPX 로드");
    let first = core.export_hwp_with_adapter().expect("1회차");
    let second = core.export_hwp_with_adapter().expect("2회차");
    assert_eq!(
        first, second,
        "차트 문서에서 어댑터를 두 번 돌리면 같은 바이트가 나와야 한다"
    );
}
