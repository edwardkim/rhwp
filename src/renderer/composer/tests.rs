use super::*;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};

/// [#2632] NO_LS 문단에 글자모양이 섞여 있으면 compose_lines fallback 이 만든
/// 단일 run 을 char shape 별로 재분할해야 한다 — 측정이 렌더와 같은 값을 내려면
/// 필수다. 종전엔 본문 경로만 그 역할을 했고, 셀 경로는 `char_shapes[0]` 하나로
/// 문단 전체를 재던 탓에 같은 문단이 경로에 따라 다르게 재였다. 이 테스트는 그
/// **차이를 고정하는 대조군**이었다.
///
/// [#5193] 셀 재조판이 프레임으로 이관되면서 그 차이가 없어졌다. 두 경로가 한
/// 소유자(`recompose_stored_lines_in_frame`)를 공유하고, 그 fill 이
/// `para.char_shapes` 로 토큰화하므로 **셀도 본문과 똑같이 재분할한다**. 대조군의
/// 계약을 "두 경로가 다르다"에서 "두 경로가 같다"로 갱신한다 — 상자만 다르게
/// 말하고(본문 `ParagraphBox::body`, 셀 `ParagraphBox::content_width_px`) 나머지는
/// 한 메커니즘이라는 것이 이관의 내용 그 자체다.
fn the_frame_splits_a_fallback_run_by_char_shapes_on_both_the_body_and_the_cell_route() {
    let para = Paragraph {
        text: "abcdefghij".to_string(),
        char_offsets: (0..10).collect(),
        char_count: 11,
        char_shapes: vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            },
            CharShapeRef {
                start_pos: 5,
                char_shape_id: 1,
            },
        ],
        // line_segs 가 비어 있어 compose_lines 의 CHARS_PER_LINE fallback 경로를 탄다.
        ..Default::default()
    };
    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();
    // 문단 폭 안에 다 들어가도록 충분히 넓게 잡아 줄바꿈 자체는 문제되지 않게 한다.
    let inner_width_px = 2000.0;

    assert_eq!(
        compose_paragraph(&para).lines[0]
            .runs
            .iter()
            .map(|r| r.char_style_id)
            .collect::<Vec<u32>>(),
        vec![0],
        "전제: compose_lines 의 NO_LS fallback 은 char_shapes[0] 단일 run 을 낸다"
    );

    let mut cell_variant = compose_paragraph(&para);
    recompose_cell_lines_in_frame(
        &mut cell_variant,
        &para,
        ParagraphBox::content_width_px(inner_width_px, 96.0),
        &styles,
        96.0,
        false,
    );
    let cell_run_ids: Vec<u32> = cell_variant.lines[0]
        .runs
        .iter()
        .map(|r| r.char_style_id)
        .collect();

    let composed = compose_paragraph(&para);
    let body_variant = recompose_stored_lines_in_frame(
        &composed,
        &para,
        ParagraphBox::content(0..crate::renderer::px_to_hwpunit(inner_width_px, 96.0)),
        inner_width_px,
        &styles,
        96.0,
        false,
        StoredRowMissPolicy::Reflow,
        &[],
    )
    .expect("the frame owns the NO_LS rebuild");
    let body_run_ids: Vec<u32> = body_variant.lines[0]
        .runs
        .iter()
        .map(|r| r.char_style_id)
        .collect();

    assert_eq!(
        body_run_ids,
        vec![0, 1],
        "프레임 fill 은 char_shapes 로 토큰화해 두 글자모양이 드러나야 함"
    );
    assert_eq!(
        cell_run_ids, body_run_ids,
        "[#5193] 셀 경로도 같은 프레임을 쓰므로 본문과 같은 재분할이 나와야 함"
    );
}

fn clean_cell_cache_is_admitted_only_for_its_exact_content_box() {
    let text = "clean cell geometry changes independently of text";
    let mut para = Paragraph {
        text: text.to_string(),
        char_offsets: (0..text.chars().count() as u32).collect(),
        char_count: text.chars().count() as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 1_000,
            text_height: 900,
            baseline_distance: 800,
            segment_width: 1,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }],
        ..Default::default()
    };
    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();
    let dpi = 96.0;
    let measured = estimate_composed_line_width(&compose_paragraph(&para).lines[0], &styles);
    let exact_width = crate::renderer::px_to_hwpunit(measured * 1.2, dpi);
    para.line_segs[0].segment_width = exact_width;
    assert!(!para.stored_text_partition_is_dirty());

    let source = compose_paragraph(&para);
    let mut exact = source.clone();
    recompose_cell_lines_in_frame(
        &mut exact,
        &para,
        ParagraphBox::content(0..exact_width),
        &styles,
        dpi,
        false,
    );
    assert_eq!(exact.lines.len(), 1, "an exact cache hit stays stored");
    assert_eq!(exact.lines[0].segment_width, exact_width);

    let changed_width = crate::renderer::px_to_hwpunit(measured / 1.2, dpi);
    let changed_width_px = crate::renderer::hwpunit_to_px(changed_width, dpi);
    assert!(
        measured < changed_width_px * 1.8,
        "the defensive stale heuristic is not the invalidator"
    );
    let mut changed = source;
    recompose_cell_lines_in_frame(
        &mut changed,
        &para,
        ParagraphBox::content(0..changed_width),
        &styles,
        dpi,
        false,
    );
    assert_eq!(
        changed.lines.len(),
        1,
        "a clean imported cell-box miss is unmodelled without mutation provenance"
    );

    let mut dirty_para = para.clone();
    dirty_para.invalidate_layout_inputs();
    let mut dirty_changed = compose_paragraph(&dirty_para);
    recompose_cell_lines_in_frame(
        &mut dirty_changed,
        &dirty_para,
        ParagraphBox::content(0..changed_width),
        &styles,
        dpi,
        false,
    );
    assert!(
        dirty_changed.lines.len() > 1,
        "a proven cell geometry/text mutation makes the same miss reflowable"
    );
    assert!(dirty_changed
        .lines
        .iter()
        .all(|line| line.segment_width == changed_width));
}

