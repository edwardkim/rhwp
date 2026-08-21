//! 이슈 #5799 — 인라인 탭(`<hp:tab … leader="3"/>`)의 채움 종류를 엉뚱한 슬롯에서 읽어
//! 목차 점선(탭 리더)이 한 개도 안 그려지던 결함의 회귀 가드.
//!
//! 인라인 탭 확장 레코드 `[u16; 7]` 의 인코딩은
//! `ext[0..2]` = 탭 폭(UINT32 저/고워드), `ext[2]` = `(탭 종류 << 8) | 채움 종류` 다
//! (`parser/hwpx/section.rs` `parse_tab_extension`, `parser/body_text.rs` 의 TAB 인라인
//! 컨트롤, 위치 계산부 `text_measurement.rs` `inline_tab_x` 의 `fill_low = ext[2] & 0xFF`).
//! `extract_tab_leaders_with_extended` 만 채움 종류를 `ext[1]` 에서 읽었는데 `ext[1]` 은
//! 탭 폭의 상위 16비트라 목차 탭 폭(수만 HWPUNIT)에서는 늘 0 이다. 그래서 문단 모양의
//! `TabDef` 가 채움을 따로 갖고 있지 않으면 `leader` 가 붙은 탭도 채움 없음으로 읽혀
//! 항목명과 쪽번호 사이가 빈칸으로 남았다.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// 탭 리더 점선(채움 종류 3)이 SVG 로 나갈 때의 시그니처.
/// `renderer/svg.rs` 탭 리더 `3 =>` 분기만 내는 형태다.
const DOT_LEADER: &str = "stroke-width=\"1.0\" stroke-dasharray=\"0.1 3\" stroke-linecap=\"round\"";

/// 목차 4줄 이상이 점선으로 이어져야 한다는 뜻의 하한.
const MIN_TOC_DOT_LEADERS: usize = 20;

/// `samples/SO-SUEOP.hwpx` 2쪽(0-base 1) 이 목차다.
const TOC_SAMPLE: &str = "samples/SO-SUEOP.hwpx";
const TOC_PAGE: u32 = 1;

fn sample_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn dot_leader_count(svg: &str) -> usize {
    svg.match_indices(DOT_LEADER).count()
}

fn render_toc_page(bytes: &[u8], label: &str) -> String {
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(bytes)
        .unwrap_or_else(|e| panic!("{label} 파싱 실패: {e:?}"));
    doc.render_page_svg_native(TOC_PAGE)
        .unwrap_or_else(|e| panic!("{label} {TOC_PAGE}쪽 렌더 실패: {e:?}"))
}

/// 목차 문서에서 **문단 모양의 `TabDef` 채움만** 지운 사본을 시험 시점에 합성한다.
///
/// `SO-SUEOP.hwpx` 의 목차 줄은 `<hp:tab width="35940" leader="3" type="0"/>` 로
/// 점선을 만들지만, 같은 문서의 `<hh:tabItem … leader="DASH">` 가 같은 점선을 한 번 더
/// 지정하고 있어 인라인 탭 쪽 결함이 가려진다. `hh:tabItem` 의 `leader` 만 `NONE` 으로
/// 바꾸면 **인라인 탭의 `leader` 가 유일한 정보원**이 되어 #5799 가 그대로 드러난다.
/// (본문 `<hp:tab>` 과 탭 위치는 건드리지 않으므로 조판 결과는 그대로다.)
fn toc_doc_with_tabdef_leaders_cleared() -> Vec<u8> {
    let bytes = std::fs::read(sample_path(TOC_SAMPLE)).expect("SO-SUEOP.hwpx 읽기");
    let mut zin = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("hwpx zip 열기");
    let mut out = Vec::new();
    {
        let mut zout = zip::ZipWriter::new(std::io::Cursor::new(&mut out));
        let mut patched = 0usize;
        for i in 0..zin.len() {
            let mut entry = zin.by_index(i).expect("zip 항목");
            let name = entry.name().to_string();
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).expect("zip 항목 읽기");
            if name == "Contents/header.xml" {
                let mut xml = String::from_utf8(buf).expect("header.xml UTF-8");
                for leader in [
                    "SOLID",
                    "DOT",
                    "DASH",
                    "DASH_DOT",
                    "DASH_DOT_DOT",
                    "LONG_DASH",
                    "CIRCLE",
                ] {
                    let from = format!("leader=\"{leader}\"");
                    patched += xml.matches(from.as_str()).count();
                    xml = xml.replace(from.as_str(), "leader=\"NONE\"");
                }
                buf = xml.into_bytes();
            }
            zout.start_file(
                name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )
            .expect("zip 항목 쓰기 시작");
            zout.write_all(&buf).expect("zip 항목 쓰기");
        }
        assert!(
            patched > 0,
            "{TOC_SAMPLE} header.xml 에서 채움 있는 hh:tabItem 을 찾지 못했다 — \
             픽스처 합성 전제가 깨졌다"
        );
        zout.finish().expect("zip 마감");
    }
    out
}

/// 인라인 탭의 `leader` 만으로 목차 점선이 그려져야 한다 (#5799 본체).
#[test]
fn issue_5799_inline_tab_leader_alone_draws_toc_dots() {
    let svg = render_toc_page(
        &toc_doc_with_tabdef_leaders_cleared(),
        "TabDef 채움을 지운 목차 문서",
    );
    let count = dot_leader_count(&svg);
    assert!(
        count >= MIN_TOC_DOT_LEADERS,
        "TabDef 채움이 없는 목차에서도 인라인 탭 `leader=\"3\"` 만으로 점선이 \
         {MIN_TOC_DOT_LEADERS}개 이상 나와야 한다 (실측 {count}개).\n  \
         0개 = #5799 회귀 — `extract_tab_leaders_with_extended` 가 인라인 탭 채움 종류를 \
         `ext[2]` 하위바이트가 아닌 곳에서 읽고 있는지 확인할 것."
    );
}

/// 원본(문단 모양의 `TabDef` 에도 채움이 있는 문서)은 종전과 똑같이 그려야 한다.
/// 인라인 채움은 `TabDef` 폴백보다 우선할 뿐 그것을 없애지 않는다.
#[test]
fn issue_5799_tabdef_leader_path_still_draws_toc_dots() {
    let bytes = std::fs::read(sample_path(TOC_SAMPLE)).expect("SO-SUEOP.hwpx 읽기");
    let count = dot_leader_count(&render_toc_page(&bytes, "SO-SUEOP.hwpx 원본"));
    assert!(
        count >= MIN_TOC_DOT_LEADERS,
        "원본 목차의 점선이 {MIN_TOC_DOT_LEADERS}개 이상이어야 한다 (실측 {count}개) — \
         TabDef 폴백 경로 회귀."
    );
}
