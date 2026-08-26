//! `docdiff` — 두 문서 IR 의 **의미 차이**를 계산하는 재사용 엔진.
//!
//! 회귀 검증도, 왕복 시험도, 편집 전후 확인도 결국 같은 질문 하나를 던진다:
//! *"두 문서가 어떻게 다른가."* 그 답을 **라이브러리로 부를 수 있게** 만든 것이 이
//! 모듈이다. 입력은 이미 파싱된 [`Document`](crate::model::document::Document) 이고
//! (이 모듈은 파일을 열지 않는다), 출력은 좌표가 붙은 [`Finding`] 목록이다.
//!
//! ```
//! use rhwp::docdiff::{diff_documents, DiffOptions, FindingKind};
//! use rhwp::model::document::{Document, Section};
//! use rhwp::model::paragraph::Paragraph;
//!
//! let mut a = Document::default();
//! a.sections.push(Section {
//!     paragraphs: vec![Paragraph { text: "계약서".into(), ..Default::default() }],
//!     ..Default::default()
//! });
//! let mut b = a.clone();
//! b.sections[0].paragraphs[0].text = "계약서(안)".into();
//!
//! let diff = diff_documents(&a, &b, &DiffOptions::default());
//! assert!(!diff.identical);
//! assert_eq!(diff.findings[0].kind, FindingKind::TextChanged);
//! assert_eq!(diff.findings[0].path.to_string(), "sec[0]/para[0]");
//! ```
//!
//! # 기존 `ir-diff` 와의 경계
//!
//! 저장소에는 이미 문서를 맞대 보는 장치가 둘 있다. 이 모듈은 **셋째 장치가 아니라
//! 한 단계 위의 층**이다.
//!
//! | | 무엇을 묻나 | 어디에 사는가 |
//! |---|---|---|
//! | [`crate::serializer::hwpx::roundtrip::diff_documents`] | *저장했다 다시 읽었을 때 원본이 그대로 살아 있나* — 글자모양 시퀀스, `line_segs` 9 필드, 용지·여백, 캡션, 그림 크기 | 라이브러리 (재사용 가능) |
//! | `rhwp ir-diff` (CLI) | *두 파일의 IR 필드가 다른가* — 위의 것 + 본문 텍스트·`char_offsets`·탭·`ParaShape`·`TabDef` | `src/main.rs` 안의 비공개 함수 |
//! | **`docdiff` (이 모듈)** | *사람이 보기에 문서가 어떻게 달라졌나* — 문단이 늘었나 줄었나, 어느 문단 글이 바뀌었나, 표 모양이 바뀌었나 | 라이브러리 |
//!
//! 셋의 차이는 **충실도(fidelity)와 의미(semantics)** 의 차이다. 앞의 둘은 "한 비트도
//! 잃지 않았나"를 묻는 저장기 게이트라서, 문단 하나가 앞에 끼어들면 뒤따르는 문단
//! 전부를 "달라졌다"로 보고한다(둘 다 `a[i]` 대 `b[i]` 로 자리끼리 맞댄다). 편집·변환
//! 결과를 사람이나 에이전트에게 설명하는 자리에서는 그 잡음이 곧 쓸모없음이다.
//!
//! 그래서 이 모듈은 다르게 한다.
//!
//! - **정렬한다.** 문단 목록을 LCS 로 짝지어 삽입·삭제를 삽입·삭제로 본다. 한 문단
//!   삽입은 `ParagraphAdded` 1 건이지 뒤 문단 전부의 `TextChanged` 가 아니다.
//! - **좌표를 타입으로 준다.** `path: String` 이 아니라 [`NodePath`] 다. 소비자가
//!   문자열을 파싱하지 않고 구역 번호를 꺼낼 수 있다.
//! - **결과가 값이다.** 앞의 둘은 화면에 줄을 뿌리거나(`ir-diff`) 문자열 `detail` 을
//!   쌓는다. `ir-diff --json` 의 카테고리 집계마저 **출력 문자열의 앞부분을 되파싱해**
//!   만든다(`src/main.rs` 의 `IrDiffEmitter`). 여기서는 [`FindingKind`] 가 처음부터
//!   타입이고 [`DiffSummary`] 가 그것을 센다.
//! - **상한을 밝힌다.** 잘렸으면 [`DocumentDiff::truncated`] 로 말한다.
//!
//! 그러므로 둘 다 필요하다. 저장기 회귀는 여전히 충실도 게이트가 지켜야 하고
//! (한 비트라도 잃으면 실패여야 한다), 사람·에이전트에게 내보이는 "무엇이 바뀌었나"는
//! 이 엔진이 답한다.
//!
//! # 비범위
//!
//! - **파싱하지 않는다.** 입력은 `Document` IR 이다.
//! - **렌더 좌표를 보지 않는다.** 화면 위 픽셀 변위는
//!   [`crate::diagnostics::render_geom_diff`] 의 몫이다.
//! - **직렬화 충실도를 재지 않는다.** `line_segs`·`char_shapes`·원본 바이트 보존은
//!   왕복 게이트가 본다.
//! - **글상자·각주·머리말 안으로는 아직 들어가지 않는다.** 표 셀까지만 재귀한다.
//!   컨트롤 개수·종류 차이는 잡히므로 손실이 조용히 통과하지는 않는다.
//! - **`main.rs` 를 고치지 않는다.** `ir-diff` 를 이 엔진 위로 옮기는 것은 후속 작업이다.