fn synthetic_multirow_cell_compatibility_applies_only_while_clean() {
    let text = "alpha beta gamma delta";
    let mut clean_para = Paragraph {
        text: text.to_string(),
        char_offsets: (0..text.chars().count() as u32).collect(),
        char_count: text.chars().count() as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 1_000,
                segment_width: 10_000,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE | LineSeg::TAG_IMPLEMENTATION_PROPERTY,
                ..Default::default()
            },
            LineSeg {
                text_start: 1,
                vertical_pos: 1_000,
                line_height: 1_000,
                segment_width: 10_000,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE | LineSeg::TAG_IMPLEMENTATION_PROPERTY,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();
    let dpi = 96.0;
    let clean_source = compose_paragraph(&clean_para);
    let mut clean = clean_source.clone();
    recompose_cell_lines_in_frame(
        &mut clean,
        &clean_para,
        ParagraphBox::content(0..10_000),
        &styles,
        dpi,
        false,
    );
    assert_eq!(
        clean
            .lines
            .iter()
            .map(|line| line.char_start)
            .collect::<Vec<_>>(),
        vec![0, 1],
        "clean adapter rows retain their compatibility owner"
    );

    clean_para.insert_text_at(text.chars().count(), " epsilon zeta eta");
    clean_para.invalidate_layout_inputs();
    assert!(clean_para.stored_text_partition_is_dirty());
    let mut dirty = compose_paragraph(&clean_para);
    let measured = dirty
        .lines
        .iter()
        .map(|line| estimate_composed_line_width(line, &styles))
        .sum::<f64>();
    let width_hwp = crate::renderer::px_to_hwpunit(measured / 1.2, dpi);
    recompose_cell_lines_in_frame(
        &mut dirty,
        &clean_para,
        ParagraphBox::content(0..width_hwp),
        &styles,
        dpi,
        false,
    );
    assert!(dirty.lines.len() > 1);
    assert!(
        dirty.lines[1].char_start > 1,
        "dirty synthetic boundaries must not survive the fresh Frame fill"
    );
}

/// A context-expanded field owns only its model span. It must not make the
/// stale NO_LS fallback reclaim the ordinary runs that Frame split from the
/// paragraph's two CharShapeRef ranges.
fn context_display_overlay_preserves_frame_style_partitions_around_the_field() {
    let para = Paragraph {
        text: "abc\u{0017}def".to_string(),
        char_offsets: (0..7).collect(),
        char_count: 8,
        char_shapes: vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            },
            CharShapeRef {
                start_pos: 4,
                char_shape_id: 1,
            },
        ],
        ..Default::default()
    };
    let mut source = compose_paragraph(&para);
    let stale = source.lines[0].runs[0].clone();
    source.lines[0].runs = vec![
        ComposedTextRun {
            text: "abc".to_string(),
            ..stale.clone()
        },
        ComposedTextRun {
            text: "\u{0017}".to_string(),
            display_text: Some("report.hwp".to_string()),
            ..stale.clone()
        },
        ComposedTextRun {
            text: "def".to_string(),
            ..stale
        },
    ];

    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();
    let rebuilt = recompose_stored_lines_in_frame(
        &source,
        &para,
        ParagraphBox::content(0..100_000),
        crate::renderer::hwpunit_to_px(100_000, 96.0),
        &styles,
        96.0,
        false,
        StoredRowMissPolicy::Reflow,
        &[],
    )
    .expect("Frame owns the NO_LS rebuild");

    let runs: Vec<_> = rebuilt.lines[0]
        .runs
        .iter()
        .map(|run| {
            (
                run.text.as_str(),
                run.char_style_id,
                run.display_text.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        runs,
        vec![
            ("abc", 0, None),
            ("\u{0017}", 0, Some("report.hwp")),
            ("def", 1, None),
        ]
    );
}

/// A field fill can invalidate a stored text partition without making its one
/// line grotesquely overfull. Geometry still matches, so mutation provenance
/// must defeat cache admission before the defensive 1.8x import heuristic.
fn modest_field_fill_reflows_a_geometry_matching_stored_partition() {
    let initial = "old";
    let mut para = Paragraph {
        text: initial.to_string(),
        char_offsets: (0..initial.chars().count() as u32).collect(),
        char_count: initial.chars().count() as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 1_000,
            text_height: 900,
            baseline_distance: 800,
            segment_width: 1,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }],
        controls: vec![crate::model::control::Control::Field(
            crate::model::control::Field::default(),
        )],
        ..Default::default()
    };

    para.delete_text_at(0, initial.chars().count());
    para.insert_text_at(0, "moderately wider field value");
    para.invalidate_layout_inputs();
    assert!(para.stored_text_partition_is_dirty());

    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();
    let dpi = 96.0;
    let composed = compose_paragraph(&para);
    let measured = estimate_composed_line_width(&composed.lines[0], &styles);
    let width_hwp = crate::renderer::px_to_hwpunit(measured / 1.2, dpi);
    let inner_width = crate::renderer::hwpunit_to_px(width_hwp, dpi);
    para.line_segs[0].segment_width = width_hwp;
    assert!(measured > inner_width);
    assert!(
        measured < inner_width * 1.8,
        "the heuristic must stay false"
    );

    let mut clean_probe = para.clone();
    clean_probe.replace_line_segs(clean_probe.line_segs.clone());
    let clean_composed = compose_paragraph(&clean_probe);
    assert!(!stored_rows_are_stale(
        &clean_composed,
        &clean_probe,
        inner_width,
        &styles
    ));
    let mut clean_frame = ParagraphBox::content(0..width_hwp).frame(0);
    assert!(matches!(
        line_breaking::resolve_stored_line_segs_in_frame(
            &clean_probe,
            &mut clean_frame,
            &styles,
            dpi,
            false,
            StoredRowMissPolicy::Reflow,
            false,
            &[],
            false,
        ),
        Some(line_breaking::StoredRowResolution::Stored)
    ));

    let stale = stored_rows_are_stale(&composed, &para, inner_width, &styles);
    assert!(
        stale,
        "the mutation bit, not overflow magnitude, is decisive"
    );
    let mut dirty_frame = ParagraphBox::content(0..width_hwp).frame(0);
    assert!(matches!(
        line_breaking::resolve_stored_line_segs_in_frame(
            &para,
            &mut dirty_frame,
            &styles,
            dpi,
            false,
            StoredRowMissPolicy::Reflow,
            stale,
            &[],
            false,
        ),
        Some(line_breaking::StoredRowResolution::Reflowed)
    ));
    let rebuilt = dirty_frame.project_line_segs();
    assert!(rebuilt.len() > 1, "the field value needs a new boundary");
    assert!(rebuilt[1].text_start > 0);

    let mut materialized = para;
    reflow_line_segs(
        &mut materialized,
        ParagraphBox::content(0..width_hwp),
        &styles,
        dpi,
    );
    assert!(
        !materialized.stored_text_partition_is_dirty(),
        "a model-writing reflow publishes a current partition"
    );
}

/// 누름틀을 가진 본문 문단도 프레임이 소유한다 — 소유자 없는 문단을 만들면 안 된다.
///
/// `hwp_doc_fill_fields`/`edit fill-fields` 는 필드의 텍스트만 바꾸고
/// `reflow_line_segs` 를 부르지 않는다(`queries/field_query.rs` 의
/// `set_field_text_at`). 그래서 채움 뒤 문단은 "빈 서식 한 줄"을 설명하던 저장
/// LINE_SEG 하나를 그대로 들고 수천 자를 담게 된다. 그 줄을 다시 나눌 마지막
/// 관문이 프레임이므로, 필드가 있다는 이유로 프레임이 물러나면 그 문단에는
/// **소유자가 아무도 없다** — samples/field-01.hwp 채움본이 5,109바이트를 한 줄로
/// 그리고 쪽수가 3에 머문 원인이다(수정 후 10쪽).
fn a_field_paragraph_whose_stored_row_cannot_hold_its_text_is_rewrapped() {
    let text = "가나다라마바사아자차".repeat(60);
    let char_count = text.chars().count();
    let para = Paragraph {
        text,
        char_offsets: (0..char_count as u32).collect(),
        char_count: char_count as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        // 채움이 남긴 상태: 저장 줄은 하나, 태그는 정품(구현속성 아님).
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 1000,
            text_height: 1000,
            baseline_distance: 850,
            segment_width: 40_000,
            tag: LineSeg::TAG_FIRST_SEGMENT | LineSeg::TAG_LAST_SEGMENT,
            ..Default::default()
        }],
        controls: vec![crate::model::control::Control::Field(
            crate::model::control::Field::default(),
        )],
        ..Default::default()
    };
    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();
    let inner_width_px = crate::renderer::hwpunit_to_px(40_000, 96.0);

    let composed = compose_paragraph(&para);
    assert_eq!(
        composed.lines.len(),
        1,
        "전제: 저장 줄이 하나라 조합도 한 줄이다"
    );

    let rewrapped = recompose_stored_lines_in_frame(
        &composed,
        &para,
        ParagraphBox::content(0..40_000),
        inner_width_px,
        &styles,
        96.0,
        false,
        StoredRowMissPolicy::Reflow,
        &[],
    )
    .expect("필드를 가진 본문 문단도 프레임이 소유한다");
    assert!(
        rewrapped.lines.len() > 1,
        "내폭을 크게 넘긴 저장 줄은 다시 나뉘어야 한다 (줄수 {})",
        rewrapped.lines.len()
    );
}

