//! [Issue #5585 국소형 ②] 한 문단이 품은 **형제 표**마다 문단 앵커 오프셋을 다시 물어
//! 예산을 깎던 결함의 가드.
//!
//! `vert_rel_to = Para` 의 양수 `vertOffset` 은 **문단 앵커로부터의** 거리다. 한 문단이
//! 표 여러 개를 품고 그것들이 쪽을 하나씩 차지하면, 두 번째 이후 표에게 그 오프셋은
//! **이미 앞 형제가 소진한** 값이다. 그런데 조판기는 쪽마다 다시 물어 `avail_for_rows`
//! 를 그만큼 깎았다.
//!
//! 원문과 정상 한컴 2020 출력은 `stored_float_anchor_control` fixture에 대응한다.
//! 정상 PDF는 841×595pt 가로 용지 86쪽이며, 전체 Native/fresh WASM 비교로 검증한다.
//!
//! 54쪽의 첫 수용 가능한 형제는 `pi=72, ci=2`, 55쪽은 앞 형제에서 앵커가
//! 소진된 `ci=1`이다. 이후 표의 문단 오프셋을 다시 예약하면 꼬리 쪽이 생긴다.
//! 79~81쪽의 `pi=83` 표도 원문 소유 순서와 소비된 앵커를 유지해야 한다.
//!
//! 행 높이 검사값은 같은 PDF의 가로 괘선 좌표에서 정했다. 완전 셀의 마지막
//! 줄간격을 중복 가산하면 19·20·37·44·75쪽 행이 커지고 뒤 내용이 밀린다.
//! 본문 깊이 검사는 꼬리말 쪽번호를 제외한다. 꼬리말까지 포함하면 실제 본문이
//! 약72px인 결함 쪽도 약774px를 채운 것으로 오인해 검사를 통과한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

/// 공개 검증 원문. 개인 Windows 경로가 없을 때 성공처럼 건너뛰지 않는다.
/// #7269에서 상단 저장 앵커 수용 범위를 넓혀도 형제 표의 흐름을 보존해야 한다.
/// 기준 PDF: `pdf/pr7382/1351000_policy_indicators-2020.pdf` (86쪽).
fn sample() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/stored_float_anchor_control/1351000_policy_indicators.hwp");
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("검증 원문을 읽을 수 없다 ({}): {error}", path.display()))
}

/// 쪽수는 한/글 2020 와 같은 **86쪽**이어야 한다 — 형제 표마다 앵커 오프셋을 다시 물면
/// 표 다섯 개가 쪼개져 91쪽이 된다.
#[test]
fn sibling_tables_do_not_recharge_the_paragraph_anchor_offset() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let pages = core.page_count();
    assert_eq!(
        pages, 86,
        "쪽수는 한/글 2020 와 같은 86쪽이어야 한다 — #5585 회귀. \
         형제 표마다 앵커 오프셋(11.3px)을 다시 물면 91쪽이 된다 (got {pages})"
    );

    use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
    fn table(node: &RenderNode, pi: usize, ci: usize) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(pi) && t.control_index == Some(ci)
                && t.cell_context.is_none())
        {
            return Some(node);
        }
        node.children.iter().find_map(|child| table(child, pi, ci))
    }
    fn cell(node: &RenderNode, row: u16, col: u16) -> Option<BoundingBox> {
        // 외곽 표의 직접 셀만 검사한다. 중첩 표의 같은 행·열과 혼동하지 않는다.
        node.children.iter().find_map(|child| {
            if matches!(&child.node_type, RenderNodeType::TableCell(c)
                if c.row == row && c.col == col)
            {
                Some(child.bbox)
            } else {
                None
            }
        })
    }
    // 독립 PDF의 표 상단 좌표. 쪽수만 같고 표가 바뀌거나 다시 밀려도 검출한다.
    for (page, pi, ci, top) in [
        (53, 72, 0, 20.7893),
        (54, 72, 2, 31.9827),
        (55, 72, 1, 20.7893),
        (79, 83, 0, 20.7893),
        (80, 83, 1, 20.7893),
        (81, 83, 2, 20.7893),
    ] {
        let tree = core
            .build_page_render_tree(page - 1)
            .expect("검증 쪽 렌더 트리");
        let actual = table(&tree.root, pi, ci).expect("기준 PDF에 대응하는 원문 표");
        assert!(
            (actual.bbox.y - top).abs() < 0.5,
            "{page}쪽 pi={pi} ci={ci}: 표 상단 {}px, 독립 PDF {top}px",
            actual.bbox.y
        );
    }
    // 독립 PDF의77·78쪽은 다른 호스트가 소유한 두 흐름 표다.
    // 앞 표의 점유 영역으로 다음 표 원점을 되돌려 같은 쪽에 겹치면 안 된다.
    for (page, own_pi, other_pi) in [(77, 81, 82), (78, 82, 81)] {
        let tree = core
            .build_page_render_tree(page - 1)
            .expect("독립 표 소유 쪽");
        assert!(
            table(&tree.root, own_pi, 0).is_some(),
            "{page}쪽 표 소유가 바뀌었다"
        );
        assert!(
            table(&tree.root, other_pi, 0).is_none(),
            "{page}쪽 앞뒤 독립 표가 겹쳤다"
        );
    }
    // 가로 괘선 중심 간 거리(96dpi). 마지막 줄간격으로 행을 키우면 실패한다.
    for (page, pi, row, col, height) in [
        (19, 38, 3, 1, 61.4073),
        (20, 39, 2, 10, 40.4580),
        (37, 56, 4, 0, 76.7587),
        (44, 63, 1, 10, 45.4153),
        (75, 79, 1, 1, 40.6180),
    ] {
        let tree = core
            .build_page_render_tree(page - 1)
            .expect("행 높이 검증 쪽");
        let outer = table(&tree.root, pi, 0).expect("검증 외곽 표");
        let actual = cell(outer, row, col).expect("기준 PDF에 대응하는 셀");
        assert!(
            (actual.height - height).abs() < 0.5,
            "{page}쪽 r={row} c={col}: 행 높이 {}px, 독립 PDF {height}px",
            actual.height
        );
    }
}

