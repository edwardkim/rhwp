//! 칸 안 어울림(Square) 중첩 표 옆으로 흐르는 글은 저장 줄의 `horzpos`(column_start)
//! 에서 시작한다. 종전에는 칸 안 어울림 신호가 그림·도형만 보고 표를 빠뜨려, 글이
//! 칸 왼끝에서 시작해 중첩 표 위에 겹쳤다(여권 발급 신청서 '배우자의 로마자 성' 칸).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use std::io::{Cursor, Read, Write};

const BASE: &[u8] = include_bytes!("../../samples/task2169/empty_ladder.hwpx");

fn para(runs: &str, segs: &str) -> String {
    format!(
        r#"<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0">{runs}</hp:run><hp:linesegarray>{segs}</hp:linesegarray></hp:p>"#
    )
}

fn seg(vpos: i32, height: i32, column_start: i32, width: i32) -> String {
    format!(
        r#"<hp:lineseg textpos="0" vertpos="{vpos}" vertsize="{height}" textheight="{height}" baseline="{}" spacing="{}" horzpos="{column_start}" horzsize="{width}" flags="393216"/>"#,
        height * 85 / 100,
        height * 60 / 100
    )
}

fn cell(col: usize, width: i32, height: i32, content: &str) -> String {
    format!(
        r#"<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2"><hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">{content}</hp:subList><hp:cellAddr colAddr="{col}" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="{width}" height="{height}"/><hp:cellMargin left="0" right="0" top="0" bottom="0"/></hp:tc>"#
    )
}

fn table(
    id: u32,
    width: i32,
    height: i32,
    tac: bool,
    wrap: &str,
    cells: &str,
    cols: usize,
) -> String {
    let horz_offset = if tac { 0 } else { 100 };
    format!(
        r#"<hp:tbl id="{id}" zOrder="0" numberingType="TABLE" textWrap="{wrap}" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="1" colCnt="{cols}" cellSpacing="0" borderFillIDRef="2" noAdjust="0"><hp:sz width="{width}" widthRelTo="ABSOLUTE" height="{height}" heightRelTo="ABSOLUTE" protect="0"/><hp:pos treatAsChar="{}" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="{horz_offset}"/><hp:outMargin left="0" right="0" top="0" bottom="0"/><hp:inMargin left="0" right="0" top="0" bottom="0"/><hp:tr>{cells}</hp:tr></hp:tbl>"#,
        tac as u8
    )
}

/// 바깥 1칸 표의 칸 안에 [어울림 상자 표 + 공백] 문단과, 그 표 옆(cs=21000HU)으로
/// 흐르는 글 "메모" 문단을 둔다.
fn fixture(nested_wrap: &str) -> Vec<u8> {
    let boxes: String = (0..5)
        .map(|col| cell(col, 4000, 2000, &para("", "")))
        .collect();
    let nested = table(300, 20000, 2000, false, nested_wrap, &boxes, 5);
    let cell_body = para(
        &format!("{nested}<hp:t> </hp:t>"),
        &seg(0, 300, 21000, 27000),
    ) + &para("<hp:t>메모</hp:t>", &seg(300, 1000, 21000, 27000));
    let outer = table(
        200,
        48000,
        3000,
        true,
        "TOP_AND_BOTTOM",
        &cell(0, 48000, 3000, &cell_body),
        1,
    );
    let body = format!(
        r#"<hp:t></hp:t></hp:run><hp:linesegarray>{}</hp:linesegarray></hp:p>
<hp:p id="2" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0">{outer}<hp:t/></hp:run><hp:linesegarray>{}</hp:linesegarray></hp:p></hs:sec>"#,
        seg(0, 1000, 0, 48188),
        seg(1600, 3000, 0, 48188)
    );

    let mut source = zip::ZipArchive::new(Cursor::new(BASE)).expect("base ZIP");
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).expect("entry");
        let name = entry.name().expect("name").into_owned();
        if name != "Contents/section0.xml" {
            output.raw_copy_file(entry).expect("copy");
            continue;
        }
        let mut xml = String::new();
        entry.read_to_string(&mut xml).expect("section XML");
        let head_end = xml.find("<hp:bookmark").expect("secPr paragraph");
        let section = format!("{}{body}", &xml[..head_end]);
        output
            .start_file(name, zip::write::SimpleFileOptions::default())
            .expect("section entry");
        output.write_all(section.as_bytes()).expect("section");
    }
    output.finish().expect("finish").into_inner()
}

fn attr(tag: &str, name: &str) -> Option<f64> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = start + tag[start..].find('"')?;
    tag[start..end].parse().ok()
}

fn text_x(svg: &str, ch: &str) -> f64 {
    svg.split("<text ")
        .skip(1)
        .find(|tag| {
            tag.split("</text>")
                .next()
                .is_some_and(|run| run.contains(ch))
        })
        // 태그 첫 속성도 찾도록 앞에 공백을 붙인다.
        .and_then(|tag| attr(&format!(" {tag}"), "x"))
        .expect("text run")
}

#[test]
fn text_beside_square_nested_table_starts_at_stored_column_start() {
    let core = DocumentCore::from_bytes(&fixture("SQUARE")).expect("fixture");
    let svg = core.render_page_svg_native(0).expect("svg");
    // 칸 안쪽 왼끝(본문 왼끝) + cs 21000HU(280px). 상자 표 오른끝은 왼끝 + 약 268px.
    let body_left = 75.58;
    let x = text_x(&svg, "메");
    assert!(
        (x - (body_left + 280.0)).abs() < 1.0,
        "글이 저장 column_start 가 아닌 x={x:.1} 에서 시작했다"
    );
}