/// A paragraph whose margins exceed its column has no box, and **no route may
/// publish a record for it**.
///
/// `margin_left = 80px` against a `100px` column with `margin_right = 40px` is
/// legal input — a large left indent inside a narrow column, or a multi-column
/// layout whose spacing eats the body. The box is `6000..4500` HWPUNIT, i.e.
/// `width_hwp() == -1500`.
///
/// The edit path used to carry that straight into `make_line_seg` and publish
/// `segment_width = -1500` on every row, and `segment_width` goes to disk —
/// `serializer::body_text` writes it with `write_i32` and the HWPX
/// `linesegarray` takes it raw — so the corrupt extent reached the file. A
/// self-roundtrip does not necessarily catch it either: `hwpx::roundtrip`
/// compares `horzsize` as `i64`, so `-1500` round-trips to `-1500` and matches.
///
/// This pins the refusal, not a floor. If someone restores a
/// `.max(1.0)`-style clamp the stored record survives but `line_segs` gains a
/// fabricated width, and the first assertion fails.
fn an_impossible_paragraph_box_publishes_nothing_on_either_route() {
    let stored = LineSeg {
        text_start: 0,
        line_height: 1000,
        text_height: 1000,
        baseline_distance: 850,
        column_start: 0,
        segment_width: 40_000,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
        ..Default::default()
    };
    let para = Paragraph {
        text: "가나다라마바사".to_string(),
        char_offsets: (0..7).collect(),
        char_count: 8,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![stored.clone()],
        ..Default::default()
    };
    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();

    let impossible = ParagraphBox::body(100.0, 80.0, 40.0, 96.0);
    assert!(
        impossible.width_hwp() < 0,
        "전제: 여백이 열을 넘으면 상자 폭이 음수다 ({})",
        impossible.width_hwp()
    );
    assert!(!impossible.is_usable());

    // Edit path: the stored record is left exactly as it was.
    let mut edited = para.clone();
    reflow_line_segs(&mut edited, impossible.clone(), &styles, 96.0);
    let extents = |segs: &[LineSeg]| {
        segs.iter()
            .map(|seg| (seg.text_start, seg.column_start, seg.segment_width))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        extents(&edited.line_segs),
        extents(std::slice::from_ref(&stored)),
        "불가능한 상자는 저장 기록을 그대로 두어야 한다 — 음수 폭도, 날조된 폭도 발행 금지"
    );

    // Render path: no recomposition, so the composition stands.
    let composed = compose_paragraph(&para);
    assert!(
        recompose_stored_lines_in_frame(
            &composed,
            &para,
            impossible,
            0.0,
            &styles,
            96.0,
            false,
            StoredRowMissPolicy::Reflow,
            &[],
        )
        .is_none(),
        "렌더 경로도 같은 규칙을 따른다"
    );
}

/// 단일 줄, 단일 스타일 문단
#[test]
fn test_compose_single_line_single_style() {
    the_frame_splits_a_fallback_run_by_char_shapes_on_both_the_body_and_the_cell_route();
    clean_cell_cache_is_admitted_only_for_its_exact_content_box();
    synthetic_multirow_cell_compatibility_applies_only_while_clean();
    context_display_overlay_preserves_frame_style_partitions_around_the_field();
    modest_field_fill_reflows_a_geometry_matching_stored_partition();
    a_field_paragraph_whose_stored_row_cannot_hold_its_text_is_rewrapped();
    an_impossible_paragraph_box_publishes_nothing_on_either_route();
    let para = Paragraph {
        text: "안녕하세요".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_count: 6, // 5 chars + 1 (paragraph end)
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 3,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 800,
            baseline_distance: 640,
            ..Default::default()
        }],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 1);
    assert_eq!(composed.lines[0].runs.len(), 1);
    assert_eq!(composed.lines[0].runs[0].text, "안녕하세요");
    assert_eq!(composed.lines[0].runs[0].char_style_id, 3);
}

/// 단일 줄, 다중 스타일
#[test]
fn test_compose_single_line_multi_style() {
    let para = Paragraph {
        text: "ABCDE".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_count: 6,
        char_shapes: vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 1,
            },
            CharShapeRef {
                start_pos: 3,
                char_shape_id: 2,
            },
        ],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 400,
            baseline_distance: 320,
            ..Default::default()
        }],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 1);
    assert_eq!(composed.lines[0].runs.len(), 2);
    assert_eq!(composed.lines[0].runs[0].text, "ABC");
    assert_eq!(composed.lines[0].runs[0].char_style_id, 1);
    assert_eq!(composed.lines[0].runs[1].text, "DE");
    assert_eq!(composed.lines[0].runs[1].char_style_id, 2);
}

/// 다중 줄 문단
#[test]
fn test_compose_multi_line() {
    let para = Paragraph {
        text: "첫줄텍스트두번째줄".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        char_count: 10,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 5,
        }],
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            },
            LineSeg {
                text_start: 5,
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 2);
    assert_eq!(composed.lines[0].runs[0].text, "첫줄텍스트");
    assert_eq!(composed.lines[1].runs[0].text, "두번째줄");
}

/// 단일 LINE_SEG 안의 Shift+Enter 강제 줄바꿈도 실제 visual line 으로 분리한다.
#[test]
fn test_compose_internal_forced_line_break_splits_visual_lines() {
    let para = Paragraph {
        text: "가나\n다라".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_count: 6,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 7,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 400,
            baseline_distance: 320,
            ..Default::default()
        }],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 2);
    assert_eq!(composed.lines[0].runs[0].text, "가나");
    assert!(composed.lines[0].has_line_break);
    assert_eq!(composed.lines[0].char_start, 0);
    assert_eq!(composed.lines[1].runs[0].text, "다라");
    assert!(!composed.lines[1].has_line_break);
    assert_eq!(composed.lines[1].char_start, 3);
}

/// 끝의 Shift+Enter는 줄바꿈 표시 줄만 만들고 빈 후속 줄을 중복 생성하지 않는다.
#[test]
fn test_compose_trailing_forced_line_break_keeps_single_marked_line() {
    let para = Paragraph {
        text: "가나\n".to_string(),
        char_offsets: vec![0, 1, 2],
        char_count: 4,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 7,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 400,
            baseline_distance: 320,
            ..Default::default()
        }],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 1);
    assert_eq!(composed.lines[0].runs[0].text, "가나");
    assert!(composed.lines[0].has_line_break);
    assert_eq!(composed.lines[0].char_start, 0);
}

/// 다중 줄 + 다중 스타일 (줄 경계에서 스타일 변경)
#[test]
fn test_compose_multi_line_multi_style() {
    let para = Paragraph {
        text: "AAABBBCCCC".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        char_count: 11,
        char_shapes: vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 1,
            },
            CharShapeRef {
                start_pos: 3,
                char_shape_id: 2,
            },
            CharShapeRef {
                start_pos: 6,
                char_shape_id: 3,
            },
        ],
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            },
            LineSeg {
                text_start: 6,
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 2);

    // 첫 줄: "AAA" (style 1) + "BBB" (style 2)
    assert_eq!(composed.lines[0].runs.len(), 2);
    assert_eq!(composed.lines[0].runs[0].text, "AAA");
    assert_eq!(composed.lines[0].runs[0].char_style_id, 1);
    assert_eq!(composed.lines[0].runs[1].text, "BBB");
    assert_eq!(composed.lines[0].runs[1].char_style_id, 2);

    // 두번째 줄: "CCCC" (style 3)
    assert_eq!(composed.lines[1].runs.len(), 1);
    assert_eq!(composed.lines[1].runs[0].text, "CCCC");
    assert_eq!(composed.lines[1].runs[0].char_style_id, 3);
}

/// 빈 문단
#[test]
fn test_compose_empty_paragraph() {
    let para = Paragraph::default();
    let composed = compose_paragraph(&para);
    assert!(composed.lines.is_empty());
    assert!(composed.inline_controls.is_empty());
}

/// table-vpos-01 page 5의 10/11/12 마커는 CharOverlap 하나에 두 개의
/// HWP PUA 구성 글자가 들어온다. 텍스트 흐름과 캐럿 이동은 한 글자 폭이어야 한다.
#[test]
fn test_char_overlap_multi_component_is_single_advance() {
    let chars = vec!['\u{F02BA}', '\u{F02C3}'];
    assert_eq!(decode_pua_overlap_number(&chars), None);
    assert_eq!(char_overlap_advance_units(&chars), 1);
}

