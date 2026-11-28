//! 관계형: 그림의 원본 순서와 셀 안의 포함·비겹침을 검사한다.
//! 주석 갱신: 2026-11-28T19:54:22+09:00; 직전 파일 커밋: 2026-10-13T20:40:43+09:00 (cdaa4ede1dec75988d4c47b2e2778c262af1a8a3).
//! 2026-11-28 한컴 Print 대조(96dpi, PDFium, 2px raw), 전1쪽 최저와 1쪽 모두 Native 100.00% / fresh WASM 100.00%.
//! 근거: mydocs/pr/assets/issues7813_8100/native/7863.tsv, mydocs/pr/assets/issues7813_8100/wasm/7863.tsv 및 각 대표 비교 PNG.
//!
//! [Issue #6840] `extract-pages` 가 참조 없는 `BinData` 를 하나도 버리지 않아, 쪽을 한
//! 장만 남겨도 파일이 거의 줄지 않던 결함의 가드.
//!
//! ## 왜 중요한가
//!
//! 이 명령이 내건 용도는 **발췌·부분 제출**과 결함 이분법이다. 그림이 많은 문서는 본문을
//! 88% 걷어내도 파일이 20% 밖에 줄지 않았고, **버리려던 쪽의 그림 원본이 산출물에 그대로
//! 남았다** — 부분 제출에서는 유출 경로다.
//!
//! ```text
//!   104쪽 6.5MB 문서 → --from 77 --to 77
//!     수정 전   BinData 204개 그대로   파일 5,225,984
//!     수정 후   BinData  36개          파일 1,364,992
//! ```
//!
//! ## 무엇을 잠그나 — 뒤바뀜이 진짜 위험이다
//!
//! `ImageAttr.bin_data_id` 는 **1 기준 순번(위치)** 이라 항목을 지우면 뒤 참조가 한 칸씩
//! 밀려 **그림이 조용히 뒤바뀐다.** 개수만 보는 시험은 그걸 못 잡으므로, 남은 그림의
//! **원본 소유 경로와 이미지 바이트**를 정리 전후로 대조한다.
//! 축약 #5833 입력의 그림 순서와 소속을 #7863의 관계형 회귀로 복원했다.
//!
//! ⚠ 내장 글꼴은 순번이 아니라 **storage id** 로 참조한다
//! (`load_bounded_embedded_font_bytes` 가 `content.id == font_id` 로 찾는다). 그 축을
//! 빠뜨리면 발췌본에서 글꼴이 통째로 사라지므로 음성 통제군으로 함께 잠근다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rhwp::DocumentCore;

/// 43개 중 7개만 실제로 쓰는 문서 — 칸 안 다문단 float 그림.
const PICTURES: &str = "samples/issue5833/cell_multi_para_float_pics.hwp";
/// 그림은 없고 `BinData` 가 **내장 글꼴 하나**뿐인 문서.
const EMBEDDED_FONT: &str = "samples/render-p35-font-native-bitmap.hwpx";
/// 34개를 모두 쓰는 문서 — 버릴 것이 없으면 건드리지 않아야 한다.
const ALL_REFERENCED: &str = "samples/basic/Worldcup_FIFA2010_32.hwp";

