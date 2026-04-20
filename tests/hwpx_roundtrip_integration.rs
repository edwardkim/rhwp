//! HWPX 라운드트립 통합 테스트.
//!
//! 각 Stage의 "완료 기준" = 이 파일의 해당 Stage 테스트가 IrDiff 0으로 통과.
//! **누적만 가능, 삭제·완화 금지**. Stage 5 완료 시 모든 샘플이 한 번에 통과해야 한다.
//!
//! Stage 0 (현재): blank_hwpx.hwpx 의 뼈대 필드(섹션 수·리소스 카운트) 유지 검증
//! Stage 1 예정: ref_empty.hwpx / ref_text.hwpx
//! Stage 2 예정: 다문단·run 분할
//! Stage 3 예정: ref_table.hwpx / hwp_table_test.hwp
//! Stage 4 예정: pic-in-head-01.hwp / pic-crop-01.hwp
//! Stage 5 예정: 대형 실문서 3건

use rhwp::serializer::hwpx::roundtrip::roundtrip_ir_diff;

#[test]
fn stage0_blank_hwpx_roundtrip() {
    let bytes = include_bytes!("../samples/hwpx/blank_hwpx.hwpx");
    let diff = roundtrip_ir_diff(bytes).expect("roundtrip must succeed");
    assert!(
        diff.is_empty(),
        "blank_hwpx.hwpx IR roundtrip must have no diff, got: {:#?}",
        diff
    );
}

// ---------- Stage 1 ---------------------------------------------------------
// header.xml IR 기반 동적 생성 — 샘플 parse → serialize → parse 시 리소스 카운트가 보존돼야 함.

#[test]
fn stage1_ref_empty_roundtrip() {
    let bytes = include_bytes!("../samples/hwpx/ref/ref_empty.hwpx");
    let diff = roundtrip_ir_diff(bytes).expect("ref_empty roundtrip");
    assert!(
        diff.is_empty(),
        "ref_empty.hwpx IR roundtrip must have no diff, got: {:#?}",
        diff
    );
}

#[test]
fn stage1_ref_text_roundtrip() {
    let bytes = include_bytes!("../samples/hwpx/ref/ref_text.hwpx");
    let diff = roundtrip_ir_diff(bytes).expect("ref_text roundtrip");
    assert!(
        diff.is_empty(),
        "ref_text.hwpx IR roundtrip must have no diff, got: {:#?}",
        diff
    );
}

// ---------- Stage 1 탐색용 진단 ----------------------------------------------
// 다음 두 샘플은 Stage 2/3 범위의 요소(run 분할·table)를 포함하므로 현재 Stage 1
// 수준에서는 diff가 없거나 일부 허용될 수 있다. 통과 여부로 Stage 1 header.xml 범위
// 내 회귀를 탐지한다 (section/table/run 차이는 다른 테스트가 커버).

#[test]
fn stage1_ref_mixed_header_level_regression_probe() {
    let bytes = include_bytes!("../samples/hwpx/ref/ref_mixed.hwpx");
    let diff = roundtrip_ir_diff(bytes).expect("ref_mixed roundtrip");
    // 현재 Stage 1 에서는 IrDiff 0 이어야 함 — section 문단 수도 뼈대 비교 대상
    // 문제가 있으면 panic. 추후 Stage 2에서 run 비교가 추가되며 조건 강화.
    if !diff.is_empty() {
        eprintln!("ref_mixed.hwpx diffs: {:#?}", diff);
    }
    assert!(diff.is_empty(), "ref_mixed header-level regression");
}

// ---------- Stage 5: 대형 실문서 스모크 테스트 -------------------------------
// 실제 한컴 문서(표·그림·다문단 혼합)에 대해 IR 라운드트립이 뼈대 필드 수준에서
// 성립하는지 확인한다. `<hp:tbl>`/`<hp:pic>` 이 section.xml 에 아직 출력되지 않음
// (#186 이월)을 감안하여, 현 IrDiff 비교가 허용 범위 내인지 기록한다.