/// [#4085] `charSz` 는 OWPML 상 "테두리 내부 글자의 크기 비율"이므로 테두리를
/// 그리지 않는 겹침에는 적용하지 않는다.
///
/// 한컴 실측 두 건이 이 규칙을 함께 만족한다:
/// - k-water-rfp p13 — 반전 사각형(4) + `charSz=-2` → 0.80 (PR #1101 시각 검증)
/// - 관세청 월간 수출입 현황 p1 — 테두리 없음(0) + `charSz=-4` → 축소 없음.
///   한컴 PDF content stream 에서 마커와 본문이 같은 `101 Tf`, 같은 baseline.
#[test]
fn char_overlap_size_ratio_applies_only_when_a_border_is_drawn() {
    // 테두리 없음 — charSz 부호와 무관하게 축소하지 않는다 (#4085 관세청 오라클)
    assert_eq!(char_overlap_size_ratio(0, -4), 1.0);
    assert_eq!(char_overlap_size_ratio(0, 90), 1.0);

    // 테두리 있음 — PR #1101 실측 규칙 보존 (회귀 금지)
    assert_eq!(char_overlap_size_ratio(4, -2), 0.8);
    assert!((char_overlap_size_ratio(1, -3) - 0.7).abs() < 1e-9);

    // 양수는 percent 그대로
    assert_eq!(char_overlap_size_ratio(1, 50), 0.5);

    // 0 은 기본 100%
    assert_eq!(char_overlap_size_ratio(3, 0), 1.0);
}

/// LineSeg 없는 텍스트 문단
#[test]
fn test_compose_no_line_segs() {
    let para = Paragraph {
        text: "텍스트만 있음".to_string(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 7,
        }],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 1);
    assert_eq!(composed.lines[0].runs[0].text, "텍스트만 있음");
    assert_eq!(composed.lines[0].runs[0].char_style_id, 7);
}

/// 확장 컨트롤 문자로 인한 위치 격차
#[test]
fn test_compose_with_ctrl_char_gap() {
    // 원본 UTF-16: [ctrl 8units][A][B][C]
    // text = "ABC"
    // char_offsets = [8, 9, 10]
    // LineSeg.text_start = 0 (첫 줄은 처음부터)
    let para = Paragraph {
        text: "ABC".to_string(),
        char_offsets: vec![8, 9, 10],
        char_count: 12,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 1,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 400,
            baseline_distance: 320,
            ..Default::default()
        }],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 1);
    assert_eq!(composed.lines[0].runs[0].text, "ABC");
    assert_eq!(composed.lines[0].runs[0].char_style_id, 1);
}

/// 인라인 컨트롤 식별
#[test]
fn test_identify_inline_controls_table() {
    use crate::model::table::Table;

    let mut table = Table::default();
    table.common.treat_as_char = true;
    let para = Paragraph {
        text: "표 앞 텍스트".to_string(),
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.inline_controls.len(), 1);
    assert_eq!(
        composed.inline_controls[0].control_type,
        InlineControlType::Table
    );
    assert_eq!(composed.inline_controls[0].control_index, 0);
}

/// UTF-16 위치 → 텍스트 인덱스 변환
#[test]
fn test_utf16_range_to_text_range() {
    let offsets = vec![0u32, 1, 2, 8, 9, 10]; // 위치 3~7은 확장 컨트롤

    let (s, e) = utf16_range_to_text_range(&offsets, 0, 3, 6);
    assert_eq!(s, 0);
    assert_eq!(e, 3); // offsets[3]=8 >= 3 이므로 인덱스 3

    let (s, e) = utf16_range_to_text_range(&offsets, 8, 11, 6);
    assert_eq!(s, 3);
    assert_eq!(e, 6);
}

/// 오프셋 없는 경우 1:1 매핑
#[test]
fn test_utf16_range_no_offsets() {
    let (s, e) = utf16_range_to_text_range(&[], 0, 5, 10);
    assert_eq!(s, 0);
    assert_eq!(e, 5);
}

#[test]
fn test_compose_decreasing_lineseg_text_start_uses_empty_range() {
    let para = Paragraph {
        text: "ABCDE".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_count: 6,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 1,
        }],
        line_segs: vec![
            LineSeg {
                text_start: 4,
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            },
            LineSeg {
                text_start: 0,
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 2);
    assert!(composed.lines[0].runs.is_empty());
    assert_eq!(composed.lines[0].char_start, 4);
    assert_eq!(composed.lines[1].runs[0].text, "ABCDE");
}

/// find_active_char_shape 테스트
#[test]
fn test_find_active_char_shape() {
    let shapes = vec![
        CharShapeRef {
            start_pos: 0,
            char_shape_id: 1,
        },
        CharShapeRef {
            start_pos: 10,
            char_shape_id: 2,
        },
        CharShapeRef {
            start_pos: 20,
            char_shape_id: 3,
        },
    ];

    assert_eq!(find_active_char_shape(&shapes, 0), 1);
    assert_eq!(find_active_char_shape(&shapes, 5), 1);
    assert_eq!(find_active_char_shape(&shapes, 10), 2);
    assert_eq!(find_active_char_shape(&shapes, 15), 2);
    assert_eq!(find_active_char_shape(&shapes, 25), 3);
}

// === reflow_line_segs 테스트 ===

fn make_styles_with_font_size(font_size: f64) -> ResolvedStyleSet {
    use crate::renderer::style_resolver::{ResolvedCharStyle, ResolvedParaStyle, ResolvedStyleSet};
    ResolvedStyleSet {
        hwp3_variant: false,
        char_styles: vec![ResolvedCharStyle {
            font_size,
            ratio: 1.0,
            ..Default::default()
        }],
        para_styles: vec![ResolvedParaStyle::default()],
        ..Default::default()
    }
}

/// 짧은 텍스트 → 1줄
#[test]
fn test_reflow_short_text_single_line() {
    let styles = make_styles_with_font_size(16.0);
    let mut para = Paragraph {
        text: "안녕".to_string(),
        char_offsets: vec![0, 1],
        char_count: 3,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    // 컬럼 너비 500px → "안녕" (16*2=32px) 충분히 들어감
    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(500.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 1);
    assert_eq!(para.line_segs[0].text_start, 0);
}

/// 긴 텍스트 → 2줄 이상
#[test]
fn test_reflow_long_text_multi_line() {
    let styles = make_styles_with_font_size(16.0);
    // CJK 10글자: 각 16px → 총 160px
    let mut para = Paragraph {
        text: "가나다라마바사아자차".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        char_count: 11,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    // 컬럼 너비 80px → 16px * 5글자 = 80px → 5글자씩 2줄
    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(80.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 2);
    assert_eq!(para.line_segs[0].text_start, 0);
    assert_eq!(para.line_segs[1].text_start, 5); // 6번째 글자부터 2번째 줄
}

/// 빈 텍스트 → 기본 LineSeg 1개
#[test]
fn test_reflow_empty_text() {
    let styles = make_styles_with_font_size(16.0);
    let mut para = Paragraph::default();

    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(500.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 1);
    assert_eq!(para.line_segs[0].text_start, 0);
}

/// [#4677] 인라인 개체만 있는 문단의 둘째 줄 `text_start` 는 **UTF-16 오프셋**이다.
///
/// HWP5 PARA_TEXT 에서 확장 제어문자 하나는 8 코드유닛을 차지한다. 컨트롤 인덱스(=1)를
/// 그대로 쓰면 둘째 줄이 첫 제어문자 블록 한가운데를 가리키고, 그 저장본을 한글 2022 는
/// 본문을 통째로 버린 빈 1쪽 문서로 연다 (rhwp 재파싱은 통과하는 함정).
#[test]
fn test_reflow_inline_only_paragraph_uses_utf16_text_start() {
    use crate::model::image::Picture;
    use crate::model::shape::CommonObjAttr;

    let styles = make_styles_with_font_size(16.0);
    let make_pic = |width: u32| {
        Control::Picture(Box::new(Picture {
            common: CommonObjAttr {
                width,
                height: 3000,
                treat_as_char: true,
                ..Default::default()
            },
            ..Default::default()
        }))
    };
    // 가용 폭 50px = 3750 HWPUNIT → 3000 짜리 개체 둘은 한 줄에 못 들어간다.
    let mut para = Paragraph {
        controls: vec![make_pic(3000), make_pic(3000)],
        char_count: 17, // 8 + 8 + 문단 끝 1
        ..Default::default()
    };

    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(50.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 2, "개체마다 한 줄");
    assert_eq!(para.line_segs[0].text_start, 0);
    assert_eq!(
        para.line_segs[1].text_start, 8,
        "둘째 개체의 UTF-16 오프셋은 8 (컨트롤 인덱스 1 이 아니다)"
    );
}

/// 라틴 문자 리플로우 (0.5 * font_size)
#[test]
fn test_reflow_latin_text() {
    let styles = make_styles_with_font_size(16.0);
    // 라틴 10글자: 각 8px → 총 80px
    let mut para = Paragraph {
        text: "ABCDEFGHIJ".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        char_count: 11,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    // 컬럼 너비 40px → 8px * 5글자 = 40px → 5글자씩 2줄
    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(40.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 2);
    assert_eq!(para.line_segs[0].text_start, 0);
    assert_eq!(para.line_segs[1].text_start, 5);
}

/// line_height가 올바르게 설정되는지 검증
#[test]
fn test_reflow_line_height() {
    let styles = make_styles_with_font_size(16.0);
    let mut para = Paragraph {
        text: "가".to_string(),
        char_offsets: vec![0],
        char_count: 2,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(500.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 1);
    // line_height = px_to_hwpunit(16.0, 96) = (16.0 * 7200 / 96) = 1200
    // HWP LineSeg.line_height = 폰트 크기 (실증: 10pt→1000, 12pt→1200)
    assert_eq!(para.line_segs[0].line_height, 1200);
}

// ===== split_runs_by_lang 테스트 =====

/// 한영 혼합 텍스트가 언어별로 분할되는지 검증
#[test]
fn test_split_runs_by_lang_korean_english() {
    let runs = vec![ComposedTextRun {
        text: "안녕Hello세계".to_string(),
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: None,
    }];
    let result = split_runs_by_lang(runs);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].text, "안녕");
    assert_eq!(result[0].lang_index, 0); // 한국어
    assert_eq!(result[1].text, "Hello");
    assert_eq!(result[1].lang_index, 1); // 영어
    assert_eq!(result[2].text, "세계");
    assert_eq!(result[2].lang_index, 0); // 한국어
}

/// 단일 언어 텍스트는 분할 없음
#[test]
fn test_split_runs_by_lang_no_split() {
    let runs = vec![ComposedTextRun {
        text: "안녕하세요".to_string(),
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: None,
    }];
    let result = split_runs_by_lang(runs);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "안녕하세요");
    assert_eq!(result[0].lang_index, 0);
}

/// 공백은 이전 문자의 언어를 따름 (불필요한 분할 방지)
#[test]
fn test_split_runs_by_lang_space_follows_prev() {
    let runs = vec![ComposedTextRun {
        text: "안녕 Hello 세계".to_string(),
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: None,
    }];
    let result = split_runs_by_lang(runs);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].text, "안녕 ");
    assert_eq!(result[0].lang_index, 0); // 한국어 + 공백
    assert_eq!(result[1].text, "Hello ");
    assert_eq!(result[1].lang_index, 1); // 영어 + 공백
    assert_eq!(result[2].text, "세계");
    assert_eq!(result[2].lang_index, 0); // 한국어
}