mod compare;
mod model;
mod summary;

pub use compare::diff_documents;
pub use model::{DiffOptions, DiffSummary, DocumentDiff, Finding, FindingKind, NodePath, PathStep};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::Control;
    use crate::model::document::{Document, Section};
    use crate::model::paragraph::Paragraph;
    use crate::model::style::Style;
    use crate::model::table::{Cell, Table};

    /// 텍스트만 채운 문단.
    fn para(text: &str) -> Paragraph {
        Paragraph {
            text: text.to_string(),
            ..Default::default()
        }
    }

    /// 구역별 문단 텍스트 목록으로 문서를 짓는다.
    fn doc(sections: &[&[&str]]) -> Document {
        Document {
            sections: sections
                .iter()
                .map(|texts| Section {
                    paragraphs: texts.iter().map(|t| para(t)).collect(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    /// 행렬 크기와 셀별 텍스트로 표 컨트롤 하나를 품은 문단을 짓는다.
    fn para_with_table(rows: u16, cols: u16, cell_texts: &[&str]) -> Paragraph {
        let stride = cols.max(1);
        let cells = cell_texts
            .iter()
            .enumerate()
            .map(|(idx, text)| Cell {
                row: (idx as u16) / stride,
                col: (idx as u16) % stride,
                row_span: 1,
                col_span: 1,
                paragraphs: vec![para(text)],
                ..Default::default()
            })
            .collect();
        Paragraph {
            controls: vec![Control::Table(Box::new(Table {
                row_count: rows,
                col_count: cols,
                cells,
                ..Default::default()
            }))],
            ..Default::default()
        }
    }

    fn kinds(diff: &DocumentDiff) -> Vec<FindingKind> {
        diff.findings.iter().map(|f| f.kind).collect()
    }

    fn paths(diff: &DocumentDiff) -> Vec<String> {
        diff.findings.iter().map(|f| f.path.to_string()).collect()
    }

    /// 같은 문서를 맞대면 `identical` 이고 차이가 하나도 없다.
    #[test]
    fn identical_documents_report_no_findings() {
        let a = doc(&[&["첫 문단", "둘째 문단", "셋째 문단"]]);
        let b = a.clone();
        let diff = diff_documents(&a, &b, &DiffOptions::default());

        assert!(diff.identical);
        assert!(diff.findings.is_empty());
        assert!(!diff.truncated);
        assert_eq!(diff.summary().total, 0);
    }

    /// 빈 문서 대 빈 문서 — 구역이 아예 없어도, 빈 구역만 있어도 차이가 없다.
    #[test]
    fn empty_documents_are_identical() {
        let diff = diff_documents(
            &Document::default(),
            &Document::default(),
            &DiffOptions::default(),
        );
        assert!(diff.identical);
        assert!(diff.findings.is_empty());
        assert!(!diff.truncated);

        let diff2 = diff_documents(&doc(&[&[]]), &doc(&[&[]]), &DiffOptions::default());
        assert!(diff2.identical);
    }

    /// 문단 텍스트 한 글자 변경 — `TextChanged` 한 건이고 좌표가 그 문단을 짚는다.
    #[test]
    fn single_character_edit_is_one_text_change() {
        let a = doc(&[&["계약서", "제1조 목적", "제2조 범위"]]);
        let b = doc(&[&["계약서", "제1조 목적", "제2조 범위!"]]);
        let diff = diff_documents(&a, &b, &DiffOptions::default());

        assert!(!diff.identical);
        assert_eq!(kinds(&diff), vec![FindingKind::TextChanged]);
        assert_eq!(paths(&diff), vec!["sec[0]/para[2]"]);
        assert!(diff.findings[0].detail.contains("제2조 범위"));
    }

    /// 가운데 문단 추가 — 정렬 덕분에 뒤따르는 문단이 오염되지 않는다.
    #[test]
    fn inserted_paragraph_does_not_cascade() {
        let a = doc(&[&["가", "나", "다"]]);
        let b = doc(&[&["가", "새 문단", "나", "다"]]);
        let diff = diff_documents(&a, &b, &DiffOptions::default());

        // 자리끼리 맞대는 방식이라면 3 건(나→새 문단, 다→나, 없음→다)이 났을 것이다.
        assert_eq!(kinds(&diff), vec![FindingKind::ParagraphAdded]);
        assert_eq!(paths(&diff), vec!["sec[0]/para[1]"]);
        assert!(diff.findings[0].detail.contains("새 문단"));
    }

    /// 가운데 문단 삭제 — `ParagraphRemoved` 한 건.
    #[test]
    fn removed_paragraph_is_reported_once() {
        let a = doc(&[&["가", "나", "다", "라"]]);
        let b = doc(&[&["가", "다", "라"]]);
        let diff = diff_documents(&a, &b, &DiffOptions::default());

        assert_eq!(kinds(&diff), vec![FindingKind::ParagraphRemoved]);
        assert_eq!(paths(&diff), vec!["sec[0]/para[1]"]);
        assert!(diff.findings[0].detail.contains('나'));
    }

    /// 구역 수 변화 — 문서 뿌리 좌표에서 보고한다.
    #[test]
    fn section_count_change_is_reported_at_root() {
        let a = doc(&[&["가"]]);
        let b = doc(&[&["가"], &["나"]]);
        let diff = diff_documents(&a, &b, &DiffOptions::default());

        assert_eq!(kinds(&diff), vec![FindingKind::SectionCountChanged]);
        assert_eq!(paths(&diff), vec!["doc"]);
        assert!(diff.findings[0].detail.contains("A=1 B=2"));
    }

    /// 표 행렬 변화 — 행·열 수와 셀 수를 각각 짚는다.
    #[test]
    fn table_shape_change_is_detected() {
        let mut a = doc(&[&[]]);
        a.sections[0]
            .paragraphs
            .push(para_with_table(2, 2, &["A1", "A2", "B1", "B2"]));
        let mut b = doc(&[&[]]);
        b.sections[0]
            .paragraphs
            .push(para_with_table(3, 2, &["A1", "A2", "B1", "B2", "C1", "C2"]));

        let diff = diff_documents(&a, &b, &DiffOptions::default());

        assert_eq!(
            kinds(&diff),
            vec![
                FindingKind::TableShapeChanged,
                FindingKind::TableShapeChanged,
            ]
        );
        assert_eq!(diff.findings[0].path.to_string(), "sec[0]/para[0]/ctrl[0]");
        assert!(diff.findings[0].detail.contains("A=2x2 B=3x2"));
        assert!(diff.findings[1].detail.contains("셀 수: A=4 B=6"));
    }

    /// 표 셀 안 텍스트 변경 — 좌표가 행·열까지 내려간다.
    #[test]
    fn table_cell_text_change_carries_cell_coordinates() {
        let mut a = doc(&[&[]]);
        a.sections[0]
            .paragraphs
            .push(para_with_table(2, 2, &["A1", "A2", "B1", "B2"]));
        let mut b = doc(&[&[]]);
        b.sections[0]
            .paragraphs
            .push(para_with_table(2, 2, &["A1", "A2", "B1", "바뀜"]));

        let diff = diff_documents(&a, &b, &DiffOptions::default());

        assert_eq!(kinds(&diff), vec![FindingKind::TextChanged]);
        assert_eq!(
            paths(&diff),
            vec!["sec[0]/para[0]/ctrl[0]/cell[r1,c1]/para[0]"]
        );
        // 좌표에서 구역·문단을 문자열 파싱 없이 꺼낼 수 있어야 한다.
        assert_eq!(diff.findings[0].path.section(), Some(0));
        assert_eq!(diff.findings[0].path.paragraph(), Some(0));
    }

    /// 컨트롤 개수·종류 변화.
    #[test]
    fn control_count_and_kind_changes_are_detected() {
        let mut a = doc(&[&[]]);
        a.sections[0].paragraphs.push(Paragraph {
            controls: vec![Control::Bookmark(Default::default())],
            ..Default::default()
        });
        let mut b = doc(&[&[]]);
        b.sections[0].paragraphs.push(Paragraph {
            controls: vec![
                Control::PageHide(Default::default()),
                Control::Bookmark(Default::default()),
            ],
            ..Default::default()
        });

        let diff = diff_documents(&a, &b, &DiffOptions::default());

        assert_eq!(
            kinds(&diff),
            vec![
                FindingKind::ControlCountChanged,
                FindingKind::ControlKindChanged,
            ]
        );
        assert_eq!(paths(&diff)[1], "sec[0]/para[0]/ctrl[0]");
        assert!(diff.findings[1].detail.contains("A=bookmark B=pageHide"));
    }

    /// 문서 정보의 스타일 개수·정의 변화.
    #[test]
    fn style_count_and_definition_changes_are_detected() {
        let mut a = Document::default();
        a.doc_info.styles.push(Style {
            local_name: "바탕글".into(),
            para_shape_id: 0,
            ..Default::default()
        });
        let mut b = a.clone();
        b.doc_info.styles[0].para_shape_id = 7;
        b.doc_info.styles.push(Style {
            local_name: "제목".into(),
            ..Default::default()
        });

        let diff = diff_documents(&a, &b, &DiffOptions::default());

        assert_eq!(
            kinds(&diff),
            vec![FindingKind::StyleCountChanged, FindingKind::StyleChanged]
        );
        assert_eq!(paths(&diff), vec!["doc", "style[0]"]);
        assert!(diff.findings[1].detail.contains("para_shape_id: A=0 B=7"));
        assert_eq!(diff.findings[1].path.section(), None);
    }

    /// 결정성 — 같은 두 문서는 몇 번을 돌려도 완전히 같은 결과·같은 순서다.
    #[test]
    fn same_input_always_yields_same_output() {
        let mut a = doc(&[&["가", "나", "다", "라", "마"], &["별첨"]]);
        let mut b = doc(&[&["가", "낫", "다", "새 줄", "라", "마"], &["별첨2"]]);
        a.sections[0]
            .paragraphs
            .push(para_with_table(2, 2, &["A1", "A2", "B1", "B2"]));
        b.sections[0]
            .paragraphs
            .push(para_with_table(2, 2, &["A1", "A2", "B1", "달라짐"]));
        a.doc_info.styles.push(Style::default());
        b.doc_info.styles.push(Style {
            style_type: 1,
            ..Default::default()
        });

        let opts = DiffOptions::default();
        let first = diff_documents(&a, &b, &opts);
        let second = diff_documents(&a, &b, &opts);

        assert_eq!(first, second, "같은 두 문서는 같은 결과·같은 순서여야 한다");
        assert_eq!(first.summary(), second.summary());
        for _ in 0..10 {
            assert_eq!(diff_documents(&a, &b, &opts), first);
        }
        assert!(!first.identical);
    }

    /// 상한을 넘기면 조용히 자르지 않고 `truncated` 로 밝힌다.
    #[test]
    fn exceeding_max_findings_sets_truncated() {
        let a = doc(&[&["가", "나", "다", "라", "마"]]);
        let b = doc(&[&["가1", "나1", "다1", "라1", "마1"]]);

        let full = diff_documents(&a, &b, &DiffOptions::default());
        assert_eq!(full.findings.len(), 5);
        assert!(!full.truncated);

        let capped = diff_documents(&a, &b, &DiffOptions::default().max_findings(2));
        assert_eq!(capped.findings.len(), 2);
        assert!(capped.truncated, "버린 차이가 있으면 반드시 밝힌다");
        assert!(!capped.identical);
        // 잘린 앞 2 건은 무제한 결과의 앞 2 건과 같아야 한다(결정성).
        assert_eq!(capped.findings, full.findings[..2].to_vec());
        assert!(capped.summary().truncated);
    }

    /// 상한 0 — 차이를 전부 버려도 `identical` 로 거짓말하지 않는다.
    #[test]
    fn zero_max_findings_never_claims_identical() {
        let a = doc(&[&["가"]]);
        let b = doc(&[&["나"]]);
        let diff = diff_documents(&a, &b, &DiffOptions::default().max_findings(0));

        assert!(diff.findings.is_empty());
        assert!(diff.truncated);
        assert!(
            !diff.identical,
            "차이를 전부 버렸다고 '같다'가 되면 게이트가 무너진다"
        );

        // 반대로 정말 같은 문서는 상한 0 이어도 identical 이다(버릴 것이 없으므로).
        let same = diff_documents(&a, &a, &DiffOptions::default().max_findings(0));
        assert!(same.identical);
        assert!(!same.truncated);
    }

    /// 불변식 — `identical == true` 면 `findings` 는 비어 있고 자른 것도 없다.
    #[test]
    fn identical_implies_empty_findings_invariant() {
        let cases: [(Document, Document); 5] = [
            (doc(&[]), doc(&[])),
            (doc(&[&[]]), doc(&[&[]])),
            (doc(&[&["가"]]), doc(&[&["가"]])),
            (doc(&[&["가"]]), doc(&[&["나"]])),
            (doc(&[&["가"]]), doc(&[&["가"], &["나"]])),
        ];
        for (a, b) in &cases {
            for opts in [
                DiffOptions::default(),
                DiffOptions::default().max_findings(1),
                DiffOptions::default().max_findings(0),
                DiffOptions::default().ignore_whitespace(true),
            ] {
                let diff = diff_documents(a, b, &opts);
                if diff.identical {
                    assert!(diff.findings.is_empty(), "불변식: identical → 차이 없음");
                    assert!(!diff.truncated, "불변식: identical → 자른 것 없음");
                }
                assert_eq!(diff.summary().total, diff.findings.len());
            }
        }
    }

    /// 요약 집계 — 카테고리별 건수·총계·순서가 정확하다.
    #[test]
    fn summary_counts_each_category_exactly() {
        let a = doc(&[&["머리말", "가", "나", "다", "지울 줄", "꼬리말"]]);
        let b = doc(&[&["머리말", "가!", "나!", "다", "넣을 줄", "꼬리말"]]);
        let diff = diff_documents(&a, &b, &DiffOptions::default());
        let summary = diff.summary();

        assert_eq!(summary.total, diff.findings.len());
        assert_eq!(summary.count_of(FindingKind::TextChanged), 3);
        assert_eq!(summary.count_of(FindingKind::ParagraphAdded), 0);
        assert_eq!(summary.count_of(FindingKind::ParagraphRemoved), 0);
        assert_eq!(summary.count_of(FindingKind::SectionCountChanged), 0);
        assert!(!summary.truncated);

        // 합이 총계와 맞고, 0 건 카테고리는 목록에 없다.
        assert_eq!(
            summary.by_kind.iter().map(|(_, n)| *n).sum::<usize>(),
            summary.total
        );
        assert!(summary.by_kind.iter().all(|(_, n)| *n > 0));

        // 순서는 FindingKind 선언 순서 — 해시 순회에 기대지 않는다.
        let declared: Vec<_> = FindingKind::ALL
            .iter()
            .filter(|k| summary.count_of(**k) > 0)
            .copied()
            .collect();
        assert_eq!(
            summary.by_kind.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
            declared
        );
    }

    /// 꼬리에 문단이 붙으면 추가로만 센다.
    #[test]
    fn summary_separates_added_from_removed() {
        let a = doc(&[&["가", "나", "다"]]);
        let b = doc(&[&["가", "나", "다", "라", "마"]]);
        let summary = diff_documents(&a, &b, &DiffOptions::default()).summary();

        assert_eq!(summary.count_of(FindingKind::ParagraphAdded), 2);
        assert_eq!(summary.count_of(FindingKind::ParagraphRemoved), 0);
        assert_eq!(summary.total, 2);
    }

    /// 공백 무시 옵션 — 들여쓰기 차이는 삼키되 글자 차이는 계속 잡는다.
    #[test]
    fn ignore_whitespace_swallows_indentation_only() {
        let a = doc(&[&["  제1조   목적  ", "제2조"]]);
        let b = doc(&[&["제1조 목적", "제2조"]]);

        let strict = diff_documents(&a, &b, &DiffOptions::default());
        assert_eq!(kinds(&strict), vec![FindingKind::TextChanged]);

        let lenient = diff_documents(&a, &b, &DiffOptions::default().ignore_whitespace(true));
        assert!(lenient.identical, "공백만 다르면 차이가 아니다");

        let c = doc(&[&["제1조  범위", "제2조"]]);
        let still = diff_documents(&a, &c, &DiffOptions::default().ignore_whitespace(true));
        assert_eq!(kinds(&still), vec![FindingKind::TextChanged]);
    }

    /// 텍스트가 같아도 문단모양 참조가 바뀌면 잡는다.
    #[test]
    fn paragraph_shape_change_is_detected_without_text_change() {
        let a = doc(&[&["같은 글"]]);
        let mut b = a.clone();
        b.sections[0].paragraphs[0].para_shape_id = 3;

        let diff = diff_documents(&a, &b, &DiffOptions::default());

        assert_eq!(kinds(&diff), vec![FindingKind::ParagraphStyleChanged]);
        assert_eq!(paths(&diff), vec!["sec[0]/para[0]"]);
        assert!(diff.findings[0].detail.contains("para_shape_id: A=0 B=3"));
    }

    /// 좌표는 타입이다 — 문자열을 파싱하지 않고 구역·문단을 꺼낸다.
    #[test]
    fn node_path_is_readable_without_string_parsing() {
        let a = doc(&[&["가"], &["나", "다"]]);
        let b = doc(&[&["가"], &["나", "라"]]);
        let diff = diff_documents(&a, &b, &DiffOptions::default());

        let path = &diff.findings[0].path;
        assert_eq!(path.to_string(), "sec[1]/para[1]");
        assert_eq!(path.section(), Some(1));
        assert_eq!(path.paragraph(), Some(1));
        assert!(!path.is_root());
        assert_eq!(
            path.steps(),
            &[PathStep::Section(1), PathStep::Paragraph(1)]
        );
        assert!(NodePath::root().is_root());
        assert_eq!(NodePath::root().to_string(), "doc");
    }

    /// 직렬화 계층 — 봉투 JSON 의 키·값이 계약대로이고 문자열까지 결정적이다.
    #[test]
    fn json_envelope_is_stable() {
        let a = doc(&[&["가", "나"]]);
        let b = doc(&[&["가", "낫", "다"]]);
        let diff = diff_documents(&a, &b, &DiffOptions::default());

        let json = diff.to_json();
        assert_eq!(json["identical"], serde_json::json!(false));
        assert_eq!(json["truncated"], serde_json::json!(false));
        assert_eq!(json["findingCount"], serde_json::json!(2));
        assert_eq!(
            json["findings"][0]["kind"],
            serde_json::json!("textChanged")
        );
        assert_eq!(
            json["findings"][0]["path"],
            serde_json::json!("sec[0]/para[1]")
        );
        assert_eq!(
            json["summary"]["byKind"]["textChanged"],
            serde_json::json!(1)
        );
        assert_eq!(
            json["summary"]["byKind"]["paragraphAdded"],
            serde_json::json!(1)
        );

        let once = serde_json::to_string(&diff.to_json()).unwrap();
        let twice =
            serde_json::to_string(&diff_documents(&a, &b, &DiffOptions::default()).to_json())
                .unwrap();
        assert_eq!(once, twice);
    }

    /// 카테고리 이름은 봉투의 안정 키다 — 겹치거나 비어 있으면 안 된다.
    #[test]
    fn finding_kind_labels_are_unique() {
        let mut seen: Vec<&str> = FindingKind::ALL.iter().map(|k| k.label()).collect();
        assert_eq!(seen.len(), FindingKind::ALL.len());
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), FindingKind::ALL.len(), "이름이 겹치면 안 된다");
        assert!(FindingKind::ALL.iter().all(|k| !k.label().is_empty()));
        assert_eq!(FindingKind::TextChanged.to_string(), "textChanged");
    }

    /// 큰 문서 맨 앞에 한 문단을 끼워도 차이는 한 건뿐이다.
    #[test]
    fn head_insertion_in_large_document_stays_one_finding() {
        let body: Vec<String> = (0..200).map(|i| format!("본문 {}", i)).collect();
        let refs: Vec<&str> = body.iter().map(|s| s.as_str()).collect();
        let a = doc(&[&refs]);

        let mut with_head = vec!["새 머리말"];
        with_head.extend(refs.iter().copied());
        let b = doc(&[&with_head]);

        let diff = diff_documents(&a, &b, &DiffOptions::default());

        assert_eq!(
            kinds(&diff),
            vec![FindingKind::ParagraphAdded],
            "자리끼리 맞대는 방식이라면 201 건이 났을 것이다"
        );
        assert_eq!(paths(&diff), vec!["sec[0]/para[0]"]);
    }
}
