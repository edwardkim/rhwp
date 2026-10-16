//! #7445 지정 댓글의 생물독 원본: 독립 PDF 205쪽 전수 시각 수용 후 복구한 회귀.
//! C81 Native/fresh WASM 전205쪽96DPI raw 최저95.69921/95.70411%, 미달0.
//! 원본 셀·문단·그림의 소속, 내용 보존과 상대 배치를 검사하고, 양쪽98% 이상인
//! 중요한 그림 경계만 독립 Print의 ±2 CSSpx로 보호한다. 출처는
//! completed-20261009/regression-policy-20261015/gray-grid-raster-20261017/README.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue5941/1480000-201900698-native-neartop-reset.hwp";
const SECTION: usize = 8;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).expect("전쪽 시각 수용한 원본");
    DocumentCore::from_bytes(&bytes).expect("생물독 원본 로드")
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = vec![node];
    for child in &node.children {
        out.extend(nodes(child));
    }
    out
}

fn text(node: &RenderNode) -> String {
    nodes(node)
        .into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some(run.display_or_text()),
            _ => None,
        })
        .collect()
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn tables(root: &RenderNode, host: usize) -> Vec<&RenderNode> {
    nodes(root)
        .into_iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t)
            if t.section_index == Some(SECTION) && t.para_index == Some(host)
                && t.control_index == Some(0))
        })
        .collect()
}

fn paragraph_runs(root: &RenderNode, host: usize) -> Vec<&RenderNode> {
    paragraph_runs_in(root, SECTION, host)
}

fn paragraph_runs_in(root: &RenderNode, section: usize, host: usize) -> Vec<&RenderNode> {
    nodes(root)
        .into_iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::TextRun(t)
            if t.section_index == Some(section) && t.para_index == Some(host)
                && t.cell_context.is_none())
        })
        .collect()
}

#[test]
fn picture_successor_keeps_caption_and_following_body_below_the_picture() {
    let core = core();
    assert_eq!(core.page_count(), 205);
    let source = &core.document().sections[4].paragraphs;
    let Control::Picture(picture) = &source[343].controls[0] else {
        panic!("독립 Print 63쪽의 예측 결과 그림");
    };
    let tree = core.build_page_render_tree(62).unwrap();
    let images: Vec<_> = nodes(&tree.root)
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Image(image)
            if image.section_index == Some(4) && image.para_index == Some(343))
        })
        .collect();
    assert_eq!(images.len(), 1, "그림의 단일 쪽 소유");
    let RenderNodeType::Image(image) = &images[0].node_type else {
        unreachable!();
    };
    assert_eq!(image.bin_data_id, picture.image_attr.bin_data_id);
    let image_bottom = images[0].bbox.y + images[0].bbox.height;
    let mut previous_bottom = image_bottom;
    for host in 344..=346 {
        let runs = paragraph_runs_in(&tree.root, 4, host);
        assert!(!runs.is_empty(), "캡션·주석·후속 본문의 같은 쪽 소유");
        let rendered: String = runs.iter().map(|node| text(node)).collect();
        assert_eq!(
            compact(&rendered),
            compact(&source[host].text),
            "내용 순서·누락·중복"
        );
        let top = runs
            .iter()
            .map(|node| node.bbox.y)
            .reduce(f64::min)
            .unwrap();
        assert!(top >= previous_bottom, "그림·캡션·주석·본문의 겹침 금지: pi={host}, top={top}, preceding_bottom={previous_bottom}");
        previous_bottom = runs
            .iter()
            .map(|node| node.bbox.y + node.bbox.height)
            .reduce(f64::max)
            .unwrap();
        if host == 344 {
            // 같은 원본 독립 Print p63: 캡션 잉크 상단699.79083pt,
            // 그림 하단688.64697pt. C81 전205쪽은 양쪽90% 이상이고
            // 이 쪽은 raw99.95764/99.95854% 및 review/overlay 직접 판독 완료.
            assert!((top - 933.05444).abs() <= 2.0, "독립 캡션 원점 ±2CSSpx");
        }
    }
    assert!(
        (image_bottom - 918.19596).abs() <= 2.0,
        "그림 자체 원점은 유지"
    );
}