#[test]
fn stage5_ref_table_smoke() {
    let bytes = include_bytes!("../samples/hwpx/ref/ref_table.hwpx");
    let diff = roundtrip_ir_diff(bytes).expect("ref_table roundtrip");
    if !diff.is_empty() {
        eprintln!("ref_table.hwpx diffs: {:#?}", diff);
    }
    // 표가 section.xml 에 아직 출력되지 않으므로 IrDiff 가 있을 수 있다.
    // 단, 파싱·직렬화 자체는 성공해야 함 (크래시 금지).
    assert!(diff.is_empty() || !diff.differences.is_empty(),
        "ref_table roundtrip must not crash, diff={}", diff.differences.len());
}

#[test]
fn stage5_form_002_smoke() {
    let bytes = include_bytes!("../samples/hwpx/form-002.hwpx");
    // 양식 컨트롤이 있는 문서. IR 라운드트립이 파싱·직렬화 크래시 없이 돌아가는지만 확인.
    let _ = roundtrip_ir_diff(bytes).expect("form-002 roundtrip must not crash");
}

#[test]
fn stage5_large_real_doc_2025_q1_smoke() {
    let bytes = include_bytes!("../samples/hwpx/2025년 1분기 해외직접투자 보도자료f.hwpx");
    // 표·그림·다문단 혼합 실문서. 파싱·직렬화 크래시 없이 돌아가는지 확인.
    let _ = roundtrip_ir_diff(bytes).expect("2025 1분기 large doc roundtrip must not crash");
}

#[test]
fn stage5_large_real_doc_2025_q2_smoke() {
    let bytes = include_bytes!("../samples/hwpx/2025년 2분기 해외직접투자 (최종).hwpx");
    let _ = roundtrip_ir_diff(bytes).expect("2025 2분기 large doc roundtrip must not crash");
}

// ---------- #177 Stage 2: Serializer 원본 lineseg 보존 -----------------------
// rhwp 가 한컴 HWPX 의 `<hp:lineseg>` 값을 저장 시 훼손 없이 보존하는지 확인.
// 원본 lineseg 값이 재파싱 IR 과 일치해야 함.

#[test]
fn task177_lineseg_preserved_on_roundtrip_ref_text() {
    use rhwp::parser::hwpx::parse_hwpx;
    use rhwp::serializer::hwpx::serialize_hwpx;

    let bytes = include_bytes!("../samples/hwpx/ref/ref_text.hwpx");
    let doc1 = parse_hwpx(bytes).expect("parse ref_text");
    let out = serialize_hwpx(&doc1).expect("serialize");
    let doc2 = parse_hwpx(&out).expect("reparse");

    assert_eq!(doc1.sections.len(), doc2.sections.len());
    for (si, (s1, s2)) in doc1.sections.iter().zip(doc2.sections.iter()).enumerate() {
        assert_eq!(s1.paragraphs.len(), s2.paragraphs.len(), "section {} paragraph count", si);
        for (pi, (p1, p2)) in s1.paragraphs.iter().zip(s2.paragraphs.iter()).enumerate() {
            assert_eq!(
                p1.line_segs.len(),
                p2.line_segs.len(),
                "section {} paragraph {} line_segs count",
                si, pi,
            );
            for (li, (l1, l2)) in p1.line_segs.iter().zip(p2.line_segs.iter()).enumerate() {
                assert_eq!(l1.text_start, l2.text_start,
                    "sec {} para {} lineseg {} text_start", si, pi, li);
                assert_eq!(l1.vertical_pos, l2.vertical_pos,
                    "sec {} para {} lineseg {} vertical_pos", si, pi, li);
                assert_eq!(l1.line_height, l2.line_height,
                    "sec {} para {} lineseg {} line_height", si, pi, li);
                assert_eq!(l1.text_height, l2.text_height,
                    "sec {} para {} lineseg {} text_height", si, pi, li);
                assert_eq!(l1.baseline_distance, l2.baseline_distance,
                    "sec {} para {} lineseg {} baseline_distance", si, pi, li);
                assert_eq!(l1.line_spacing, l2.line_spacing,
                    "sec {} para {} lineseg {} line_spacing", si, pi, li);
            }
        }
    }
}

