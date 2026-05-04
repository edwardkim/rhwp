use super::*;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};

/// 단일 줄, 단일 스타일 문단
#[test]
fn test_compose_single_line_single_style() {
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
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
            CharShapeRef { start_pos: 3, char_shape_id: 2 },
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

/// 다중 줄 + 다중 스타일 (줄 경계에서 스타일 변경)
#[test]
fn test_compose_multi_line_multi_style() {
    let para = Paragraph {
        text: "AAABBBCCCC".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        char_count: 11,
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
            CharShapeRef { start_pos: 3, char_shape_id: 2 },
            CharShapeRef { start_pos: 6, char_shape_id: 3 },
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

    let para = Paragraph {
        text: "표 앞 텍스트".to_string(),
        controls: vec![Control::Table(Box::new(Table::default()))],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.inline_controls.len(), 1);
    assert_eq!(composed.inline_controls[0].control_type, InlineControlType::Table);
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

/// find_active_char_shape 테스트
#[test]
fn test_find_active_char_shape() {
    let shapes = vec![
        CharShapeRef { start_pos: 0, char_shape_id: 1 },
        CharShapeRef { start_pos: 10, char_shape_id: 2 },
        CharShapeRef { start_pos: 20, char_shape_id: 3 },
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
        char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    // 컬럼 너비 500px → "안녕" (16*2=32px) 충분히 들어감
    reflow_line_segs(&mut para, 500.0, &styles, 96.0);
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
        char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    // 컬럼 너비 80px → 16px * 5글자 = 80px → 5글자씩 2줄
    reflow_line_segs(&mut para, 80.0, &styles, 96.0);
    assert_eq!(para.line_segs.len(), 2);
    assert_eq!(para.line_segs[0].text_start, 0);
    assert_eq!(para.line_segs[1].text_start, 5); // 6번째 글자부터 2번째 줄
}

/// 빈 텍스트 → 기본 LineSeg 1개
#[test]
fn test_reflow_empty_text() {
    let styles = make_styles_with_font_size(16.0);
    let mut para = Paragraph::default();

    reflow_line_segs(&mut para, 500.0, &styles, 96.0);
    assert_eq!(para.line_segs.len(), 1);
    assert_eq!(para.line_segs[0].text_start, 0);
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
        char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    // 컬럼 너비 40px → 8px * 5글자 = 40px → 5글자씩 2줄
    reflow_line_segs(&mut para, 40.0, &styles, 96.0);
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
        char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    reflow_line_segs(&mut para, 500.0, &styles, 96.0);
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
        display_text: None,    }];
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
        display_text: None,    }];
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
        display_text: None,    }];
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
        display_text: None,    }];
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
        display_text: None,    }];
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
        char_styles: vec![ResolvedCharStyle {
            font_family: "함초롬돋움".to_string(),
            font_families: vec![
                "함초롬돋움".to_string(), // 한국어
                "Arial".to_string(),       // 영어
                "".to_string(), "".to_string(), "".to_string(),
                "".to_string(), "".to_string(),
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
        char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    // 너비 충분 → 1줄
    reflow_line_segs(&mut para, 500.0, &styles, 96.0);
    assert_eq!(para.line_segs.len(), 1);

    // 너비 부족 → 여러 줄 (언어별 폰트 적용 확인)
    reflow_line_segs(&mut para, 30.0, &styles, 96.0);
    assert!(para.line_segs.len() > 1, "좁은 너비에서 줄 바꿈이 발생해야 함");
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
        display_text: None,        }],
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
        char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    // 너비 100px → "안녕하세요" (80px) + " " (8px) = 88px 들어감
    // "반갑습니다" (80px) → 2번째 줄
    reflow_line_segs(&mut para, 100.0, &styles, 96.0);
    assert_eq!(para.line_segs.len(), 2, "어절 경계에서 줄 바꿈");
    assert_eq!(para.line_segs[0].text_start, 0);
    // 두 번째 줄은 공백 다음 글자부터 (char_offset 6)
    assert_eq!(para.line_segs[1].text_start, 6);
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
        char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    // 너비 60px → "Hello" (40px) + " " (8px) = 48px 들어감
    // "World" (40px) → 2번째 줄
    reflow_line_segs(&mut para, 60.0, &styles, 96.0);
    assert_eq!(para.line_segs.len(), 2, "단어 경계에서 줄 바꿈");
    assert_eq!(para.line_segs[0].text_start, 0);
    assert_eq!(para.line_segs[1].text_start, 6); // "World" 시작
}

/// 강제 줄 바꿈: \n에서 즉시 줄 바꿈
#[test]
fn test_reflow_forced_line_break() {
    let styles = make_styles_with_font_size(16.0);
    let mut para = Paragraph {
        text: "가나\n다라".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_count: 6,
        char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    reflow_line_segs(&mut para, 500.0, &styles, 96.0);
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
    let shapes = vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }];

    let tokens = tokenize_paragraph(&text, &offsets, &shapes, &styles, 0, 0);
    // "가나" (Text) + " " (Space) + "다라" (Text) = 3 tokens
    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0], BreakToken::Text { start_idx: 0, end_idx: 2, .. }));
    assert!(matches!(tokens[1], BreakToken::Space { idx: 2, .. }));
    assert!(matches!(tokens[2], BreakToken::Text { start_idx: 3, end_idx: 5, .. }));
}

/// 토크나이저: 영어 단어 토큰화
#[test]
fn test_tokenize_english_words() {
    let styles = make_styles_with_font_size(16.0);
    let text: Vec<char> = "AB CD".chars().collect();
    let offsets: Vec<u32> = (0..text.len() as u32).collect();
    let shapes = vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }];

    let tokens = tokenize_paragraph(&text, &offsets, &shapes, &styles, 0, 0);
    // "AB" (Text) + " " (Space) + "CD" (Text) = 3 tokens
    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0], BreakToken::Text { start_idx: 0, end_idx: 2, .. }));
    assert!(matches!(tokens[1], BreakToken::Space { idx: 2, .. }));
    assert!(matches!(tokens[2], BreakToken::Text { start_idx: 3, end_idx: 5, .. }));
}

/// 토크나이저: 줄 바꿈 토큰
#[test]
fn test_tokenize_line_break() {
    let styles = make_styles_with_font_size(16.0);
    let text: Vec<char> = "가\n나".chars().collect();
    let offsets: Vec<u32> = (0..text.len() as u32).collect();
    let shapes = vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }];

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
        text: "\u{F861}".to_string(),  // PUA 책괄호 (1 char)
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: Some("《".to_string()),  // 변환된 자모 (1 char in this case)
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
        text: "\u{F8E0}".to_string(),  // PUA 옛한글 합자 (가상 codepoint, 1 char)
        char_style_id: 0,
        lang_index: 0,
        char_overlap: None,
        footnote_marker: None,
        display_text: Some("ᄃᆞᄫᆡ".to_string()),  // 4 jamo chars
    };
    let effective = super::effective_text_for_metrics(&run);
    assert_eq!(
        effective.chars().count(), 4,
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
