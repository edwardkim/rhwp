//! Keep shared SVG-image eligibility within the browser's bounded XML subset.
use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::canvaskit_policy::{
    analyze_canvaskit_document_preflight, analyze_canvaskit_replay_plan,
    CanvasKitPreflightPageBuild, CanvasKitReplayMode, CanvasKitReplayStatus,
};
use rhwp::renderer::render_tree::{BoundingBox, PageBackgroundImage, PageBackgroundNode};

fn state(contents: &str) -> (CanvasKitReplayStatus, bool) {
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">{contents}<path d="M0 0L10 10"/></svg>"#
    );
    let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
    let tree = PageLayerTree::new(
        10.0,
        10.0,
        LayerNode::leaf(
            bbox,
            None,
            vec![PaintOp::page_background(
                bbox,
                PageBackgroundNode {
                    background_color: None,
                    border_color: None,
                    border_width: 0.0,
                    gradient: None,
                    image: Some(PageBackgroundImage {
                        data: svg.into_bytes(),
                        fill_mode: Default::default(),
                        brightness: 0,
                        contrast: 0,
                        effect: Default::default(),
                    }),
                },
            )],
        ),
    );
    let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
    assert_eq!(plan.items.len(), 1);
    let preflight = analyze_canvaskit_document_preflight(
        1,
        CanvasKitReplayMode::Default,
        RenderProfile::Screen,
        |_, _| {
            Ok::<_, String>(CanvasKitPreflightPageBuild::Complete {
                tree: Box::new(tree.clone()),
                prelower_work_units: 0,
            })
        },
    );
    assert!(preflight.complete);
    (plan.items[0].status, preflight.eligible)
}

fn assert_rejected(contents: &str) {
    assert_eq!(
        state(contents),
        (CanvasKitReplayStatus::DirectRequired, false),
        "{contents:?}"
    );
}

#[test]
fn style_xml_comments_are_not_shared_direct_image_candidates() {
    for contents in [
        "<style>path{fill:red}<!-- ordinary --></style>",
        "<style><!--a--><g><!--b--></g></style>",
        "<s:style xmlns:s=\"http://www.w3.org/2000/svg\"><!--a--></s:style>",
    ] {
        assert_rejected(contents);
    }
}

#[test]
fn malformed_xml_comments_are_not_shared_direct_image_candidates() {
    for contents in ["<!-- a--b -->", "<!-- <!-- nested -->", "<!-- bad --->"] {
        assert_rejected(contents);
    }
}

#[test]
fn invalid_xml_comment_characters_are_not_shared_direct_image_candidates() {
    for character in [
        '\u{0}', '\u{8}', '\u{b}', '\u{c}', '\u{e}', '\u{1f}', '\u{fffe}', '\u{ffff}',
    ] {
        assert_rejected(&format!("<!-- {character} -->"));
    }
}

#[test]
fn ordinary_comments_and_comment_free_styles_remain_direct_candidates() {
    for contents in [
        "<!-- ordinary -->",
        "<!-- \t\r\n -->",
        "<style>path{fill:red}</style><!-- ordinary -->",
        "<style/><!-- ordinary -->",
        "<style>path{fill:red}</style>",
    ] {
        assert_eq!(
            state(contents),
            (CanvasKitReplayStatus::Direct, true),
            "{contents:?}"
        );
    }
}
