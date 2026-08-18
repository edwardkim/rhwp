fn parse_arrow(name: &str) -> ArrowStyle {
    match name {
        "none" => ArrowStyle::None,
        "arrow" => ArrowStyle::Arrow,
        "concaveArrow" => ArrowStyle::ConcaveArrow,
        "openDiamond" => ArrowStyle::OpenDiamond,
        "openCircle" => ArrowStyle::OpenCircle,
        "openSquare" => ArrowStyle::OpenSquare,
        "diamond" => ArrowStyle::Diamond,
        "circle" => ArrowStyle::Circle,
        "square" => ArrowStyle::Square,
        other => panic!("unknown arrow {other}"),
    }
}

fn parse_line_type(name: &str) -> LineRenderType {
    match name {
        "single" => LineRenderType::Single,
        "double" => LineRenderType::Double,
        "thinThickDouble" => LineRenderType::ThinThickDouble,
        "thickThinDouble" => LineRenderType::ThickThinDouble,
        "thinThickThinTriple" => LineRenderType::ThinThickThinTriple,
        other => panic!("unknown line type {other}"),
    }
}

fn parse_dash(name: &str) -> StrokeDash {
    match name {
        "solid" => StrokeDash::Solid,
        "dash" => StrokeDash::Dash,
        "dot" => StrokeDash::Dot,
        "dashDot" => StrokeDash::DashDot,
        "dashDotDot" => StrokeDash::DashDotDot,
        other => panic!("unknown dash {other}"),
    }
}

fn parse_decoration_kind(name: &str) -> TextDecorationKind {
    match name {
        "underline" => TextDecorationKind::Underline,
        "strikethrough" => TextDecorationKind::Strikethrough,
        "emphasisDot" => TextDecorationKind::EmphasisDot,
        other => panic!("unknown decoration {other}"),
    }
}

fn sample_path(style: ShapeStyle, line_style: Option<LineStyle>) -> PathNode {
    let mut path = PathNode::new(
        vec![
            PathCommand::MoveTo(0.0, 0.0),
            PathCommand::LineTo(20.0, 8.0),
        ],
        style,
        None,
    );
    path.line_style = line_style;
    path
}

fn css_to_colorref(css: &str) -> u32 {
    let hex = css.trim().trim_start_matches('#');
    let r = u32::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u32::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u32::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    (b << 16) | (g << 8) | r
}

fn shadow_from_row(row: &serde_json::Value) -> crate::renderer::ShadowStyle {
    crate::renderer::ShadowStyle {
        shadow_type: row["shadowType"].as_u64().unwrap_or(1) as u32,
        color: 0x0000_0000,
        offset_x: row["offsetX"].as_f64().unwrap_or(1.5),
        offset_y: row["offsetY"].as_f64().unwrap_or(1.5),
        alpha: row["alpha"].as_u64().unwrap_or(80) as u8,
    }
}

