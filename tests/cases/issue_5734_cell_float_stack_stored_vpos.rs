//! 저장 그림 사다리의 순서·셀 포함·그림별 쪽 소속을 검증한다.
//! 같은 원본의 독립 한컴 Print PDF 전쪽 Native/fresh WASM 검증 후 교정한 기존 검사다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::{model::control::Control, DocumentCore};
use std::collections::BTreeMap;
const SAMPLE: &str = "samples/issue5734/cell_float_stack_stored_vpos.hwpx";

fn nodes(root: &RenderNode) -> Vec<&RenderNode> {
    let mut out = vec![root];
    for child in &root.children {
        out.extend(nodes(child));
    }
    out
}

#[test]
fn issue_5734_left_cell_floats_stack_by_stored_vpos() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 픽스처")).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        2,
        "독립 Print의 전체 쪽수와 대응 내용 소속을 함께 검사"
    );
    let Control::Table(source) = &core.document().sections[0].paragraphs[2].controls[0] else {
        panic!("그림 표 호스트");
    };
    let (cell_index, cell) = source
        .cells
        .iter()
        .enumerate()
        .find(|(_, cell)| cell.row == 5 && cell.col == 0)
        .expect("독립 PDF의 병합 그림 칸");
    let mut rendered = BTreeMap::new();
    let mut rectangles = BTreeMap::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("PDF 대응 쪽");
        for node in nodes(&tree.root) {
            if let RenderNodeType::Image(image) = &node.node_type {
                assert!(
                    rendered.insert(image.bin_data_id, page).is_none(),
                    "원본 그림 중복 금지"
                );
                rectangles.insert(image.bin_data_id, node.bbox);
            }
        }
        let owner = nodes(&tree.root).into_iter().find(|n| matches!(&n.node_type, RenderNodeType::TableCell(c) if c.model_cell_index == Some(cell_index as u32))).expect("양쪽에 이어지는 병합 칸");
        if page == 0 {
            // 한컴 첫 조각에서 완전히 소비한 두 오른쪽 행은 선언된 물리 높이 비를 보존한다.
            // 마지막 행만 줄간격으로 접으면 이 비가 깨진다. 화면의 절대 좌표는 고정하지 않는다.
            let scales: Vec<f64> = [5, 6]
                .iter()
                .map(|&row| {
                    let (index, original) = source
                        .cells
                        .iter()
                        .enumerate()
                        .find(|(_, c)| c.row == row && c.col == 1)
                        .expect("완전 소비한 형제 행");
                    let placed = nodes(&tree.root)
                        .into_iter()
                        .find(|n| {
                            matches!(
                                &n.node_type, RenderNodeType::TableCell(c)
                                    if c.model_cell_index == Some(index as u32)
                            )
                        })
                        .expect("첫 조각의 형제 행");
                    placed.bbox.height / f64::from(original.height)
                })
                .collect();
            assert!(
                (scales[0] - scales[1]).abs()
                    <= f64::EPSILON * scales[0].abs().max(scales[1].abs()) * 64.0,
                "완전히 소비한 물리 행의 높이 비 보존: {scales:?}"
            );
        }
        for (pi, para) in cell.paragraphs.iter().enumerate() {
            for (ci, control) in para.controls.iter().enumerate() {
                let Control::Picture(picture) = control else {
                    panic!("원본 그림");
                };
                let expected = if pi < 3 || ci == 0 { 0 } else { 1 };
                if expected != page {
                    continue;
                }
                let rectangle = rectangles
                    .get(&picture.image_attr.bin_data_id)
                    .expect("같은 쪽의 원본 그림");
                assert!(
                    rectangle.y >= owner.bbox.y
                        && rectangle.y + rectangle.height <= owner.bbox.y + owner.bbox.height,
                    "그림은 소유 셀의 세로 범위 안에 표시"
                );
            }
        }
    }
    let mut source_count = 0;
    for original in &source.cells {
        for para in &original.paragraphs {
            for control in &para.controls {
                if let Control::Picture(picture) = control {
                    source_count += 1;
                    assert!(
                        rendered.contains_key(&picture.image_attr.bin_data_id),
                        "원본 그림 누락 금지"
                    );
                }
            }
        }
    }
    assert_eq!(
        rendered.len(),
        source_count,
        "모든 원본 그림이 전체 쪽에서 한 번씩 표시"
    );
    let mut previous: Option<(u32, f64)> = None;
    for (pi, para) in cell.paragraphs.iter().enumerate() {
        for (ci, control) in para.controls.iter().enumerate() {
            let Control::Picture(picture) = control else {
                unreachable!();
            };
            let expected = if pi < 3 || ci == 0 { 0 } else { 1 };
            assert_eq!(
                rendered[&picture.image_attr.bin_data_id], expected,
                "Print에서 확인한 개별 그림의 쪽 소속"
            );
            let rectangle = rectangles[&picture.image_attr.bin_data_id];
            if let Some((page, bottom)) = previous {
                if page == expected {
                    assert!(
                        rectangle.y + f64::EPSILON * rectangle.y.abs().max(bottom.abs()) * 8.0
                            >= bottom,
                        "같은 쪽의 그림은 원본 순서로 겹침 없이 이어짐"
                    );
                }
            }
            previous = Some((expected, rectangle.y + rectangle.height));
        }
    }
    // 원본 행의 그림보다 큰 선언 빈 공간으로 예산을 줄인다. 내용 컷만 fit하더라도
    // 물리 행이 본문을 넘으면 같은 컷을 수용하지 않아야 한다. 원본 픽스처는 변경하지 않는다.
    let last_whole = source
        .cells
        .iter()
        .find(|c| c.row == 6 && c.col == 1)
        .unwrap();
    let Control::Picture(last_picture) = &last_whole.paragraphs[0].controls[0] else {
        panic!("마지막 완전 행의 원본 그림");
    };
    let shrink = (last_whole.height - last_picture.common.height) / 2;
    assert!(shrink > 0, "원본이 선언한 그림 아래 빈 공간");
    let mut reduced = core.document().clone();
    reduced.sections[0].section_def.page_def.height -= shrink;
    let body_bottom = {
        let page = &reduced.sections[0].section_def.page_def;
        rhwp::renderer::hwpunit_to_px(
            (page.height - page.margin_bottom - page.margin_footer) as i32,
            96.0,
        )
    };
    let mut narrowed = DocumentCore::from_bytes(
        &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap(),
    )
    .unwrap();
    narrowed.set_document(reduced);
    assert_eq!(
        narrowed.page_count(),
        2,
        "본문이 좁아져도 빈 꼬리 쪽을 만들지 않음"
    );
    let mut narrowed_images = BTreeMap::new();
    for page in 0..narrowed.page_count() {
        let tree = narrowed.build_page_render_tree(page).unwrap();
        for node in nodes(&tree.root) {
            if let RenderNodeType::Image(image) = &node.node_type {
                assert!(
                    narrowed_images.insert(image.bin_data_id, page).is_none(),
                    "이월 그림 중복 금지"
                );
            }
            if matches!(node.node_type, RenderNodeType::Table(_)) {
                let end = node.bbox.y + node.bbox.height;
                assert!(
                    end <= body_bottom + f64::EPSILON * body_bottom.abs().max(end.abs()) * 64.0,
                    "내용만 fit하는 컷으로 표를 본문 밖에 배치하지 않음: {end} > {body_bottom}"
                );
            }
        }
    }
    assert_eq!(
        narrowed_images.len(),
        source_count,
        "예산 실패 이월에서도 모든 원본 그림 보존"
    );
}