/// 65.7px 짜리 꼬리 조각 쪽이 없어야 한다 — 본문은744.6px다.
/// CLI의 최종 트리를 검사해 실제 출력 경로의 꼬리말·본문 소유를 대조한다.
#[test]
fn no_sliver_tail_pages_remain() {
    struct RenderOutput(std::path::PathBuf);
    impl Drop for RenderOutput {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("시스템 시각")
        .as_nanos();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = RenderOutput(root.join(format!(
        "output/pr-review/regression-temp/5585-{}-{stamp}",
        std::process::id()
    )));
    std::fs::create_dir_all(&output.0).expect("검증 출력 디렉터리");
    // 이전 결함의 봉인 바이너리 재현에만 명시적으로 출력기를 바꿀 수 있다.
    // 일반 회귀 실행은 현재 Cargo가 빌드한 출력기를 사용한다.
    let bin = std::env::var_os("RHWP_5585_RENDER_BIN")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_rhwp"))
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into());
    let result = std::process::Command::new(bin)
        .arg("export-render-tree")
        .arg(root.join("tests/fixtures/stored_float_anchor_control/1351000_policy_indicators.hwp"))
        .arg("--output")
        .arg(&output.0)
        .output()
        .expect("CLI 최종 트리 출력");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut slivers = Vec::new();
    // 원본 pi72 구간의46~75쪽만 본다. 앞쪽에는 정상적으로 짧은 쪽이 있다.
    for page in 46..=75 {
        let path = output.0.join(format!("render_tree_{page:03}.json"));
        let tree: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).expect("본문 검증 쪽 최종 트리"))
                .expect("본문 검증 트리 해석");
        let mut deepest = 0.0f64;
        fn walk(n: &serde_json::Value, deepest: &mut f64, in_body: bool) {
            let in_body = in_body || n["type"] == "Body";
            if in_body && n["type"] == "TextRun" {
                let end = n["bbox"]["y"].as_f64().expect("본문 글자 원점")
                    + n["bbox"]["h"].as_f64().expect("본문 글자 높이");
                *deepest = deepest.max(end);
            }
            if let Some(children) = n["children"].as_array() {
                for c in children {
                    walk(c, deepest, in_body);
                }
            }
        }
        walk(&tree, &mut deepest, false);
        if deepest > 0.0 && deepest < 150.0 {
            slivers.push((page, (deepest * 10.0).round() / 10.0));
        }
    }
    assert!(
        slivers.is_empty(),
        "본문744.6px인데150px도 못 채운 꼬리 조각 쪽이 있다 — #5585 회귀. {slivers:?}"
    );
}
