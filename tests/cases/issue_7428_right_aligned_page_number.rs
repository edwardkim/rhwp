//! #7428: 우측 정렬 머리말의 자동 쪽번호가 **한 글자 전진폭**만큼 오른쪽으로 밀리면 안 된다.
//!
//! # 원인 (b042224df 에서 해소, 이 검사로 고정한다)
//!
//! 바탕쪽 표 칸의 쪽번호 자리는 모델에서 **공백 한 칸**이다. 표시값(`display_text`)은
//! 숫자지만, 우측 정렬의 말미 공백 제외(`trailing_space_width_after_last_inline_object`)가
//! 모델 문자열 `run.text` 를 보던 시절에는 그 공백을 말미 공백으로 세어 정렬 폭에서
//! 뺐다. 그래서 번호가 공백 전진폭(글꼴 41.7px 의 반각 20.9px)만큼 칸 우단 밖으로
//! 나갔다. 정렬 폭 추정(`estimate_line_run_widths`)은 이미 표시 글자를 쓰고 있었으므로
//! 측정과 말미 공백 제외가 **같은 표시 문자열**을 소비해야 한다.
//!
//! # 기대값
//!
//! 독립 기준은 한컴 출력 `pdf/exam_kor-2022.pdf`(Producer `Hancom PDF 1.3.0.550`)다.
//! `mutool draw -F stext` 의 머리말 쪽번호 글자 원점(96dpi px):
//!
//! ```text
//!   홀수쪽(우측 정렬)  '1'·'3'… x=978.4, 숫자 전진폭 26.9 -> 오른끝 1005.3
//!   11쪽(두 자리)       '1' 951.5, '1' 978.4              -> 오른끝 1005.3
//!   짝수쪽(좌측 정렬)  '2'·'10'·'12' 첫 글자 x=117.1
//! ```
//!
//! 우측 정렬 번호의 오른끝은 칸 오른쪽(1005.4)과, 좌측 정렬 번호의 시작은 칸 왼쪽(117.2)과
//! 일치한다. 그래서 절대 좌표가 아니라 **칸 경계와의 관계**로 검사한다. 수정 전 코드(말미 공백을
//! `run.text` 로 세던 줄로 되돌린 빌드)에서 이 검사는 3쪽 오른끝 1024.4 / 칸 1005.4(+19.0px)로
//! 실패한다. `exam_eng` 3·5·7쪽은 같은 원인으로 +20.9px 였다.
//!
//! 반례: 두 자리 번호(11쪽), 좌측 정렬 번호(짝수쪽), 같은 머리말의 우측 정렬 리터럴은
//! 글상자 안이라 이 검사의 대상이 아니다(#7428 실측에서 제자리).
//!
//! `exam_eng` 를 쓰지 않는 것은 그 문서의 Native Visual Sweep 이 90% 미만 쪽을 가지기
//! 때문이다(세로 축의 별개 결함). `exam_kor` 는 20쪽 전부 90% 이상이다.
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const TOLERANCE_PX: f64 = 1.0;

struct Number {
    text: String,
    run: BoundingBox,
    cell: BoundingBox,
}

/// 바탕쪽 아래 표 칸에 놓인, 숫자로만 표시되는 글자열 run 과 그 칸을 모은다.
fn collect(node: &RenderNode, in_master: bool, cell: Option<&BoundingBox>, out: &mut Vec<Number>) {
    let in_master = in_master || matches!(node.node_type, RenderNodeType::MasterPage);
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(&node.bbox)
    } else {
        cell
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let shown = run.display_text.as_deref().unwrap_or(&run.text);
        if in_master
            && !shown.is_empty()
            && shown.chars().all(|c| c.is_ascii_digit())
            && run.style.font_size > 35.0
        {
            if let Some(cell) = cell {
                out.push(Number {
                    text: shown.to_string(),
                    run: node.bbox,
                    cell: *cell,
                });
            }
        }
    }
    for child in &node.children {
        collect(child, in_master, cell, out);
    }
}

fn page_number(doc: &DocumentCore, page: u32) -> Number {
    let tree = doc.build_page_render_tree(page - 1).expect("쪽 렌더 트리");
    let mut found = Vec::new();
    collect(&tree.root, false, None, &mut found);
    assert_eq!(
        found.len(),
        1,
        "{page}쪽 바탕쪽 표 칸의 쪽번호는 정확히 한 run 이어야 한다"
    );
    let number = found.pop().unwrap();
    assert_eq!(number.text, page.to_string(), "{page}쪽 쪽번호 표시값");
    number
}

fn open(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("samples/{name}"));
    DocumentCore::from_bytes(&std::fs::read(path).expect("공개 회귀 문서")).expect("문서 파싱")
}

#[test]
fn right_aligned_master_page_number_ends_at_cell_right_edge() {
    let doc = open("exam_kor.hwp");
    // 홀수쪽은 우측 정렬. 11쪽은 두 자리 번호다. 1쪽 번호는 본문 표 안 글상자라 대상이 아니다.
    for page in [3u32, 5, 9, 11] {
        let n = page_number(&doc, page);
        let run_right = n.run.x + n.run.width;
        let cell_right = n.cell.x + n.cell.width;
        assert!(
            (run_right - cell_right).abs() <= TOLERANCE_PX,
            "{page}쪽 우측 정렬 쪽번호 오른끝 {run_right:.1} 이 칸 오른쪽 {cell_right:.1} 과 \
             어긋난다(공백 한 칸만큼 오른쪽이면 #7428 재발: 표시 숫자가 아니라 모델 공백을 말미 공백으로 뺐다)"
        );
    }
}

#[test]
fn left_aligned_master_page_number_starts_at_cell_left_edge() {
    let doc = open("exam_kor.hwp");
    // 짝수쪽은 좌측 정렬 — 말미 공백 제외 경로를 타지 않는 대조군. 10·12쪽은 두 자리다.
    for page in [2u32, 10, 12] {
        let n = page_number(&doc, page);
        assert!(
            (n.run.x - n.cell.x).abs() <= TOLERANCE_PX,
            "{page}쪽 좌측 정렬 쪽번호 시작 {:.1} 이 칸 왼쪽 {:.1} 과 어긋난다",
            n.run.x,
            n.cell.x
        );
    }
}
