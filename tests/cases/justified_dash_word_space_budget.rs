//! 검사 유형: 고정형(독립 Print의 글자·리더 원점 ±2 CSSpx), 관계형(자연 대시·공백 축소 한도·소유 유일성).
//! 주석 갱신: 2026-11-09T05:04:04+09:00; 신규 파일로 직전 파일 변경 커밋 없음.
//! 2026-11-09 canonical raw 2px, 96 DPI, compat2022; 독립 한컴2020 Print의 Linux PDFium PNG를 SHA 검증 후 재사용.
//! issue3307_outline_number.hwpx 전 9쪽 Native/fresh WASM 모두 최저 100.00%, 쪽 누락·중복·보호 표시값 하락 0.
//! 고정하는 8쪽도 두 경로 모두 100.00%; 같은 쪽 review·overlay와 원점·미세 간격 확대 판독 후 등록했다.
//! 실행은 설치 글꼴을 전제하지 않는다. 시각 측정의 원본 HFT 공급 환경과 모델 회귀의 실행 조건을 구분한다.
//! 근거: mydocs/pr/assets/planet6897_20261106/justified-dash-space-budget/validation.json.

use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{BoundingBox, RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};

fn owned_runs() -> Vec<(BoundingBox, TextRunNode)> {
    fn collect(node: &RenderNode, out: &mut Vec<(BoundingBox, TextRunNode)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.section_index == Some(1)
                && run.cell_context.as_ref().is_some_and(|context| {
                    context.parent_para_index == 40
                        && context.path.len() == 1
                        && context.path[0].cell_index == 11
                        && context.path[0].cell_para_index == 0
                })
            {
                out.push((node.bbox, run.clone()));
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let mut core = DocumentCore::from_bytes(
        &std::fs::read("samples/task3307/issue3307_outline_number.hwpx").unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(false);
    let tree = core.build_page_render_tree(7).unwrap();
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    runs
}

fn unique_run<'a>(
    runs: &'a [(BoundingBox, TextRunNode)],
    text: &str,
) -> &'a (BoundingBox, TextRunNode) {
    let found: Vec<_> = runs.iter().filter(|(_, run)| run.text == text).collect();
    assert_eq!(
        found.len(),
        1,
        "원본 셀·문단의 텍스트 소유·누락·중복: {text}"
    );
    found[0]
}

fn positions(run: &TextRunNode) -> Vec<f64> {
    run.layout_positions
        .clone()
        .unwrap_or_else(|| EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style))
}

#[test]
fn justified_words_and_leader_keep_independent_print_origins() {
    let runs = owned_runs();
    let (leader_box, leader) = unique_run(&runs, " -----");
    let first_dash = leader_box.x + positions(leader)[1];
    assert!(
        (first_dash - 591.6786702473959).abs() <= 2.0,
        "독립 Print의 첫 대시 원점: {first_dash}"
    );
    let (word_box, word) = unique_run(&runs, "에 따른 소극행정");
    let last = word.text.chars().position(|ch| ch == '정').unwrap();
    let last_origin = word_box.x + positions(word)[last];
    assert!(
        (last_origin - 564.9746907552084).abs() <= 2.0,
        "독립 Print의 마지막 낱말 글자 원점: {last_origin}"
    );
}

#[test]
fn natural_leader_uses_only_the_existing_word_space_budget() {
    let runs = owned_runs();
    let (_, leader) = unique_run(&runs, " -----");
    let actual = positions(leader);
    let mut natural = leader.style.clone();
    natural.extra_word_spacing = 0.0;
    natural.extra_dash_advance = 0.0;
    let dash = EmbeddedTextMeasurer.compute_char_positions("-", &natural)[1];
    let space = EmbeddedTextMeasurer.compute_char_positions(" ", &natural)[1];
    // 원문 HU 장치 전진의 한 양자다. 실물 좌표의 허용치를 완화하는 값이 아니다.
    let quantum = 1.0 / 75.0;
    for index in 1..actual.len() - 1 {
        let advance = actual[index + 1] - actual[index];
        assert!(
            (advance - dash).abs() <= quantum,
            "각 대시는 해당 글자 모양의 자연 폭까지만 복원한다: {advance} / {dash}"
        );
    }
    let compressed = actual[1] - actual[0];
    assert!(
        compressed < space,
        "자연 대시의 부족은 낱말 공백에서 소비한다"
    );
    assert!(
        compressed >= space * 0.8 - quantum,
        "기존 양쪽 정렬 공백 20% 한도를 넘지 않는다: {compressed} / {space}"
    );
}