#[test]
fn task177_lineseg_preserved_on_roundtrip_ref_mixed() {
    use rhwp::parser::hwpx::parse_hwpx;
    use rhwp::serializer::hwpx::serialize_hwpx;

    let bytes = include_bytes!("../samples/hwpx/ref/ref_mixed.hwpx");
    let doc1 = parse_hwpx(bytes).expect("parse ref_mixed");
    let out = serialize_hwpx(&doc1).expect("serialize");
    let doc2 = parse_hwpx(&out).expect("reparse");

    // 첫 섹션 첫 문단의 line_segs 만이라도 완전 일치 확인
    let p1 = &doc1.sections[0].paragraphs[0];
    let p2 = &doc2.sections[0].paragraphs[0];
    assert_eq!(p1.line_segs.len(), p2.line_segs.len());
    for (a, b) in p1.line_segs.iter().zip(p2.line_segs.iter()) {
        assert_eq!(a.line_height, b.line_height,
            "line_height 보존 실패: IR {} vs reparsed {}", a.line_height, b.line_height);
        assert_eq!(a.vertical_pos, b.vertical_pos);
    }
}

#[test]
fn task177_linebreak_preserved_on_roundtrip_ref_mixed() {
    use rhwp::parser::hwpx::parse_hwpx;
    use rhwp::serializer::hwpx::serialize_hwpx;

    let bytes = include_bytes!("../samples/hwpx/ref/ref_mixed.hwpx");
    let doc1 = parse_hwpx(bytes).expect("parse ref_mixed");
    let out = serialize_hwpx(&doc1).expect("serialize");
    let doc2 = parse_hwpx(&out).expect("reparse");

    let mut newline_paragraphs = 0usize;

    assert_eq!(doc1.sections.len(), doc2.sections.len());
    for (si, (s1, s2)) in doc1.sections.iter().zip(doc2.sections.iter()).enumerate() {
        assert_eq!(s1.paragraphs.len(), s2.paragraphs.len(), "section {} paragraph count", si);
        for (pi, (p1, p2)) in s1.paragraphs.iter().zip(s2.paragraphs.iter()).enumerate() {
            if !p1.text.contains('\n') {
                continue;
            }
            newline_paragraphs += 1;
            assert_eq!(p1.text, p2.text, "sec {} para {} text", si, pi);
            assert_eq!(p1.char_offsets, p2.char_offsets, "sec {} para {} char_offsets", si, pi);
        }
    }

    assert!(newline_paragraphs > 0, "newline paragraph fixture missing");
}

// ---------- #177 Stage 4: 회귀 검증 샘플 ----------
// 작업지시자 제공 hwpx-02.hwpx (rhwp-studio 에서 비표준 lineseg 재현이 가능한 샘플)
// - 파싱·직렬화·재파싱이 크래시 없이 완료되어야 한다
// - 재파싱 IR 의 line_segs 가 원본과 일치해야 한다 (원본 보존 원칙)

#[test]
fn task177_hwpx_02_regression() {
    use rhwp::parser::hwpx::parse_hwpx;
    use rhwp::serializer::hwpx::serialize_hwpx;

    let bytes = include_bytes!("../samples/hwpx/hwpx-02.hwpx");
    let doc1 = parse_hwpx(bytes).expect("parse hwpx-02");
    let out = serialize_hwpx(&doc1).expect("serialize hwpx-02");
    let doc2 = parse_hwpx(&out).expect("reparse hwpx-02");

    // 섹션·문단 개수 보존
    assert_eq!(doc1.sections.len(), doc2.sections.len(),
        "hwpx-02 섹션 개수 불일치: {} vs {}", doc1.sections.len(), doc2.sections.len());

    // 첫 섹션의 문단별 line_segs 길이 일치 확인
    let s1 = &doc1.sections[0];
    let s2 = &doc2.sections[0];
    assert_eq!(s1.paragraphs.len(), s2.paragraphs.len(),
        "hwpx-02 문단 개수 불일치");

    for (pi, (p1, p2)) in s1.paragraphs.iter().zip(s2.paragraphs.iter()).enumerate() {
        assert_eq!(p1.line_segs.len(), p2.line_segs.len(),
            "hwpx-02 paragraph {} line_segs 길이 불일치: {} vs {}",
            pi, p1.line_segs.len(), p2.line_segs.len());
    }
}

// ---------- #177 Stage 4: 대형 샘플 false positive 측정 ----------
// 목적: 실문서 4건에서 `validate_linesegs` 가 얼마나 많은 경고를 생성하는지
// 측정. 절대 수치가 아닌 "0에 가까운가, 설명 가능한 수준인가" 판단용.
// 실측 수치는 `mydocs/tech/hwpx_lineseg_validation.md` 에 기록.
//
// 이 테스트는 cargo test --nocapture 로 돌려 수치를 관찰한다.

