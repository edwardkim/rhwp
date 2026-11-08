//! 독립 한컴 Print 6쪽과 대조한 양쪽 정렬의 줄 가장자리 공백 계약.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::DocumentCore;
use serde_json::Value;

const INPUT: &[u8] =
    include_bytes!("../fixtures/justified_edge_spaces/leading-tracking-contract.hwpx");

struct FirstLine {
    characters: Vec<(char, f64)>,
    end: f64,
}

fn layouts() -> (rhwp::model::document::Document, Vec<Value>) {
    let source = rhwp::parser::parse_document(INPUT).expect("시각 검증한 원본 파싱");
    let core = DocumentCore::from_bytes(INPUT).expect("같은 원본의 공개 배치");
    assert_eq!(core.page_count(), 6, "독립 Print의 전쪽 수");
    let pages = (0..core.page_count())
        .map(|page| {
            serde_json::from_str(&core.get_page_text_layout_native(page as u32).unwrap()).unwrap()
        })
        .collect();
    (source, pages)
}

fn paragraph_runs(page: &Value, paragraph: usize) -> Vec<&Value> {
    page["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| run["secIdx"] == 0 && run["paraIdx"] == paragraph)
        .collect()
}

fn first_line(page: &Value, paragraph: usize) -> FirstLine {
    let runs = paragraph_runs(page, paragraph);
    let first_y = runs.first().expect("문단의 첫 줄")["y"].as_f64().unwrap();
    let mut characters = Vec::new();
    let mut end = 0.0;
    for run in runs
        .into_iter()
        .take_while(|run| run["y"].as_f64() == Some(first_y))
    {
        let x = run["x"].as_f64().unwrap();
        let positions = run["charX"].as_array().unwrap();
        let text = run["text"].as_str().unwrap();
        assert_eq!(positions.len(), text.chars().count() + 1);
        characters.extend(
            text.chars()
                .zip(positions)
                .map(|(character, position)| (character, x + position.as_f64().unwrap())),
        );
        end = x + positions.last().unwrap().as_f64().unwrap();
    }
    FirstLine { characters, end }
}

#[test]
fn print_verified_edge_spaces_preserve_source_order_and_page_ownership() {
    let (source, pages) = layouts();
    let paragraphs = &source.sections[0].paragraphs;
    assert_eq!(paragraphs.len(), 12, "쪽마다 Left/Justify 원문 한 쌍");
    for (page_index, page) in pages.iter().enumerate() {
        let runs = page["runs"].as_array().unwrap();
        assert!(!runs.is_empty());
        assert!(
            runs.iter().all(|run| {
                run["secIdx"] == 0
                    && run["paraIdx"].as_u64().is_some_and(|owner| {
                        (page_index * 2..page_index * 2 + 2).contains(&(owner as usize))
                    })
            }),
            "원본의 명시적 쪽 나눔과 문단 소유"
        );
        for paragraph in page_index * 2..page_index * 2 + 2 {
            let text = &paragraphs[paragraph].text;
            assert!(text.is_ascii(), "이 입력의 문자 인덱스는 UTF-16과 같다");
            let mut end = 0;
            let owned = paragraph_runs(page, paragraph);
            assert!(owned.len() > 1, "실제 자동 줄바꿈을 통과한다");
            let mut prior_y = f64::NEG_INFINITY;
            for run in owned {
                let start = run["charStart"].as_u64().unwrap() as usize;
                let value = run["text"].as_str().unwrap();
                assert!(start >= end, "문자의 중복과 순서 역전 방지");
                assert!(
                    text[end..start].chars().all(char::is_whitespace),
                    "내용 누락 방지"
                );
                let next = start + value.len();
                assert_eq!(&text[start..next], value, "런은 같은 원문 범위를 소유한다");
                let y = run["y"].as_f64().unwrap();
                assert!(y >= prior_y, "줄은 내용 순서대로 배치한다");
                prior_y = y;
                end = next;
            }
            assert!(
                text[end..].chars().all(char::is_whitespace),
                "마지막 줄의 내용 보존"
            );
        }
    }
}

#[test]
fn print_verified_justification_distributes_only_between_visible_words() {
    let (source, pages) = layouts();
    for (page_index, page) in pages.iter().enumerate() {
        let paragraph = page_index * 2;
        assert_eq!(
            source.sections[0].paragraphs[paragraph].text,
            source.sections[0].paragraphs[paragraph + 1].text
        );
        let plain = first_line(page, paragraph);
        let justified = first_line(page, paragraph + 1);
        let text = |line: &FirstLine| {
            line.characters
                .iter()
                .map(|value| value.0)
                .collect::<String>()
        };
        assert_eq!(
            text(&plain),
            text(&justified),
            "정렬에 무관한 첫 줄의 내용 소유"
        );
        let leading = plain
            .characters
            .iter()
            .position(|value| !value.0.is_whitespace())
            .unwrap();
        for index in 0..=leading {
            let plain_advance = plain.characters[index].1 - plain.characters[0].1;
            let justified_advance = justified.characters[index].1 - justified.characters[0].1;
            // 공개 JSON의 원점과 문자 전진은 각각 소수 첫째 자리로 직렬화된다.
            assert!(
                (plain_advance - justified_advance).abs() <= 0.21,
                "선두 공백은 자연 전진 유지: page={}",
                page_index + 1
            );
        }
        let tail = plain
            .characters
            .iter()
            .rposition(|value| !value.0.is_whitespace())
            .unwrap()
            + 1;
        assert!(
            tail < plain.characters.len(),
            "자동 줄바꿈이 소비한 구분 공백"
        );
        let natural_tail = plain.end - plain.characters[tail].1;
        let justified_tail = justified.end - justified.characters[tail].1;
        assert!(natural_tail > 0.0);
        assert!((natural_tail - justified_tail).abs() <= 0.21,
                "말미 구분자는 내부 배분에 참여하지 않는다: page={}, Left={natural_tail}, Justify={justified_tail}",
                page_index + 1);
        let inside = (leading + 1..tail)
            .find(|index| plain.characters[*index].0 == ' ')
            .expect("내부 단어 사이 공백");
        let plain_gap = plain.characters[inside + 1].1 - plain.characters[inside].1;
        let justified_gap = justified.characters[inside + 1].1 - justified.characters[inside].1;
        assert!(
            justified_gap > plain_gap,
            "가장자리와 달리 내부 공백은 배분된다"
        );
    }
}

#[test]
fn print_verified_completed_pages_keep_leading_space_origins_within_two_pixels() {
    let (_, pages) = layouts();
    // 독립 Print의 CTM × 텍스트 행렬에서 얻은 원점에 96/72를 곱한다.
    // 99%대인 p5·p6을 우선하고, 자간 0% 경계는 기존 p2로 함께 보호한다.
    // 전쪽 90% 이상, 선정 쪽 양쪽 98% 이상 및 확대 판독 근거는 fixture README 참조.
    for (page_index, expected_x) in [(4, 72.669056), (5, 84.353728), (1, 73.469376)] {
        for paragraph in page_index * 2..page_index * 2 + 2 {
            let line = first_line(&pages[page_index], paragraph);
            let first_visible = line
                .characters
                .iter()
                .find(|value| !value.0.is_whitespace())
                .unwrap();
            assert_eq!(first_visible.0, 'a');
            assert!(
                (first_visible.1 - expected_x).abs() <= 2.0,
                "독립 Print의 선두 공백 다음 글자 원점: paragraph={paragraph}, actual={}",
                first_visible.1
            );
        }
    }
}