fn load(rel: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("#6840 공개 fixture 읽기 {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("{rel} 파싱: {error:?}"))
}

/// 1쪽 SVG 의 `<image>` 를 `(x, y, 바이트 해시)` 로 모은다.
///
/// data URI 를 통째로 비교하지 않고 해시로 줄인다 — 실패 메시지에 수십 KB 가 실리면
/// 무엇이 달라졌는지 오히려 안 보인다.
fn page_images(core: &DocumentCore) -> Vec<(String, String, u64)> {
    let svg = core.render_page_svg_native(0).expect("1쪽 SVG");
    let mut out = Vec::new();
    for chunk in svg.split("<image ").skip(1) {
        let attr = |name: &str| -> String {
            chunk
                .split_once(&format!("{name}=\""))
                .and_then(|(_, rest)| rest.split_once('"'))
                .map(|(v, _)| v.to_string())
                .unwrap_or_default()
        };
        let href = attr("href");
        let mut hasher = DefaultHasher::new();
        href.hash(&mut hasher);
        out.push((attr("x"), attr("y"), hasher.finish()));
    }
    out.sort();
    out
}

/// 저장 모델의 소유 경로와 그 참조가 가리키는 바이트를 수집한다.
/// 1쪽 공유 입력의 BinData pruning 계약이며 잘못된 화면 위치를 고정하지 않는다.
fn owned_picture_payloads(core: &DocumentCore) -> Vec<(Vec<usize>, String, Vec<u8>)> {
    use rhwp::model::{control::Control, document::Document, paragraph::Paragraph};

    fn walk(
        doc: &Document,
        paragraphs: &[Paragraph],
        path: &[usize],
        out: &mut Vec<(Vec<usize>, String, Vec<u8>)>,
    ) {
        for (pi, para) in paragraphs.iter().enumerate() {
            for (ci, control) in para.controls.iter().enumerate() {
                let mut owner = path.to_vec();
                owner.extend([pi, ci]);
                match control {
                    Control::Picture(picture) => {
                        let id = picture.image_attr.bin_data_id;
                        assert!(id > 0, "그림의 BinData 순번이 있어야 한다");
                        let data = doc
                            .bin_data_content
                            .get(usize::from(id - 1))
                            .expect("그림 순번이 실제 payload를 가리켜야 한다");
                        out.push((
                            owner,
                            data.extension.clone(),
                            data.data.load_shared().to_vec(),
                        ));
                    }
                    Control::Table(table) => {
                        for (cell_index, cell) in table.cells.iter().enumerate() {
                            let mut cell_owner = owner.clone();
                            cell_owner.push(cell_index);
                            walk(doc, &cell.paragraphs, &cell_owner, out);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let mut out = Vec::new();
    let doc = core.document();
    for (section_index, section) in doc.sections.iter().enumerate() {
        walk(doc, &section.paragraphs, &[section_index], &mut out);
    }
    out
}

/// 양성 계약 — 참조 없는 항목은 버리고, 남은 그림은 **한 장도 바뀌지 않는다.**
#[test]
fn unreferenced_bin_data_is_dropped_without_swapping_pictures() {
    let mut core = load(PICTURES);
    let before = owned_picture_payloads(&core);
    assert_eq!(
        before.len(),
        7,
        "표본이 어긋났다 — 원본 소유 경로에 그림 7개가 있어야 한다"
    );

    let report = core.extract_page_range(1, 1).expect("1쪽 추출");

    assert_eq!(
        (report.bin_data_before, report.bin_data_after),
        (43, 7),
        "참조 없는 36개가 버려져야 한다 — 회귀 시 43개가 그대로 남아 파일이 줄지 않는다"
    );
    assert_eq!(
        owned_picture_payloads(&core),
        before,
        "정리 뒤 그림의 원본 소유 경로 또는 바이트가 달라졌다 — 순번 재번호매김이 어긋나면 \
         그림이 조용히 뒤바뀐다"
    );
}

/// 음성 통제군 ① — 내장 글꼴은 **순번 참조가 없어도** 남아야 한다.
///
/// 이 문서는 그림이 0장이고 `BinData` 가 글꼴 하나(`font-native-smoke.ttf`)뿐이다.
/// 순번만 보고 버리면 여기서 `1 → 0` 이 되어 발췌본의 글꼴이 사라진다.
#[test]
fn embedded_font_bin_data_survives_even_with_no_picture_reference() {
    let mut core = load(EMBEDDED_FONT);
    assert!(
        page_images(&core).is_empty(),
        "표본이 어긋났다 — 이 문서에는 그림이 없어야 글꼴 축만 남는다"
    );

    let report = core.extract_page_range(1, 1).expect("1쪽 추출");

    assert_eq!(
        (report.bin_data_before, report.bin_data_after),
        (1, 1),
        "내장 글꼴이 쓰는 항목을 버렸다 — storage id 축을 빠뜨리면 글꼴이 사라진다"
    );
}

/// 음성 통제군 ② — 모두 참조되는 문서는 건드리지 않는다.
#[test]
fn fully_referenced_bin_data_is_left_alone() {
    let mut core = load(ALL_REFERENCED);
    let before = page_images(&core);

    let report = core.extract_page_range(1, 1).expect("1쪽 추출");

    assert_eq!(
        (report.bin_data_before, report.bin_data_after),
        (34, 34),
        "버릴 것이 없는데 항목이 줄었다"
    );
    assert_eq!(page_images(&core), before, "그림이 바뀌었다");
}

#[test]
fn stacked_cell_pictures_stay_in_their_cell_in_source_order() {
    use rhwp::{
        model::control::Control,
        renderer::render_tree::{RenderNode, RenderNodeType},
    };

    fn find_picture_cell(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 1 && cell.col == 0)
            && node
                .children
                .iter()
                .any(|child| matches!(child.node_type, RenderNodeType::Image(_)))
        {
            return Some(node);
        }
        node.children.iter().find_map(find_picture_cell)
    }

    let core = load(PICTURES);
    let Control::Table(source) = &core.document().sections[0].paragraphs[1].controls[0] else {
        panic!("원본 그림 표");
    };
    let source_pictures: Vec<_> = source
        .cells
        .iter()
        .find(|cell| cell.row == 1 && cell.col == 0)
        .expect("원본의 두 그림을 소유한 셀")
        .paragraphs
        .iter()
        .flat_map(|para| &para.controls)
        .filter_map(|control| match control {
            Control::Picture(picture) => Some(picture.image_attr.bin_data_id),
            _ => None,
        })
        .collect();
    assert_eq!(source_pictures.len(), 2);
    let tree = core.build_page_render_tree(0).expect("1쪽 출력");
    let cell = find_picture_cell(&tree.root).expect("머리기사와 본문 그림을 소유한 셀");
    let pictures: Vec<_> = cell
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::Image(image) => Some((image.bin_data_id, child.bbox)),
            _ => None,
        })
        .collect();
    assert_eq!(
        pictures.iter().map(|&(id, _)| id).collect::<Vec<_>>(),
        source_pictures,
        "두 원본 개체를 각각 한 번, 원래 순서로 그린다",
    );
    assert!(pictures
        .iter()
        .all(|(_, picture)| cell.bbox.contains(picture)));
    let (first, second) = (pictures[0].1, pictures[1].1);
    assert!(
        first.y + first.height <= second.y,
        "두 그림은 세로로 겹치지 않는다"
    );
}
