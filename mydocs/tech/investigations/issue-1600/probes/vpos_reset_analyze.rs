//! 경로 A 1단계: LINE_SEG vpos-reset 신뢰도 분석 (Task #1618).
//!
//! flow 순서로 line_seg 를 훑어 vpos 역행 점프(=한글 페이지/단 경계)를 세고 vpos-예측
//! 페이지수를 산출. rhwp page_count 와 비교. (한글 정답 비교는 호출측 TSV join.)
//!
//! 사용: cargo run --release --example vpos_reset_analyze -- <file> [--verbose]
//!   또는 --batch <tsv> : tsv 의 rel 컬럼(첫 컬럼) 각 파일 분석, 한 줄 요약 출력.
//!
//! 출력(파일당, TSV): rel  rhwp_pages  vpos_pred  resets  tables  partial_table_ctrls  big_tables  sections

use rhwp::model::control::Control;
use std::env;
use std::fs;

/// 빈(텍스트 미배열) 세그먼트 비트 — PARA_LINE_SEG tag bit 16.
fn is_empty_seg(tag: u32) -> bool {
    tag & (1 << 16) != 0
}

struct Stats {
    rhwp_pages: usize,
    vpos_pred: usize,
    resets: usize,
    tables: usize,
    big_tables: usize, // 선언높이 > 0.5 page(~37000 HU) 표 (분할 후보)
    sections: usize,
}

fn analyze(data: &[u8]) -> Option<Stats> {
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(data).ok()?;
    let document = doc.document();
    let mut resets = 0usize;
    let mut tables = 0usize;
    let mut big_tables = 0usize;
    let sections = document.sections.len();

    // 전 문서 flow 순서로 line_seg vpos 를 훑으며 역행 점프 카운트.
    // 섹션 경계는 새 페이지로 간주(대부분) → 섹션마다 prev 리셋.
    for section in &document.sections {
        let mut prev_vpos: Option<i32> = None;
        for para in &section.paragraphs {
            // 표/그림 컨트롤 카운트(분할 가능성 플래그)
            for ctrl in &para.controls {
                if let Control::Table(t) = ctrl {
                    tables += 1;
                    // 선언높이가 반 페이지 초과면 페이지 분할 후보(vpos 리셋 미포착 위험)
                    if t.common.height > 37000 {
                        big_tables += 1;
                    }
                }
            }
            for ls in &para.line_segs {
                if is_empty_seg(ls.tag) {
                    continue;
                }
                if let Some(pv) = prev_vpos {
                    // 역행 점프(현재 vpos 가 직전보다 충분히 작음) = 페이지/단 경계
                    if ls.vertical_pos + 2000 < pv {
                        resets += 1;
                    }
                }
                prev_vpos = Some(ls.vertical_pos);
            }
        }
    }
    // vpos-예측 페이지: 역행 점프 수 + 1 (전체 flow 기준).
    let vpos_pred = resets + 1;
    Some(Stats {
        rhwp_pages: doc.page_count() as usize,
        vpos_pred,
        resets,
        tables,
        big_tables,
        sections,
    })
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("사용: cargo run --release --example vpos_reset_analyze -- <file|--batch tsv> [--root DIR]");
        std::process::exit(1);
    }

    if args[0] == "--batch" {
        let tsv = &args[1];
        let root = args
            .windows(2)
            .find(|w| w[0] == "--root")
            .map(|w| w[1].clone())
            .unwrap_or_else(|| "C:/Users/planet/hwpdocs".to_string());
        let content = fs::read_to_string(tsv).expect("read tsv");
        println!("rel\trhwp_pages\tvpos_pred\tresets\ttables\tbig_tables\tsections");
        for (i, line) in content.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue; // header
            }
            let rel = line.split('\t').next().unwrap_or("").trim();
            if rel.is_empty() {
                continue;
            }
            let path = format!("{}/{}", root, rel);
            match fs::read(&path) {
                Ok(data) => match analyze(&data) {
                    Some(s) => println!(
                        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                        rel,
                        s.rhwp_pages,
                        s.vpos_pred,
                        s.resets,
                        s.tables,
                        s.big_tables,
                        s.sections
                    ),
                    None => println!("{}\tERR\t-\t-\t-\t-\t-", rel),
                },
                Err(_) => println!("{}\tREAD_ERR\t-\t-\t-\t-\t-", rel),
            }
        }
    } else {
        let data = fs::read(&args[0]).expect("read");
        match analyze(&data) {
            Some(s) => println!(
                "{}: rhwp_pages={} vpos_pred={} resets={} tables={} big_tables={} sections={}",
                args[0], s.rhwp_pages, s.vpos_pred, s.resets, s.tables, s.big_tables, s.sections
            ),
            None => eprintln!("파싱 실패"),
        }
    }
}