/// 빈 텍스트 run은 그대로 유지
#[test]
fn test_split_runs_by_lang_empty() {
    let runs = vec![ComposedTextRun {
        text: "".to_string(),
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: None,
    }];
    let result = split_runs_by_lang(runs);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "");
}

/// 영어만 있는 텍스트
#[test]
fn test_split_runs_by_lang_english_only() {
    let runs = vec![ComposedTextRun {
        text: "Hello World".to_string(),
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: None,
    }];
    let result = split_runs_by_lang(runs);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Hello World");
    assert_eq!(result[0].lang_index, 1); // 영어
}

/// is_lang_neutral 검증
#[test]
fn test_is_lang_neutral() {
    assert!(is_lang_neutral(' '));
    assert!(is_lang_neutral('.'));
    assert!(is_lang_neutral(','));
    assert!(is_lang_neutral('!'));
    assert!(is_lang_neutral('('));
    assert!(!is_lang_neutral('A'));
    assert!(!is_lang_neutral('가'));
    assert!(!is_lang_neutral('漢'));
}

/// 언어 인식 리플로우: 한국어+영어 혼합 문단
#[test]
fn test_reflow_lang_aware_mixed() {
    use crate::renderer::style_resolver::{ResolvedCharStyle, ResolvedParaStyle, ResolvedStyleSet};

    let styles = ResolvedStyleSet {
        hwp3_variant: false,
        char_styles: vec![ResolvedCharStyle {
            font_family: "함초롬돋움".to_string(),
            font_families: vec![
                "함초롬돋움".to_string(), // 한국어
                "Arial".to_string(),      // 영어
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ],
            font_size: 16.0,
            ratio: 1.0,
            ratios: vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            letter_spacing: 0.0,
            letter_spacings: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ..Default::default()
        }],
        para_styles: vec![ResolvedParaStyle::default()],
        ..Default::default()
    };

    // 한영 혼합 텍스트 (충분히 좁은 너비 → 여러 줄)
    let mut para = Paragraph {
        text: "가나다ABC".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5],
        char_count: 7,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    // 너비 충분 → 1줄
    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(500.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 1);

    // 너비 부족 → 여러 줄 (언어별 폰트 적용 확인)
    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(30.0, 96.0),
        &styles,
        96.0,
    );
    assert!(
        para.line_segs.len() > 1,
        "좁은 너비에서 줄 바꿈이 발생해야 함"
    );
}

/// estimate_composed_line_width 기본 테스트
#[test]
fn test_estimate_composed_line_width() {
    let styles = make_styles_with_font_size(16.0);

    let line = ComposedLine {
        runs: vec![ComposedTextRun {
            text: "가나다".to_string(),
            char_style_id: 0,
            lang_index: 0,
            char_overlap: None,
            footnote_marker: None,
            display_text: None,
        }],
        line_height: 400,
        baseline_distance: 320,
        segment_width: 0,
        column_start: 0,
        line_spacing: 0,
        has_line_break: false,
        char_start: 0,
    };

    let width = estimate_composed_line_width(&line, &styles);
    assert!(width > 0.0, "폭이 0보다 커야 함");
}

// === 줄 나눔 엔진 테스트 ===

