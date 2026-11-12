//! [#9029] 고정형 계약 — DocInfo 끝 잔여 바이트(예약 태그 0 머리)가 있는 HWP5 문서를 연다.
//!
//! 주석 갱신: 2026-11-13 KST(+09:00), 새 파일.
//!
//! - 입력: 공개 문서 `samples/issue9029/seal_registration_docinfo_tail.hwp`
//!   (중앙선관위 264666 인영서, rhwp-issue-docs#1089). DocInfo 의 마지막 유효 레코드
//!   (tag 30·31) 뒤에 0 채움과 3바이트 어긋난 사본이 남아 다음 머리가 `tag=0 size=480`
//!   으로 해독되고 가용 31바이트를 넘는다.
//! - 기준: `pdf/issue9029/seal_registration_docinfo_tail-print-2022.pdf`
//!   (Hwp 2022 12.0.0.4547 Print, Hancom PDF 1.3.0.547) 1쪽 — 쪽 전체가 스캔 그림 한 장.
//! - 수정 전 rhwp(devel `31a4225757`)는 `레코드 데이터 부족: tag=0, 필요=480, 가용=31` 로
//!   문서 전체를 거부했다.
//!
//! 검사: 문서 열기 성공, 잔여 바이트 앞의 DocInfo 레코드(글꼴·BinData)가 보존되고, 전체
//! 쪽수 = 한컴 Print 1쪽, 그 쪽에 스캔 그림이 그려진다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::path::PathBuf;

const SOURCE: &str = "samples/issue9029/seal_registration_docinfo_tail.hwp";
/// 한컴 2022 Print 의 전체 쪽수.
const PRINT_PAGES: u32 = 1;

fn images(node: &RenderNode) -> usize {
    usize::from(matches!(node.node_type, RenderNodeType::Image(_)))
        + node.children.iter().map(images).sum::<usize>()
}

#[test]
fn reserved_tag_tail_after_valid_docinfo_records_still_opens_the_document() {
    let bytes =
        std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SOURCE)).expect(SOURCE);
    let document = rhwp::parse_document(&bytes).expect("DocInfo 끝 잔여 바이트가 있어도 연다");

    let hangul_fonts: Vec<&str> = document.doc_info.font_faces[0]
        .iter()
        .map(|font| font.name.as_str())
        .collect();
    assert_eq!(
        hangul_fonts,
        ["함초롬돋움", "함초롬바탕"],
        "잔여 바이트 앞 DocInfo 글꼴 보존"
    );
    assert_eq!(
        document.doc_info.bin_data_list.len(),
        1,
        "잔여 바이트 앞 DocInfo BinData 보존"
    );

    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), PRINT_PAGES, "한컴 Print 의 전체 쪽수");
    let tree = core.build_page_render_tree(0).expect("1쪽");
    assert_eq!(images(&tree.root), 1, "쪽 전체 스캔 그림 한 장");
}