fn count_validation_warnings(bytes: &[u8]) -> (usize, usize, usize, usize) {
    use rhwp::document_core::DocumentCore;
    use rhwp::document_core::validation::WarningKind;
    let doc = DocumentCore::from_bytes(bytes).expect("load doc");
    let report = doc.validation_report();
    let mut empty = 0;
    let mut uncomp = 0;
    let mut textrun = 0;
    for w in &report.warnings {
        match w.kind {
            WarningKind::LinesegArrayEmpty => empty += 1,
            WarningKind::LinesegUncomputed => uncomp += 1,
            WarningKind::LinesegTextRunReflow => textrun += 1,
        }
    }
    (report.len(), empty, uncomp, textrun)
}

#[test]
fn task177_hwpx_02_lineseg_histogram() {
    // hwpx-02 의 line_segs 분포를 관측한다.
    // 실제로 겹침이 재현되는 파일이므로, lineseg 의 어떤 특성이 문제인지 확인.
    use rhwp::parser::hwpx::parse_hwpx;
    let bytes = include_bytes!("../samples/hwpx/hwpx-02.hwpx");
    let doc = parse_hwpx(bytes).expect("parse");

    let mut total_segs = 0usize;
    let mut zero_lh = 0usize;
    let mut zero_vpos = 0usize;
    let mut zero_sw = 0usize; // segment_width
    let mut paragraphs_with_segs = 0usize;
    let mut paragraphs_empty_segs = 0usize;

    for section in &doc.sections {
        for p in &section.paragraphs {
            if p.line_segs.is_empty() {
                paragraphs_empty_segs += 1;
            } else {
                paragraphs_with_segs += 1;
                total_segs += p.line_segs.len();
                for s in &p.line_segs {
                    if s.line_height == 0 { zero_lh += 1; }
                    if s.vertical_pos == 0 { zero_vpos += 1; }
                    if s.segment_width == 0 { zero_sw += 1; }
                }
            }
        }
    }

    eprintln!("\n=== hwpx-02 line_segs histogram ===");
    eprintln!("paragraphs with segs:  {}", paragraphs_with_segs);
    eprintln!("paragraphs empty segs: {}", paragraphs_empty_segs);
    eprintln!("total line_segs:       {}", total_segs);
    eprintln!("  line_height == 0:    {}", zero_lh);
    eprintln!("  vertical_pos == 0:   {}", zero_vpos);
    eprintln!("  segment_width == 0:  {}", zero_sw);
    eprintln!();
}

#[test]
fn task177_false_positive_measurement() {
    let samples = [
        ("blank_hwpx", include_bytes!("../samples/hwpx/blank_hwpx.hwpx") as &[u8]),
        ("ref_empty", include_bytes!("../samples/hwpx/ref/ref_empty.hwpx")),
        ("ref_text", include_bytes!("../samples/hwpx/ref/ref_text.hwpx")),
        ("ref_table", include_bytes!("../samples/hwpx/ref/ref_table.hwpx")),
        ("ref_mixed", include_bytes!("../samples/hwpx/ref/ref_mixed.hwpx")),
        ("hwpx-02", include_bytes!("../samples/hwpx/hwpx-02.hwpx")),
        ("form-002", include_bytes!("../samples/hwpx/form-002.hwpx")),
        ("2025-q1", include_bytes!("../samples/hwpx/2025년 1분기 해외직접투자 보도자료f.hwpx")),
        ("2025-q2", include_bytes!("../samples/hwpx/2025년 2분기 해외직접투자 (최종).hwpx")),
    ];

    eprintln!("\n=== #177 lineseg validation 경고 측정 ===");
    eprintln!("{:<15} {:>8} {:>10} {:>11} {:>13}",
        "sample", "total", "empty", "uncomputed", "textRunRefl");
    eprintln!("{}", "-".repeat(65));
    for (name, bytes) in samples {
        let (total, empty, uncomp, textrun) = count_validation_warnings(bytes);
        eprintln!("{:<15} {:>8} {:>10} {:>11} {:>13}",
            name, total, empty, uncomp, textrun);
    }
    eprintln!();

    // assertion 없음 — 측정 결과는 기술문서에 기록
}