/// 한국어 어절 줄 바꿈: 공백에서 줄 바꿈
#[test]
fn test_reflow_korean_eojeol_wrap() {
    let styles = make_styles_with_font_size(16.0);
    // "안녕하세요 반갑습니다" — 5글자 + 공백 + 5글자
    // 각 16px, 공백 8px → 총 5*16 + 8 + 5*16 = 168px
    let mut para = Paragraph {
        text: "안녕하세요 반갑습니다".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        char_count: 12,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    // 너비 100px → "안녕하세요" (80px) + " " (8px) = 88px 들어감
    // "반갑습니다" (80px) → 2번째 줄
    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(100.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 2, "어절 경계에서 줄 바꿈");
    assert_eq!(para.line_segs[0].text_start, 0);
    // 두 번째 줄은 공백 다음 글자부터 (char_offset 6)
    assert_eq!(para.line_segs[1].text_start, 6);
}

/// 한글 줄 나눔 단위 계약: 0=어절, 1=글자
#[test]
fn test_reflow_korean_break_unit_contract() {
    let mut word_styles = make_styles_with_font_size(16.0);
    word_styles.para_styles[0].korean_break_unit = 0;

    let mut char_styles = make_styles_with_font_size(16.0);
    char_styles.para_styles[0].korean_break_unit = 1;

    let make_para = || Paragraph {
        text: "가나 다라".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_count: 6,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut word_para = make_para();
    reflow_line_segs(
        &mut word_para,
        ParagraphBox::content_width_px(60.0, 96.0),
        &word_styles,
        96.0,
    );

    let mut char_para = make_para();
    reflow_line_segs(
        &mut char_para,
        ParagraphBox::content_width_px(60.0, 96.0),
        &char_styles,
        96.0,
    );

    let word_starts: Vec<u32> = word_para
        .line_segs
        .iter()
        .map(|seg| seg.text_start)
        .collect();
    let char_starts: Vec<u32> = char_para
        .line_segs
        .iter()
        .map(|seg| seg.text_start)
        .collect();

    assert_eq!(word_starts, vec![0, 3], "어절 모드는 공백 뒤에서 줄바꿈");
    assert_eq!(char_starts, vec![0, 4], "글자 모드는 다음 어절 일부를 채움");
}

/// 영어 단어 줄 바꿈: 공백에서 줄 바꿈
#[test]
fn test_reflow_english_word_wrap() {
    let styles = make_styles_with_font_size(16.0);
    // "Hello World" — 각 8px (Latin=0.5*16), 공백 8px
    // "Hello" (40px) + " " (8px) + "World" (40px) = 88px
    let mut para = Paragraph {
        text: "Hello World".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        char_count: 12,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    // 너비 60px → "Hello" (40px) + " " (8px) = 48px 들어감
    // "World" (40px) → 2번째 줄
    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(60.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 2, "단어 경계에서 줄 바꿈");
    assert_eq!(para.line_segs[0].text_start, 0);
    assert_eq!(para.line_segs[1].text_start, 6); // "World" 시작
}

#[test]
fn issue_4442_corrected_noto_ascii_advances_change_threshold_wrap_with_kerning_off() {
    use crate::renderer::layout::{estimate_text_width_unrounded, resolved_to_text_style};
    use crate::renderer::style_resolver::ResolvedCharStyle;

    let mut styles = make_styles_with_font_size(1000.0);
    styles.char_styles[0] = ResolvedCharStyle {
        font_family: "Noto Sans KR".to_string(),
        font_size: 1000.0,
        kerning: false,
        ..Default::default()
    };
    let text_style = resolved_to_text_style(&styles, 0, 1);
    let corrected_width = estimate_text_width_unrounded("AVATAR", &text_style);
    let prior_table_width = 3416.0;
    assert_eq!(corrected_width, 3633.0);
    let threshold = (prior_table_width + corrected_width) / 2.0;

    let mut para = Paragraph {
        text: "AVATAR".to_string(),
        char_offsets: (0..6).collect(),
        char_count: 7,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };
    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(threshold, 96.0),
        &styles,
        96.0,
    );

    assert_eq!(
        para.line_segs
            .iter()
            .map(|line| line.text_start)
            .collect::<Vec<_>>(),
        vec![0, 5]
    );
}

fn reflow_after_prior_break_line_starts(text: &str, indent_px: f64) -> Vec<u32> {
    let mut styles = make_styles_with_font_size(16.0);
    styles.para_styles[0].indent = indent_px;
    let mut utf16_len = 0u32;
    let char_offsets = text
        .chars()
        .map(|ch| {
            let offset = utf16_len;
            utf16_len += ch.len_utf16() as u32;
            offset
        })
        .collect();
    let mut para = Paragraph {
        text: text.to_string(),
        char_offsets,
        char_count: utf16_len + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(40.0, 96.0),
        &styles,
        96.0,
    );
    para.line_segs.iter().map(|seg| seg.text_start).collect()
}

#[test]
fn issue_3822_reflow_overlong_latin_token_after_prior_break() {
    assert_eq!(
        reflow_after_prior_break_line_starts("가 ABCDEFGHIJKL", 0.0),
        vec![0, 2, 7, 12],
        "이전 공백 뒤 긴 Latin 토큰도 새 줄 폭을 넘을 때 계속 글자 단위로 분할해야 함"
    );
}

#[test]
fn issue_3822_reflow_overlong_korean_word_after_prior_break() {
    assert_eq!(
        reflow_after_prior_break_line_starts("A 가나다라마바사", 0.0),
        vec![0, 2, 4, 6, 8],
        "이전 공백 뒤 긴 한글 어절도 새 줄 폭을 넘을 때 계속 글자 단위로 분할해야 함"
    );
}

#[test]
fn issue_3822_reflow_overlong_digit_token_after_prior_break() {
    assert_eq!(
        reflow_after_prior_break_line_starts("A 123456789012", 0.0),
        vec![0, 2, 7, 12],
        "이전 공백 뒤 긴 숫자 토큰도 새 줄 폭을 넘을 때 계속 글자 단위로 분할해야 함"
    );
}

#[test]
fn issue_3822_reflow_overlong_digit_preserves_nonempty_post_break_width() {
    assert_eq!(
        reflow_after_prior_break_line_starts("A 가.123456789012", 0.0),
        vec![0, 2, 6, 11],
        "이전 break 뒤 잔여 글자 폭을 보존한 상태에서 긴 숫자를 분할해야 함"
    );
}

#[test]
fn issue_3822_reflow_overlong_token_uses_subsequent_line_indent_width() {
    assert_eq!(
        reflow_after_prior_break_line_starts("A ABCDEFGHIJKL", -8.0),
        vec![0, 2, 6, 10],
        "첫 줄 뒤에는 hanging indent가 적용된 후속 줄 폭으로 긴 토큰을 분할해야 함"
    );
}

#[test]
fn test_reflow_condense_shrinks_measured_space_width() {
    let mut styles = make_styles_with_font_size(10.0);
    styles.para_styles[0].condense_min_space = 20;

    let mut para = Paragraph {
        text: "A B ABCDEF".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        char_count: 10,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    // Natural width is 50px: 8 latin chars at 5px + 2 spaces at 5px.
    // condense=20 allows each measured space to shrink by 20%, saving 2px.
    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(48.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 1);
}

/// 강제 줄 바꿈: \n에서 즉시 줄 바꿈
#[test]
fn test_reflow_forced_line_break() {
    let styles = make_styles_with_font_size(16.0);
    let mut para = Paragraph {
        text: "가나\n다라".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_count: 6,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        ..Default::default()
    };

    reflow_line_segs(
        &mut para,
        ParagraphBox::content_width_px(500.0, 96.0),
        &styles,
        96.0,
    );
    assert_eq!(para.line_segs.len(), 2, "\\n에서 강제 줄 바꿈");
    assert_eq!(para.line_segs[0].text_start, 0);
    assert_eq!(para.line_segs[1].text_start, 3); // \n 다음
}

/// 금칙 처리: 줄 머리/꼬리 금칙 검증
#[test]
fn test_geumchik_functions() {
    // 줄 머리 금칙: 줄 시작에 올 수 없는 문자
    assert!(is_line_start_forbidden(')'));
    assert!(is_line_start_forbidden('.'));
    assert!(is_line_start_forbidden(','));
    assert!(is_line_start_forbidden('!'));
    assert!(is_line_start_forbidden('%'));
    assert!(!is_line_start_forbidden('가'));
    assert!(!is_line_start_forbidden('A'));

    // 줄 꼬리 금칙: 줄 끝에 올 수 없는 문자
    assert!(is_line_end_forbidden('('));
    assert!(is_line_end_forbidden('['));
    assert!(is_line_end_forbidden('$'));
    assert!(is_line_end_forbidden('\u{20A9}')); // ₩
    assert!(!is_line_end_forbidden('가'));
    assert!(!is_line_end_forbidden('A'));
}

/// 토크나이저: 한국어 어절 토큰화
#[test]
fn test_tokenize_korean_eojeol() {
    let styles = make_styles_with_font_size(16.0);
    let text: Vec<char> = "가나 다라".chars().collect();
    let offsets: Vec<u32> = (0..text.len() as u32).collect();
    let shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];

    // [#2185] bit7=0 = 어절 단위 (한컴 통제 실측 3중 확증 — 종전 ==1 역해석 정정)
    let tokens = tokenize_paragraph(&text, &offsets, &shapes, &styles, 0, 0);
    // "가나" (Text) + " " (Space) + "다라" (Text) = 3 tokens
    assert_eq!(tokens.len(), 3);
    assert!(matches!(
        tokens[0],
        BreakToken::Text {
            start_idx: 0,
            end_idx: 2,
            ..
        }
    ));
    assert!(matches!(tokens[1], BreakToken::Space { idx: 2, .. }));
    assert!(matches!(
        tokens[2],
        BreakToken::Text {
            start_idx: 3,
            end_idx: 5,
            ..
        }
    ));
}

/// 토크나이저: 한국어 글자 단위 토큰화
#[test]
fn test_tokenize_korean_character_unit() {
    let styles = make_styles_with_font_size(16.0);
    let text: Vec<char> = "가나".chars().collect();
    let offsets: Vec<u32> = (0..text.len() as u32).collect();
    let shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];

    let tokens = tokenize_paragraph(&text, &offsets, &shapes, &styles, 0, 1);
    assert_eq!(tokens.len(), 2);
    assert!(matches!(
        tokens[0],
        BreakToken::Text {
            start_idx: 0,
            end_idx: 1,
            ..
        }
    ));
    assert!(matches!(
        tokens[1],
        BreakToken::Text {
            start_idx: 1,
            end_idx: 2,
            ..
        }
    ));
}

/// 토크나이저: 영어 단어 토큰화
#[test]
fn test_tokenize_english_words() {
    let styles = make_styles_with_font_size(16.0);
    let text: Vec<char> = "AB CD".chars().collect();
    let offsets: Vec<u32> = (0..text.len() as u32).collect();
    let shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];

    let tokens = tokenize_paragraph(&text, &offsets, &shapes, &styles, 0, 0);
    // "AB" (Text) + " " (Space) + "CD" (Text) = 3 tokens
    assert_eq!(tokens.len(), 3);
    assert!(matches!(
        tokens[0],
        BreakToken::Text {
            start_idx: 0,
            end_idx: 2,
            ..
        }
    ));
    assert!(matches!(tokens[1], BreakToken::Space { idx: 2, .. }));
    assert!(matches!(
        tokens[2],
        BreakToken::Text {
            start_idx: 3,
            end_idx: 5,
            ..
        }
    ));
}