fn op_from_matrix_row(row: &serde_json::Value) -> PaintOp {
    let family = row["family"].as_str().unwrap();
    let op_type = row["opType"].as_str().unwrap();
    match family {
        "lineArrow" | "compoundLine" | "lineShadow" | "crossVector" => {
            let mut style = LineStyle::default();
            if let Some(name) = row["startArrow"].as_str() {
                style.start_arrow = parse_arrow(name);
            }
            if let Some(name) = row["endArrow"].as_str() {
                style.end_arrow = parse_arrow(name);
            }
            if let Some(size) = row["arrowSize"].as_u64() {
                style.start_arrow_size = size as u8;
                style.end_arrow_size = size as u8;
            }
            if let Some(name) = row["lineType"].as_str() {
                style.line_type = parse_line_type(name);
            }
            if let Some(name) = row["dash"].as_str() {
                style.dash = parse_dash(name);
            }
            if row
                .get("shadowType")
                .and_then(|value| value.as_u64())
                .is_some()
            {
                style.shadow = Some(shadow_from_row(row));
            }
            if op_type == "path" {
                PaintOp::path(bbox(), sample_path(ShapeStyle::default(), Some(style)))
            } else {
                PaintOp::line(bbox(), LineNode::new(0.0, 0.0, 24.0, 6.0, style))
            }
        }
        "shapeShadow" | "patternFill" => {
            let mut style = ShapeStyle::default();
            if family == "shapeShadow" {
                style.shadow = Some(shadow_from_row(row));
            }
            if family == "patternFill" {
                style.pattern = Some(crate::renderer::PatternFillInfo {
                    pattern_type: row["patternType"].as_i64().unwrap_or(1) as i32,
                    pattern_color: css_to_colorref(
                        row["patternColor"].as_str().unwrap_or("#000000"),
                    ),
                    background_color: css_to_colorref(
                        row["backgroundColor"].as_str().unwrap_or("#ffffff"),
                    ),
                });
            }
            match op_type {
                "ellipse" => PaintOp::ellipse(bbox(), EllipseNode::new(style, None)),
                "path" => PaintOp::path(bbox(), sample_path(style, None)),
                _ => PaintOp::rectangle(bbox(), RectangleNode::new(0.0, style, None)),
            }
        }
        "unsupportedTextDecoration" => {
            let mut run = text_run("A");
            let shape = row["shape"].as_u64().unwrap_or(0) as u8;
            let emphasis = row["emphasisDot"].as_u64().unwrap_or(0) as u8;
            run.style.underline_shape = shape;
            run.style.strike_shape = shape;
            run.style.emphasis_dot = emphasis;
            PaintOp::text_decoration(bbox(), run, parse_decoration_kind(op_type))
        }
        "invalidTabLeader" => {
            let mut run = text_run("A");
            let start = row["startX"].as_f64().unwrap_or(f64::NAN);
            let end = row["endX"].as_f64().unwrap_or(f64::NAN);
            run.style.tab_leaders.push(crate::renderer::TabLeaderInfo {
                start_x: start,
                end_x: end,
                fill_type: row["fillType"].as_u64().unwrap_or(1) as u8,
            });
            PaintOp::tab_leader(bbox(), run)
        }
        "footnoteMarker" => PaintOp::footnote_marker(
            bbox(),
            FootnoteMarkerNode {
                number: row["number"].as_u64().unwrap_or(1) as u16,
                text: row["text"].as_str().unwrap_or("1)").to_string(),
                base_font_size: row["fontSize"].as_f64().unwrap_or(7.0),
                font_family: row["fontFamily"].as_str().unwrap_or("Test").to_string(),
                color: 0x0000_0000,
                section_index: 0,
                para_index: 0,
                control_index: 0,
            },
        ),
        "visualItemLimitExceeded" => {
            let count = row["itemCount"].as_u64().unwrap_or(1) as usize;
            match op_type {
                "tabLeader" => {
                    let mut run = text_run("A");
                    run.style.tab_leaders = (0..count)
                        .map(|index| crate::renderer::TabLeaderInfo {
                            start_x: index as f64,
                            end_x: index as f64 + 0.5,
                            fill_type: 1,
                        })
                        .collect();
                    PaintOp::tab_leader(bbox(), run)
                }
                "textDecoration" => {
                    let mut run = text_run("\u{0017}");
                    run.display_text = Some("A".repeat(count));
                    PaintOp::text_decoration(bbox(), run, TextDecorationKind::Underline)
                }
                "charOverlap" => {
                    let mut run = text_run(&" ".repeat(count));
                    run.char_overlap = Some(CharOverlapInfo {
                        border_type: 1,
                        inner_char_size: 100,
                    });
                    PaintOp::char_overlap(bbox(), run)
                }
                _ => {
                    let run = text_run(&" ".repeat(count));
                    PaintOp::text_control_mark(bbox(), run)
                }
            }
        }
        other => panic!("unhandled family {other}"),
    }
}