#[cfg(feature = "native-skia")]
#[test]
fn gray_grid_keeps_its_print_coverage_cell_owners_and_pdf_origin() {
    use rhwp::paint::RenderProfile;
    use rhwp::renderer::{layer_renderer::LayerRasterRenderer, skia::SkiaLayerRenderer};

    let core = core();
    assert_eq!(core.page_count(), 205);
    let Control::Table(source) = &core.document().sections[SECTION].paragraphs[170].controls[0]
    else {
        panic!("독립 Print 143쪽의 표42");
    };
    let tree = core.build_page_render_tree(142).unwrap();
    let fragments = tables(&tree.root, 170);
    assert_eq!(fragments.len(), 1, "39행 표42는 같은 쪽의 온전한 조각");
    let table = fragments[0];
    let mut owners = vec![0; source.cells.len()];
    for child in &table.children {
        let RenderNodeType::TableCell(cell) = &child.node_type else {
            continue;
        };
        let index = cell.model_cell_index.expect("원본 셀 소속") as usize;
        owners[index] += 1;
        let original: String = source.cells[index]
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect();
        assert_eq!(compact(&text(child)), compact(&original), "셀 내용 보존");
    }
    assert!(owners.iter().all(|count| *count == 1), "셀 누락·중복 금지");
    // 독립 Print의 같은 표 왼쪽75.32·오른쪽707.78 CSSpx, ±2CSSpx.
    assert!((table.bbox.x - 75.32).abs() <= 2.0);
    assert!((table.bbox.x + table.bbox.width - 707.78).abs() <= 2.0);

    // C80의 같은 회색 grid 경계143쪽: 양쪽 raw99.97632/99.97501%.
    // 독립 PDF를96DPI로 인쇄 raster한 내부 선은 x700에서 RGB224다.
    // 모델 좌표만 검사하면 두 픽셀에 분산되어 사라진 회색 선을 놓치므로
    // 최종 Native paint에서 같은 독립 위치±2px와 AA 색 차이8을 확인한다.
    let layer = core
        .build_page_layer_tree_with_profile(142, RenderProfile::Print)
        .unwrap();
    let png = SkiaLayerRenderer::new().render_png(&layer).unwrap();
    let pixels = image::load_from_memory(&png).unwrap().to_rgb8();
    for expected_y in [257, 273, 336, 399] {
        let visible = (expected_y - 2..=expected_y + 2).any(|y| {
            let color = pixels.get_pixel(700, y).0;
            color.into_iter().all(|value| value.abs_diff(224) <= 8)
        });
        assert!(
            visible,
            "표42 회색 선이 독립 Print y{expected_y}±2px에 표시"
        );
    }
}

#[test]
fn cellbreak_boundary_rows_belong_wholly_to_their_pdf_pages() {
    let core = core();
    assert_eq!(core.page_count(), 205, "독립 PDF와 같은 원본 쪽 수");
    let Control::Table(source) = &core.document().sections[SECTION].paragraphs[174].controls[0]
    else {
        panic!("PDF 표44의 원본 호스트");
    };
    // PDF 147쪽은 Hyaluronidase, 148쪽은 Venom prothrombin activator로 시작한다.
    for (row, owner, first_label) in [
        (36, 147, "Hyaluronidase"),
        (69, 148, "Venom prothrombin activator"),
    ] {
        let originals: Vec<_> = source
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.row == row)
            .collect();
        assert_eq!(originals.len(), 3, "원본 경계 행의 세 셀");
        assert_eq!(originals[0].1.paragraphs[0].text, first_label);
        for (index, cell) in originals {
            let mut occurrences = 0;
            for page in 146..=148 {
                let tree = core.build_page_render_tree(page - 1).expect("PDF 대응 쪽");
                for table in tables(&tree.root, 174) {
                    for node in &table.children {
                        if matches!(&node.node_type, RenderNodeType::TableCell(c)
                            if c.model_cell_index == Some(index as u32))
                        {
                            occurrences += 1;
                            assert_eq!(page, owner, "경계 행의 셀은 같은 쪽에 완전히 소속");
                            let expected: String =
                                cell.paragraphs.iter().map(|p| p.text.as_str()).collect();
                            assert_eq!(
                                compact(&text(node)),
                                compact(&expected),
                                "셀 문단 누락·중복 금지"
                            );
                        }
                    }
                }
            }
            assert_eq!(occurrences, 1, "경계 셀은 인접 쪽 전체에서 한 번만 표시");
        }
    }
    // Independent Print PDF: table 15 owns rows 0..36, 37..74,
    // 75..109 and 110..150 on pages 46..49 respectively. The first
    // control-only host must not consume a caption-sized extra line.
    let Control::Table(source) = &core.document().sections[4].paragraphs[278].controls[0] else {
        panic!("PDF 표15의 원본 호스트");
    };
    let mut occurrences = vec![0usize; source.cells.len()];
    for (page, first_row, last_row) in [(46, 0, 36), (47, 37, 74), (48, 75, 109), (49, 110, 150)] {
        let tree = core
            .build_page_render_tree(page - 1)
            .expect("표15의 PDF 대응 쪽");
        let rendered: Vec<_> = nodes(&tree.root)
            .into_iter()
            .filter(|n| {
                matches!(&n.node_type, RenderNodeType::Table(t)
                if t.section_index == Some(4) && t.para_index == Some(278)
                    && t.control_index == Some(0))
            })
            .collect();
        assert_eq!(rendered.len(), 1, "표15는 각 쪽의 한 조각에 소속");
        let table = rendered[0];
        let mut rows = std::collections::BTreeSet::new();
        for cell_node in &table.children {
            let RenderNodeType::TableCell(cell) = &cell_node.node_type else {
                continue;
            };
            let index = cell.model_cell_index.expect("원본 셀 소속") as usize;
            let original = &source.cells[index];
            occurrences[index] += 1;
            rows.insert(original.row);
            assert!(
                (first_row..=last_row).contains(&original.row),
                "경계 행은 PDF의 같은 쪽에 완전히 소속"
            );
            let expected: String = original
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect();
            assert_eq!(
                compact(&text(cell_node)),
                compact(&expected),
                "셀 내용 누락·중복 금지"
            );
            assert!(
                cell_node.bbox.y >= table.bbox.y
                    && cell_node.bbox.y + cell_node.bbox.height
                        <= table.bbox.y + table.bbox.height + f64::EPSILON * 8192.0,
                "온전한 셀은 수용한 표 조각 안에 포함"
            );
        }
        assert_eq!(
            rows,
            (first_row..=last_row).collect(),
            "수용한 모든 원본 행 보존"
        );
    }
    assert!(
        occurrences.iter().all(|count| *count == 1),
        "표15의 모든 원본 셀은 네 쪽 전체에 정확히 한 번 표시"
    );
}