/// 토크나이저: 줄 바꿈 토큰
#[test]
fn test_tokenize_line_break() {
    let styles = make_styles_with_font_size(16.0);
    let text: Vec<char> = "가\n나".chars().collect();
    let offsets: Vec<u32> = (0..text.len() as u32).collect();
    let shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];

    let tokens = tokenize_paragraph(&text, &offsets, &shapes, &styles, 0, 0);
    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[1], BreakToken::LineBreak { idx: 1 }));
}

// ─── Task #555: PUA 옛한글 → 자모 변환 후 폰트 매트릭스 ───

/// Task #555 RED: `effective_text_for_metrics` 가 `display_text` 가 있을 때
/// 자모 시퀀스를 반환해야 한다 (현재 STUB 은 `text` 반환 → RED).
///
/// PUA 옛한글 char (예: U+F861 책괄호) 가 `display_text` 에 자모 시퀀스 ("《")
/// 로 변환되어 있는 경우, 폰트 매트릭스 측정 (estimate_text_width 등) 은
/// 자모 시퀀스 기준으로 수행되어야 함.
#[test]
fn test_555_effective_text_for_metrics_uses_display_text_when_present() {
    let run = ComposedTextRun {
        text: "\u{F861}".to_string(), // PUA 책괄호 (1 char)
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: Some("《".to_string()), // 변환된 자모 (1 char in this case)
    };
    let effective = super::effective_text_for_metrics(&run);
    assert_eq!(
        effective, "《",
        "PUA 옛한글 변환 후 폰트 매트릭스는 display_text (자모 시퀀스) 기준이어야 함. \
         현재 STUB 은 text (PUA 1글자) 반환 → 자모 시퀀스 폭과 불일치."
    );
}

/// Task #555 RED: 옛한글 합자 PUA char 의 4-자모 시퀀스 변환 케이스.
///
/// 예: "" (옛한글 합자, 1 PUA char) → "ᄃᆞᄫᆡ" (4 jamo chars).
/// 폰트 매트릭스는 4 char 폭으로 측정되어야 함.
#[test]
fn test_555_effective_text_for_metrics_multi_jamo_cluster() {
    let run = ComposedTextRun {
        text: "\u{F8E0}".to_string(), // PUA 옛한글 합자 (가상 codepoint, 1 char)
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: Some("ᄃᆞᄫᆡ".to_string()), // 4 jamo chars
    };
    let effective = super::effective_text_for_metrics(&run);
    assert_eq!(
        effective.chars().count(),
        4,
        "옛한글 합자 PUA → 4-jamo 시퀀스 변환 시 폰트 매트릭스 char count 도 4 이어야 함."
    );
    assert_eq!(effective, "ᄃᆞᄫᆡ");
}

/// Task #555 GREEN: `display_text` 가 None 이면 `text` 그대로 반환 (비-PUA fallback).
///
/// 비-PUA 텍스트는 `display_text=None` 이므로 본 함수는 `text` 를 그대로 반환.
/// 회귀 가드 — 옵션 A 적용 후에도 비-PUA 영역 동작 동일.
#[test]
fn test_555_effective_text_for_metrics_no_display_text_falls_back_to_text() {
    let run = ComposedTextRun {
        text: "한글".to_string(),
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: None,
    };
    let effective = super::effective_text_for_metrics(&run);
    assert_eq!(
        effective, "한글",
        "display_text=None 인 경우 text 그대로 반환. 비-PUA fallback 회귀 가드."
    );
}

/// Issue #677: U+F081C HWP TAC filler 는 시각 폭 0으로 측정되어야 한다.
///
/// filler 원문이 display_text 로 치환되면 `text_measurement` 의 0폭 분기를
/// 우회하여 복학원서 접수증 블록이 우측으로 밀린다.
#[test]
fn test_677_effective_text_for_metrics_preserves_f081c_filler() {
    let run = ComposedTextRun {
        text: "\u{F081C}\u{F081C}".to_string(),
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: Some("□□".to_string()),
    };
    let effective = super::effective_text_for_metrics(&run);
    assert_eq!(
        effective, "\u{F081C}\u{F081C}",
        "U+F081C filler 는 0폭 측정 규칙을 유지하기 위해 원문으로 측정해야 함."
    );
}

/// 방점(U+302E/U+302F)은 유니코드 결합문자라 유효 base 없이(줄 시작/공백 뒤)
/// 셰이핑되면 dotted-circle(U+25CC) placeholder 아티팩트가 생긴다. 렌더 확장
/// 경로에서 spacing 가운데 점으로 치환해 한컴 정합을 맞춘다. (Task #1735)
#[test]
fn test_expand_tone_marks_to_spacing_dot() {
    // U+302E HANGUL SINGLE DOT TONE MARK → · (U+00B7 MIDDLE DOT)
    let out = expand_pua_render_text("\u{302E} 각");
    assert!(!out.contains('\u{302E}'), "원본 방점이 남으면 안 됨");
    assert!(!out.contains('\u{25CC}'), "dotted-circle 아티팩트 금지");
    assert_eq!(out, "\u{00B7} 각", "선두 방점은 가운데 점으로 치환");

    // U+302F HANGUL DOUBLE DOT TONE MARK → ⁚ (U+205A TWO DOT PUNCTUATION)
    let out2 = expand_pua_render_text("\u{302F}가");
    assert_eq!(out2, "\u{205A}가", "쌍방점은 세로 두 점으로 치환");
}

#[test]
fn test_expand_hancom_relationship_line_pua_to_box_drawing() {
    let out = expand_pua_render_text("\u{F0811}\u{F0817}\u{F081A}");
    assert_eq!(
        out, "┌└─",
        "한컴 관계도 PUA 선문자는 공개 폰트 환경에서 두부가 아닌 box drawing 문자로 표시되어야 함"
    );
}

/// #3486 — legacy 한컴 제품명은 raw HWP의 옛자모를 보존하면서 PDF와 같은
/// 현대 product spelling으로만 표시한다. 보통 옛한글 낱말은 건드리지 않는다.
#[test]
fn legacy_hancom_product_names_use_display_projection_only() {
    let text = "ᄒᆞᆫ글, ᄒᆞᆫ메일, ᄒᆞᆫ팩스, ᄒᆞᆫ소프트, ᄒᆞᆫ겨울";
    let char_count = text.chars().count();
    let para = Paragraph {
        text: text.to_string(),
        char_offsets: (0..char_count as u32).collect(),
        char_count: char_count as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 400,
            baseline_distance: 320,
            ..Default::default()
        }],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    let run = &composed.lines[0].runs[0];
    assert_eq!(run.text, text, "원문 IR은 바꾸지 않는다");
    assert_eq!(
        run.display_text.as_deref(),
        Some("한글, 한메일, 한팩스, 한소프트, ᄒᆞᆫ겨울"),
        "닫힌 legacy 제품명 어휘만 한컴 PDF 표기처럼 투영한다"
    );
}