fn expected_status(name: &str) -> CanvasKitReplayStatus {
    match name {
        "direct" => CanvasKitReplayStatus::Direct,
        "directRequired" => CanvasKitReplayStatus::DirectRequired,
        other => panic!("unknown status {other}"),
    }
}

fn load_jsonl(name: &str) -> Vec<serde_json::Value> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/m07_pack")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).expect("jsonl row"))
        .collect()
}

fn load_matrix() -> Vec<serde_json::Value> {
    load_jsonl("reason-matrix.jsonl")
}

fn load_envelopes() -> Vec<serde_json::Value> {
    load_jsonl("envelopes.jsonl")
}

#[test]
fn m07_pack_reason_matrix_rows_match_live_policy() {
    let rows = load_matrix();
    assert!(rows.len() >= 4000, "matrix too small: {}", rows.len());
    for row in &rows {
        let case_id = row["caseId"].as_str().unwrap();
        let op = op_from_matrix_row(row);
        let plan =
            analyze_canvaskit_replay_plan(&tree_with_ops(vec![op]), CanvasKitReplayMode::Default);
        assert_eq!(
            plan.items[0].status,
            expected_status(row["status"].as_str().unwrap()),
            "{case_id}"
        );
        assert_eq!(
            plan.items[0].detail.as_deref(),
            row["detail"].as_str(),
            "{case_id}"
        );
        if row["status"].as_str() == Some("direct") {
            assert_eq!(plan.summary.hidden_overlay_violations, 0, "{case_id}");
        } else {
            assert_eq!(plan.summary.hidden_overlay_violations, 1, "{case_id}");
        }
    }
}

#[test]
fn m07_pack_envelopes_match_live_summary() {
    let envelopes = load_envelopes();
    assert_eq!(envelopes.len(), load_matrix().len());
    for envelope in envelopes.iter().step_by(17) {
        let case_id = envelope["caseId"].as_str().unwrap();
        let input = &envelope["input"];
        let plan = analyze_canvaskit_replay_plan(
            &tree_with_ops(vec![op_from_matrix_row(input)]),
            CanvasKitReplayMode::Default,
        );
        assert_eq!(
            plan.summary.direct_items,
            envelope["summary"]["directItems"].as_u64().unwrap() as u32,
            "{case_id}"
        );
        assert_eq!(
            plan.summary.direct_required_items,
            envelope["summary"]["directRequiredItems"].as_u64().unwrap() as u32,
            "{case_id}"
        );
        assert_eq!(
            plan.items[0].reason,
            if envelope["items"][0]["reason"].as_str() == Some("directReplaySupported") {
                CanvasKitReplayReason::DirectReplaySupported
            } else {
                CanvasKitReplayReason::HiddenOverlayForbidden
            },
            "{case_id}"
        );
    }
}

#[test]
fn m07_pack_unknown_tab_fill_is_direct_inverted_range_is_not() {
    let mut valid_unknown = text_run("A");
    valid_unknown
        .style
        .tab_leaders
        .push(crate::renderer::TabLeaderInfo {
            start_x: 1.0,
            end_x: 12.0,
            fill_type: 15,
        });
    let valid_plan = analyze_canvaskit_replay_plan(
        &tree_with_ops(vec![PaintOp::tab_leader(bbox(), valid_unknown)]),
        CanvasKitReplayMode::Default,
    );
    assert_eq!(valid_plan.items[0].status, CanvasKitReplayStatus::Direct);
    assert_eq!(valid_plan.items[0].detail, None);

    let mut inverted = text_run("A");
    inverted
        .style
        .tab_leaders
        .push(crate::renderer::TabLeaderInfo {
            start_x: 12.0,
            end_x: 1.0,
            fill_type: 1,
        });
    let inverted_plan = analyze_canvaskit_replay_plan(
        &tree_with_ops(vec![PaintOp::tab_leader(bbox(), inverted)]),
        CanvasKitReplayMode::Default,
    );
    assert_eq!(
        inverted_plan.items[0].status,
        CanvasKitReplayStatus::DirectRequired
    );
    assert_eq!(
        inverted_plan.items[0].detail.as_deref(),
        Some("invalidTabLeader")
    );
}