// #7445 보정66: 같은 원본 전2쪽 Native/fresh WASM raw 100% 및 직접 판독 후 추가.
// 독립 Print PDF cell_float_stack_stored_vpos-2020.pdf의 image rectangle(72dpi pt)을
// 96/72로 변환했다. 흰색 대체 그림은 raw 100%여도 위치 오류가 숨으므로 실제
// Native CLI의 최종 RenderTree에서 쪽·셀·원본 순서를 먼저 고정하고 시작점과 세로 끝점을 비교한다.
struct NativePictureOutput(std::path::PathBuf);
impl NativePictureOutput {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("output/pr-review/regression-temp");
        std::fs::create_dir_all(&root).expect("ignored 테스트 출력 상위 폴더");
        let path = root.join(format!(
            "rhwp-5734-print-pictures-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("Native 임시 출력 폴더");
        Self(path)
    }
}
impl Drop for NativePictureOutput {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn collect_print_pictures(
    node: &serde_json::Value,
    owner: Option<(u64, u64)>,
    out: &mut BTreeMap<(u64, u64), Vec<[f64; 4]>>,
) {
    let owner = if node["type"] == "Cell" {
        Some((node["row"].as_u64().unwrap(), node["col"].as_u64().unwrap()))
    } else {
        owner
    };
    if node["type"] == "Image" {
        let b = &node["bbox"];
        let (x, y, w, h) = (
            b["x"].as_f64().unwrap(),
            b["y"].as_f64().unwrap(),
            b["w"].as_f64().unwrap(),
            b["h"].as_f64().unwrap(),
        );
        out.entry(owner.expect("원본 그림의 소유 셀"))
            .or_default()
            .push([x, y, x + w, y + h]);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect_print_pictures(child, owner, out);
        }
    }
}

#[test]
fn native_export_keeps_print_picture_origins_and_vertical_bounds() {
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    let output = NativePictureOutput::new();
    let sample = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let result = std::process::Command::new(binary)
        .arg("export-render-tree")
        .arg(sample)
        .arg("-o")
        .arg(&output.0)
        .args(["--compat", "2022"])
        .output()
        .expect("실제 Native CLI 실행");
    assert!(result.status.success(), "Native 내보내기 실패: {result:?}");
    // (PDF쪽, 원본 행, 원본 열, 해당 쪽·셀의 그림 순서, PDF x0/y0/x1/y1 CSS px).
    // source/PDF SHA와 추출 근거는 보정66 보고서와 증적 README에 보존한다.
    let expected: [(usize, u64, u64, usize, [f64; 4]); 14] = [
        (1, 1, 0, 0, [80.756, 152.792, 384.433, 383.580]),
        (1, 1, 1, 0, [402.664, 176.608, 706.341, 397.965]),
        (1, 3, 0, 0, [84.115, 466.209, 387.792, 583.361]),
        (1, 3, 1, 0, [401.544, 446.552, 705.221, 603.180]),
        (1, 5, 0, 0, [84.115, 655.603, 387.792, 739.991]),
        (1, 5, 0, 1, [82.196, 749.260, 385.873, 777.229]),
        (1, 5, 0, 2, [81.556, 781.865, 385.233, 841.160]),
        (1, 5, 0, 3, [82.196, 854.425, 385.873, 977.971]),
        (1, 5, 1, 0, [401.544, 628.912, 705.221, 788.097]),
        (1, 6, 1, 0, [396.587, 817.505, 700.264, 1002.903]),
        (2, 5, 0, 0, [80.916, 147.039, 384.593, 257.319]),
        (2, 5, 0, 1, [79.477, 257.319, 388.431, 405.636]),
        (2, 5, 0, 2, [84.115, 405.636, 387.792, 501.691]),
        (2, 7, 1, 0, [396.587, 179.963, 700.264, 468.927]),
    ];
    let mut count = 0;
    for page in 1..=2 {
        let tree: serde_json::Value = serde_json::from_slice(
            &std::fs::read(output.0.join(format!("render_tree_{page:03}.json")))
                .expect("PDF 대응 쪽"),
        )
        .unwrap();
        let mut actual = BTreeMap::new();
        collect_print_pictures(&tree, None, &mut actual);
        count += actual.values().map(Vec::len).sum::<usize>();
        for &(p, row, col, ordinal, rectangle) in expected.iter().filter(|e| e.0 == page) {
            let placed = actual
                .get(&(row, col))
                .and_then(|images| images.get(ordinal))
                .expect("원본 셀·순서의 그림");
            for edge in [0, 1, 3] {
                let (value, reference) = (placed[edge], rectangle[edge]);
                assert!(
                    (value - reference).abs() <= 2.,
                    "Print {p}쪽 ({row},{col}) 그림{ordinal} 변{edge}: {value} vs PDF {reference}"
                );
            }
        }
    }
    assert_eq!(
        count,
        expected.len(),
        "두 쪽의 원본 그림 14개: 누락·추가 금지"
    );
    assert!(
        !output.0.join("render_tree_003.json").exists(),
        "빈 꼬리 쪽 금지"
    );
}
