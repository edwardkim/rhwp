//! [#6313] 표 칸 안 그림이 자기 높이만큼 한 번 더 전진하지 않는다.
//!
//! `samples/issue6313/microbe_bank_cell_picture.hwpx` 는 원본(농촌진흥청 156624779,
//! 2.46MB)의 **구조 보존 슬라이스**다 — BinData 이미지만 8×8 PNG 로 바꿔 81KB 로
//! 줄였고, 조판은 `<hp:sz>` 선언 크기를 쓰므로 좌표가 원본과 같다.
//!
//! **근인 — 자기 변위 지문에서 오프셋이 빠졌다.**
//!
//! `#6110` 은 "빈 문단이 host 인 셀 float 의 저장 `vertical_pos` 가 **그 그림의 자기
//! 변위**면 흐름 오프셋이 아니다"를 세웠다. 그 지문을 `|vpos − 그림 높이| ≤ 1px` 로
//! 적었는데, 한글이 밀어 둔 줄의 vpos 는 **그림 바닥**을 가리키므로 세로 오프셋이
//! 0 이 아니면 높이만으로는 맞지 않는다.
//!
//! | | 저장 vpos | 그림 높이 | 세로 오프셋 | 합 |
//! |---|---|---|---|---|
//! | 왼쪽 칸 | 15250 | 14530 | **720** | **15250** |
//! | 오른쪽 칸 | 17188 | 16899 | **289** | **17188** |
//!
//! 둘 다 **단위까지** 맞는다. 종전 지문은 각각 9.6px·3.9px 차로 빗나가 vpos 를 흐름
//! 오프셋으로 신뢰했고, 그림이 제 높이만큼 더 내려가 칸과 **용지 밖**으로 나갔다
//! (아래끝 1146.6px·1198.3px, 칸은 738.1..984.4).
//!
//! `#6175`·`#6280` 이 세운 "개체 흐름 높이 = 높이 + 오프셋" 과 같은 계약이다.
//!
//! 한글 2022 실측(문서 저장 버전 = 한글 2020 → 가장 가까운 설치본):
//! 5쪽 아래 두 그림 상단 560.03pt · 574.05pt.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6313/microbe_bank_cell_picture.hwpx");
    DocumentCore::from_bytes(&std::fs::read(path).expect("정식 미생물 원본")).expect("문서 로드")
}

fn cell_images<'a>(
    node: &'a RenderNode,
    owner: Option<&'a RenderNode>,
    images: &mut Vec<(&'a RenderNode, &'a RenderNode)>,
) {
    let owner = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        owner
    };
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        images.push((node, owner.expect("셀 그림의 실제 소유 칸")));
    }
    for child in &node.children {
        cell_images(child, owner, images);
    }
}

fn inside(inner: &RenderNode, outer: &RenderNode) -> bool {
    // 좌표 변환의 부동소수점 반올림만 허용한다. 과거 픽셀 위치를 고정하지 않는다.
    let epsilon = outer.bbox.height.max(outer.bbox.width).max(1.0) * 1e-9;
    inner.bbox.x >= outer.bbox.x - epsilon
        && inner.bbox.y >= outer.bbox.y - epsilon
        && inner.bbox.x + inner.bbox.width <= outer.bbox.x + outer.bbox.width + epsilon
        && inner.bbox.y + inner.bbox.height <= outer.bbox.y + outer.bbox.height + epsilon
}

fn find<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find(child, predicate))
}

fn text(node: &RenderNode) -> String {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    own + &node.children.iter().map(text).collect::<String>()
}

#[test]
fn cell_pictures_stay_inside_the_page() {
    let core = core();
    // 독립 Print PDF는 5쪽이며 마지막 쪽의 검색 예시 그림 네 개를 보존한다.
    // 전쪽 Native/fresh WASM 최저 93.89126%를 확인한 뒤 기존 검사를 교정했다.
    assert_eq!(core.page_count(), 5, "독립 Print의 전쪽 소속 보존");
    let page = core.build_page_render_tree(4).expect("검색 방법 마지막 쪽");
    assert!(text(&page.root).contains("특허미생물 검색 방법"));
    let mut images = Vec::new();
    cell_images(&page.root, None, &mut images);
    assert_eq!(images.len(), 4, "Print의 검색 예시 네 그림을 보존");
    for (image, _) in images {
        assert!(
            inside(image, &page.root),
            "그림이 자기 페이지를 벗어났다: {:?}",
            image.bbox
        );
    }
}

#[test]
fn lower_cell_pictures_match_the_hangul_oracle() {
    let core = core();
    let page = core.build_page_render_tree(4).expect("검색 예시 마지막 쪽");
    let mut images = Vec::new();
    cell_images(&page.root, None, &mut images);
    // PDF에서 그림은 각 검색 단계 칸 안에 있다. 자기 높이를 다시 더하던
    // 결함은 아래 그림을 칸 밖으로 밀어낸다. 746.7/765.4px 핀을 대신한다.
    let mut lower_columns = Vec::new();
    for (image, owner) in images {
        assert!(
            inside(image, owner),
            "그림 {:?}이 소유 칸 {:?} 밖이다",
            image.bbox,
            owner.bbox
        );
        if let RenderNodeType::TableCell(cell) = &owner.node_type {
            if cell.row == 2 {
                lower_columns.push(cell.col);
            }
        }
    }
    lower_columns.sort_unstable();
    assert_eq!(lower_columns, [0, 1], "아래 두 검색 단계 그림의 칸 소속");

    // 같은 원본의 Print 3쪽은 수탁 표 → 분양 현황 제목 → 분양 표 순서다.
    // 앞 표 뒤 공백 줄의 높이를 버리면 뒤 표가 자기 제목을 덮었다.
    let page = core
        .build_page_render_tree(2)
        .expect("기탁 및 분양 현황 쪽");
    let table = |para| {
        find(&page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Table(data) if data.para_index == Some(para))
    }).expect("현황 표의 본문 소유")
    };
    let preceding = table(40);
    let following = table(41);
    let title = find(&page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(41))
            && text(node).contains("분양 현황")
    })
    .expect("분양 현황 제목");
    assert!(
        preceding.bbox.y + preceding.bbox.height <= title.bbox.y,
        "앞 표와 분양 현황 제목의 순서"
    );
    assert!(
        title.bbox.y + title.bbox.height <= following.bbox.y,
        "분양 표가 자기 제목을 덮으면 안 된다"
    );
    assert!(
        text(following).contains("970"),
        "분양 표의 마지막 합계 내용을 보존"
    );
}