#[test]
fn m07_pack_unknown_text_decoration_shape_is_direct() {
    let mut run = text_run("A");
    run.style.underline_shape = 15;
    run.style.emphasis_dot = 7;
    let tree = tree_with_ops(vec![
        PaintOp::text_decoration(bbox(), run.clone(), TextDecorationKind::Underline),
        PaintOp::text_decoration(bbox(), run, TextDecorationKind::EmphasisDot),
    ]);
    let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
    assert_eq!(plan.summary.direct_items, 2);
    assert_eq!(plan.summary.direct_required_items, 0);
    assert!(plan.items.iter().all(|item| item.detail.is_none()));
}

#[test]
fn m07_pack_visual_item_limit_stays_fail_closed_at_4096() {
    assert_eq!(crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN, 4096);
    let mut at_bound = text_run(&" ".repeat(4096));
    at_bound.char_overlap = Some(CharOverlapInfo {
        border_type: 1,
        inner_char_size: 100,
    });
    let bound_plan = analyze_canvaskit_replay_plan(
        &tree_with_ops(vec![PaintOp::char_overlap(bbox(), at_bound)]),
        CanvasKitReplayMode::Default,
    );
    assert_eq!(bound_plan.items[0].status, CanvasKitReplayStatus::Direct);

    let mut over = text_run(&" ".repeat(4097));
    over.char_overlap = Some(CharOverlapInfo {
        border_type: 1,
        inner_char_size: 100,
    });
    let over_plan = analyze_canvaskit_replay_plan(
        &tree_with_ops(vec![PaintOp::char_overlap(bbox(), over)]),
        CanvasKitReplayMode::Default,
    );
    assert_eq!(
        over_plan.items[0].detail.as_deref(),
        Some("visualItemLimitExceeded")
    );
}

#[test]
fn m07_pack_footnote_marker_stays_direct_with_declared_detail() {
    let tree = tree_with_ops(vec![PaintOp::footnote_marker(
        bbox(),
        FootnoteMarkerNode {
            number: 12,
            text: "12)".to_string(),
            base_font_size: 8.0,
            font_family: "Batang".to_string(),
            color: 0x0000_00ff,
            section_index: 1,
            para_index: 2,
            control_index: 3,
        },
    )]);
    let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
    assert_eq!(plan.items[0].status, CanvasKitReplayStatus::Direct);
    assert_eq!(plan.items[0].detail.as_deref(), Some("footnoteMarker"));
    assert_eq!(
        plan.items[0].feature,
        CanvasKitReplayFeature::TextSpecialVisual
    );
}

#[test]
fn m07_pack_paint_json_emits_line_shadow_and_pattern() {
    let mut line_style = LineStyle::default();
    line_style.end_arrow = ArrowStyle::ConcaveArrow;
    line_style.line_type = LineRenderType::Double;
    line_style.shadow = Some(crate::renderer::ShadowStyle {
        shadow_type: 3,
        color: 0x0000_00aa,
        offset_x: 2.0,
        offset_y: 3.0,
        alpha: 40,
    });
    let mut rect_style = ShapeStyle::default();
    rect_style.pattern = Some(crate::renderer::PatternFillInfo {
        pattern_type: 5,
        pattern_color: 0x0000_00ff,
        background_color: 0x00ff_ffff,
    });
    let tree = tree_with_ops(vec![
        PaintOp::line(bbox(), LineNode::new(0.0, 0.0, 10.0, 0.0, line_style)),
        PaintOp::rectangle(bbox(), RectangleNode::new(0.0, rect_style, None)),
    ]);
    let json = tree.to_json();
    assert!(json.contains("\"lineType\":\"double\""), "{json}");
    assert!(json.contains("\"endArrow\":\"concaveArrow\""), "{json}");
    assert!(json.contains("\"shadowType\":3"), "{json}");
    assert!(json.contains("\"patternType\":5"), "{json}");
}