/// #3486 — 제품명은 HWP line-seg나 글자모양 경계에서 나뉠 수 있다. `ᄒᆞᆫ`과
/// 뒤의 `글`이 다른 run이어도 첫 run에만 `한`을 투영해 model offset은 유지한다.
#[test]
fn legacy_hancom_product_projection_survives_line_boundary() {
    let text = "ᄒᆞᆫ글";
    let para = Paragraph {
        text: text.to_string(),
        char_offsets: vec![0, 1, 2, 3],
        char_count: 5,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            },
            LineSeg {
                text_start: 3,
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 2);
    assert_eq!(composed.lines[0].runs[0].text, "ᄒᆞᆫ");
    assert_eq!(
        composed.lines[0].runs[0].display_text.as_deref(),
        Some("한")
    );
    assert_eq!(composed.lines[1].runs[0].text, "글");
    assert_eq!(composed.lines[1].runs[0].display_text, None);
}

/// [#2244] KBU=1(글자 단위) 줄바꿈에서 행두 금칙 문자 retraction —
/// 새 줄이 마침표로 시작하지 않도록 직전 글자를 함께 이월한다.
/// 한컴 2024 저장 오라클: "…하여 적용한 | 다.111…" (LINE_SEG [...,128] —
/// '다'(128) 앞에서 분리, '.'(129) 고립 금지).
#[test]
fn test_kbu1_line_start_forbidden_retraction() {
    let styles = make_styles_with_font_size(16.0);
    let line = ComposedLine {
        runs: vec![ComposedTextRun {
            text: "적용한다.111111".to_string(),
            char_style_id: 0,
            lang_index: 0,
            char_overlap: None,
            footnote_marker: None,
            display_text: None,
        }],
        line_height: 400,
        baseline_distance: 320,
        segment_width: 0,
        column_start: 0,
        line_spacing: 0,
        has_line_break: false,
        char_start: 0,
    };
    // 한글 4자(64px)는 들어가고 '.'에서 초과하는 폭 → 수정 전엔 둘째 줄이
    // "."로 시작 ("적용한다 | .111111"), 수정 후엔 '다' 동반 이월.
    let frags =
        split_composed_line_by_width(&line, 68.0, 68.0, &styles, true, 0.0, false, 0.0, false);
    assert!(
        frags.len() >= 2,
        "두 줄 이상으로 분할되어야 함: {:?}",
        frags.len()
    );
    let line2_text: String = frags[1].runs.iter().map(|r| r.text.as_str()).collect();
    assert!(
        !line2_text.starts_with('.'),
        "새 줄이 행두 금칙 '.'로 시작하면 안 됨 (한컴: 직전 글자 동반 이월): {:?}",
        line2_text
    );
    assert!(
        line2_text.starts_with("다."),
        "한컴 오라클 정합: 둘째 줄은 '다.'로 시작해야 함: {:?}",
        line2_text
    );
    // char_start 정합: 둘째 줄 시작 = '다' 위치(3)
    assert_eq!(
        frags[1].char_start, 3,
        "retraction 후 char_start 는 '다' 위치"
    );
}

// ───────────────────── [#4149] 셀 단일줄 과밀 판정 memo ─────────────────────

/// 가드 전제(저장 단일 lineseg, 비합성 tag)를 만족하는 문단.
fn issue4149_guard_para(text: &str) -> Paragraph {
    let n = text.chars().count();
    Paragraph {
        text: text.to_string(),
        char_offsets: (0..n as u32).collect(),
        char_count: n as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        // 저장 단일 lineseg (tag=0 → TAG_IMPLEMENTATION_PROPERTY 미설정 = 비합성).
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 800,
            baseline_distance: 640,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// 두 개의 실제 fresh composition이 renderer-session cache를 공유한다. hit에서도
/// over=true의 fresh 재래핑은 수행하되 비싼 실폭 측정은 한 번만 한다.
#[test]
fn issue4149_renderer_cache_survives_fresh_compositions() {
    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();
    let para = issue4149_guard_para(&"가".repeat(60));
    let width = 50.0; // 60자 실폭 ≫ 50×1.8
    let cache = SingleLineOverflowCache::default();

    let mut composed = compose_paragraph(&para);
    recompose_stored_single_line_if_overflowing_cached(
        &mut composed,
        &para,
        width,
        &styles,
        96.0,
        Some(&cache),
    );
    assert!(composed.lines.len() > 1);

    let mut composed2 = compose_paragraph(&para);
    recompose_stored_single_line_if_overflowing_cached(
        &mut composed2,
        &para,
        width,
        &styles,
        96.0,
        Some(&cache),
    );
    assert!(
        composed2.lines.len() > 1,
        "cache hit에도 derived rewrap은 새 composition에 적용돼야 함"
    );
    assert_eq!(cache.measurement_count(), 1);
}

/// 실제 소비 frame이 바뀌면 같은 source paragraph도 별도 판정이다.
#[test]
fn issue4149_renderer_cache_keys_actual_cell_width() {
    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();
    let para = issue4149_guard_para(&"가".repeat(60));
    let cache = SingleLineOverflowCache::default();
    for width in [50.0, 5000.0] {
        let mut composed = compose_paragraph(&para);
        recompose_stored_single_line_if_overflowing_cached(
            &mut composed,
            &para,
            width,
            &styles,
            96.0,
            Some(&cache),
        );
    }
    assert_eq!(
        cache.measurement_count(),
        2,
        "서로 다른 frame width는 별도 key여야 함"
    );
}

/// Source/style revision 경계는 renderer owner 전체를 비운다. Source mutation
/// site는 memo 필드를 알거나 직접 무효화하지 않는다.
#[test]
fn issue4149_renderer_cache_clear_owns_source_invalidation() {
    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();
    let mut para = issue4149_guard_para(&"가".repeat(60));
    let cache = SingleLineOverflowCache::default();
    let mut composed = compose_paragraph(&para);
    recompose_stored_single_line_if_overflowing_cached(
        &mut composed,
        &para,
        50.0,
        &styles,
        96.0,
        Some(&cache),
    );
    para.insert_text_at(0, "X");
    cache.clear();
    let mut recomposed = compose_paragraph(&para);
    recompose_stored_single_line_if_overflowing_cached(
        &mut recomposed,
        &para,
        50.0,
        &styles,
        96.0,
        Some(&cache),
    );
    assert_eq!(cache.measurement_count(), 2);
}

/// 정합(비과밀) 판정도 cache되고 재래핑은 일어나지 않는다.
#[test]
fn issue4149_fit_judgment_is_cached_without_rewrap() {
    let styles = crate::renderer::style_resolver::ResolvedStyleSet::default();
    let para = issue4149_guard_para("가나다");
    let width = 5000.0;
    let cache = SingleLineOverflowCache::default();
    let mut composed = compose_paragraph(&para);
    recompose_stored_single_line_if_overflowing_cached(
        &mut composed,
        &para,
        width,
        &styles,
        96.0,
        Some(&cache),
    );
    assert_eq!(
        composed.lines.len(),
        1,
        "정합 단일줄은 재래핑하지 않아야 함"
    );
    assert_eq!(cache.measurement_count(), 1);
}

#[test]
fn owned_rowbreak_tac_height_selects_current_or_multirow_frames() {
    use crate::model::control::Control;
    use crate::model::table::{Table, TablePageBreak};

    let para_with_rows = |row_count, line_height| Paragraph {
        controls: vec![Control::Table(Box::new(Table {
            row_count,
            page_break: TablePageBreak::RowBreak,
            common: crate::model::shape::CommonObjAttr {
                treat_as_char: true,
                height: 32_339,
                ..Default::default()
            },
            ..Default::default()
        }))],
        line_segs: vec![LineSeg {
            line_height,
            segment_width: 49_324,
            ..Default::default()
        }],
        ..Default::default()
    };

    let multirow = para_with_rows(4, 32_339);
    assert_eq!(owned_rowbreak_tac_height(&multirow, 0), Some(32_339));

    let single_row = para_with_rows(1, 32_339);
    assert_eq!(owned_rowbreak_tac_height(&single_row, 0), None);

    let mut current_single_row = para_with_rows(1, 32_339);
    current_single_row.line_segs[0].tag = LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    assert_eq!(
        owned_rowbreak_tac_height(&current_single_row, 0),
        Some(32_339)
    );

    let undersized = para_with_rows(4, 32_338);
    assert_eq!(owned_rowbreak_tac_height(&undersized, 0), None);
}