#[test]
fn same_line_pictures_share_the_source_baseline() {
    let core = core();
    // C80 전205쪽96DPI raw Native/fresh 최저93.26163/93.26176%.
    // 같은 세 그림 동작의 양쪽 raw 최솟값:172쪽99.99587%,168쪽99.98152%.
    // 독립 Print image matrix의 pt 좌표를96DPI CSSpx로 환산했다.
    for (page, host, pdf_third_x, pdf_right) in [
        (172, 371, 477.184, 711.1387),
        (168, 323, 501.0107, 717.6947),
    ] {
        let source = &core.document().sections[SECTION].paragraphs[host];
        let line = &source.line_segs[0];
        let ratio = f64::from(line.baseline_distance) / f64::from(line.text_height);
        let tree = core
            .build_page_render_tree(page - 1)
            .expect("독립 PDF 그림 쪽");
        let mut pictures: Vec<_> = nodes(&tree.root)
            .into_iter()
            .filter(|n| {
                matches!(&n.node_type, RenderNodeType::Image(i)
                if i.section_index == Some(SECTION) && i.para_index == Some(host))
            })
            .collect();
        pictures.sort_by_key(|n| match &n.node_type {
            RenderNodeType::Image(i) => i.control_index,
            _ => None,
        });
        assert_eq!(pictures.len(), 3, "같은 줄의 세 원본 그림을 한 번 표시");
        let tallest = pictures.iter().map(|n| n.bbox.height).fold(0.0, f64::max);
        let reference = pictures[2].bbox.y + pictures[2].bbox.height * ratio;
        for (index, node) in pictures.iter().enumerate() {
            let Control::Picture(original) = &source.controls[index] else {
                panic!("원본 그림 컨트롤");
            };
            assert!(original.common.treat_as_char);
            assert!(original.caption.is_none());
            let RenderNodeType::Image(image) = &node.node_type else {
                unreachable!()
            };
            assert_eq!(image.control_index, Some(index), "누락·중복 없는 원본 소속");
            let own_baseline = node.bbox.y + node.bbox.height * ratio;
            assert!(
                ((own_baseline - reference) / tallest).abs() < 1.0 / f64::from(line.text_height),
                "같은 줄의 그림은 각자 높이에 따른 기준선을 공유"
            );
        }
        let source_extent =
            rhwp::renderer::hwpunit_to_px(line.segment_width, rhwp::renderer::DEFAULT_DPI);
        let last = pictures[2];
        let painted_extent = last.bbox.x + last.bbox.width - pictures[0].bbox.x;
        // 줄말 공백의 정수 반올림이 분수 글리프 폭과 혼합되지 않도록 원본 폭과 대조한다.
        if source.line_segs.len() > 1 {
            assert!((painted_extent - source_extent).abs() <= 4.0 / 75.0,
                "뒤 줄이 있는 그림 줄은 정렬 상자를 채움:{page}쪽 실제{painted_extent}, 원본{source_extent}");
        } else {
            assert!(
                painted_extent <= source_extent + 4.0 / 75.0,
                "마지막 줄은 자연 폭을 유지하며 원본 상자 안에 표시"
            );
        }
        assert!(
            (last.bbox.x - pdf_third_x).abs() <= 2.0,
            "{page}쪽 세 번째 그림 시작은 독립 Print ±2 CSSpx"
        );
        assert!(
            (last.bbox.x + last.bbox.width - pdf_right).abs() <= 2.0,
            "{page}쪽 마지막 그림 끝은 독립 Print ±2 CSSpx"
        );
    }
}

#[test]
fn deferred_whole_table_keeps_caption_cells_and_following_body() {
    let core = core();
    let before = core.build_page_render_tree(153).expect("PDF 154쪽");
    assert!(
        tables(&before.root, 242).is_empty(),
        "앞 쪽에 캡션/머리행만 남기지 않는다"
    );
    let tree = core.build_page_render_tree(154).expect("PDF 155쪽");
    let rendered = tables(&tree.root, 242);
    assert_eq!(rendered.len(), 1, "완전 표는 뒤 쪽 한 프레임에 소속");
    let Control::Table(source) = &core.document().sections[SECTION].paragraphs[242].controls[0]
    else {
        panic!("PDF 표48의 원본 호스트")
    };
    let table = rendered[0];
    for (index, cell) in source.cells.iter().enumerate() {
        let owned: Vec<_> = table.children.iter().filter(|n|
            matches!(&n.node_type, RenderNodeType::TableCell(c) if c.model_cell_index == Some(index as u32)))
            .collect();
        assert_eq!(owned.len(), 1, "머리행과 모든 원본 셀을 한 번 보존");
        let expected: String = cell.paragraphs.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(
            compact(&text(owned[0])),
            compact(&expected),
            "완전 표의 셀 내용 보존"
        );
    }
    let page_text = compact(&text(&tree.root));
    for caption in &source.caption.as_ref().expect("원본 상단 캡션").paragraphs {
        assert!(
            page_text.contains(&compact(&caption.text)),
            "캡션은 전체 표와 같은 쪽"
        );
    }
    let body = paragraph_runs(&tree.root, 244);
    let original_body = &core.document().sections[SECTION].paragraphs[244].text;
    assert_eq!(
        compact(&body.iter().map(|n| text(n)).collect::<String>()),
        compact(original_body),
        "뒤 본문 전체 보존"
    );
    assert!(
        body.iter()
            .all(|n| n.bbox.y >= table.bbox.y + table.bbox.height),
        "뒤 본문은 표 이후에 표시"
    );
}

#[test]
fn visible_picture_successor_consumes_before_spacing_once() {
    let core = core();
    let paragraphs = &core.document().sections[SECTION].paragraphs;
    let Control::Picture(source) = &paragraphs[18].controls[0] else {
        panic!("PDF 그림92")
    };
    assert!(
        source.caption.is_none(),
        "설명은 붙임 캡션이 아닌 다음 일반 문단"
    );
    let tree = core.build_page_render_tree(124).expect("PDF 125쪽");
    let pictures: Vec<_> = nodes(&tree.root).into_iter().filter(|n|
        matches!(&n.node_type, RenderNodeType::Image(i)
            if i.section_index == Some(SECTION) && i.para_index == Some(18) && i.control_index == Some(0)))
        .collect();
    assert_eq!(pictures.len(), 1, "그림92를 한 번 표시");
    let explanation = paragraph_runs(&tree.root, 19);
    assert!(!explanation.is_empty(), "그림 설명 보존");
    assert_eq!(
        compact(&explanation.iter().map(|n| text(n)).collect::<String>()),
        compact(&paragraphs[19].text)
    );
    let picture = pictures[0];
    let first = explanation
        .iter()
        .map(|n| n.bbox.y)
        .fold(f64::INFINITY, f64::min);
    // 원본 호스트·그림 높이·후속 저장 줄은 같은 경계를 증명하고 PDF도 인접 배치를 보인다.
    assert!(
        ((first - picture.bbox.y - picture.bbox.height) / picture.bbox.height).abs()
            < 1.0 / f64::from(source.common.height),
        "후속 일반 문단의 앞 간격을 그림 끝에 다시 더하지 않는다"
    );
    let body = paragraph_runs(&tree.root, 21);
    assert!(!body.is_empty(), "뒤 본문은 같은 쪽에 보존");
    assert_eq!(
        compact(&body.iter().map(|n| text(n)).collect::<String>()),
        compact(&paragraphs[21].text)
    );
    let explanation_end = explanation
        .iter()
        .map(|n| n.bbox.y + n.bbox.height)
        .fold(0.0, f64::max);
    assert!(
        body.iter().all(|n| n.bbox.y > explanation_end),
        "그림 → 설명 → 뒤 본문 순서"
    );
}
