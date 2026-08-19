//! rhwp-q-pack — 다음 묶음 조회 CLI 50개 + Control 필드 조회 코드.
//!
//! 선점: claimed_pack_next. rhwp-agent / rhwp-q-* / rhwp-q-kit 과 이름 중복 없음.

mod envelope;
mod inventory;
mod gen {
    mod s00;
    mod s01;
    mod s02;
    mod s03;
    mod s04;
    mod s05;
    mod s06;
    mod s07;
    mod s08;
    mod s09;
    mod s10;
    mod s11;
    mod s12;
    mod s13;
    mod s14;
    mod s15;
    mod s16;
    mod s17;
    mod s18;
    mod s19;
    mod s20;
    mod s21;
    mod s22;
    mod s23;
    mod s24;
    mod s25;
    mod s26;
    mod s27;
    mod s28;
    mod s29;
    mod s30;
    mod s31;
    mod s32;
    mod s33;
    mod s34;
    mod s35;
    mod s36;
    mod s37;
    mod s38;
    mod s39;
    mod s40;
    mod s41;
    mod s42;
    mod s43;
    mod s44;
    mod s45;
    mod s46;
    mod s47;
    mod s48;
    mod s49;

    pub fn probe_slot(slot: u32, doc: &rhwp::model::document::Document) -> u64 {
        const FNS: [fn(&rhwp::model::document::Document) -> u64; 50] = [
            s00::probe,
            s01::probe,
            s02::probe,
            s03::probe,
            s04::probe,
            s05::probe,
            s06::probe,
            s07::probe,
            s08::probe,
            s09::probe,
            s10::probe,
            s11::probe,
            s12::probe,
            s13::probe,
            s14::probe,
            s15::probe,
            s16::probe,
            s17::probe,
            s18::probe,
            s19::probe,
            s20::probe,
            s21::probe,
            s22::probe,
            s23::probe,
            s24::probe,
            s25::probe,
            s26::probe,
            s27::probe,
            s28::probe,
            s29::probe,
            s30::probe,
            s31::probe,
            s32::probe,
            s33::probe,
            s34::probe,
            s35::probe,
            s36::probe,
            s37::probe,
            s38::probe,
            s39::probe,
            s40::probe,
            s41::probe,
            s42::probe,
            s43::probe,
            s44::probe,
            s45::probe,
            s46::probe,
            s47::probe,
            s48::probe,
            s49::probe,
        ];
        FNS[slot as usize](doc)
    }
}

use envelope::{envelope, load_core, parse_slot, print_json, write_stdout, EXIT_OK, EXIT_USAGE};
use serde_json::json;

fn print_help() {
    let _ = write_stdout(&format!(
        "rhwp-q-pack v{} — 조회 CLI 50개 + volume-probe",
        rhwp::version()
    ));
    let _ = write_stdout("사용법: rhwp-q-pack <명령> [옵션]");
    let _ = write_stdout("  rhwp-q-pack forms-all <파일> [--json]  양식 개체 전수\n  rhwp-q-pack shapes-all <파일> [--json]  그리기 개체 전수\n  rhwp-q-pack char-overlaps <파일> [--json]  글자겹침 전수\n  rhwp-q-pack headers-list <파일> [--json]  머리말 전수\n  rhwp-q-pack footers-list <파일> [--json]  꼬리말 전수\n  rhwp-q-pack footnotes-list <파일> [--json]  각주 전수\n  rhwp-q-pack endnotes-list <파일> [--json]  미주 전수\n  rhwp-q-pack new-numbers <파일> [--json]  새 번호 전수\n  rhwp-q-pack page-num-ctrls <파일> [--json]  쪽번호 시작 전수\n  rhwp-q-pack page-number-pos <파일> [--json]  쪽번호 위치 전수\n  rhwp-q-pack column-defs <파일> [--json]  단 정의 전수\n  rhwp-q-pack unknown-ctrls <파일> [--json]  미지 컨트롤 전수\n  rhwp-q-pack tables-model <파일> [--json]  표 모델 행·열\n  rhwp-q-pack field-ctrls <파일> [--json]  필드 컨트롤 전수\n  rhwp-q-pack bookmark-names <파일> [--json]  책갈피 이름\n  rhwp-q-pack treat-as-char <파일> [--json]  글자처럼 취급 개체\n  rhwp-q-pack logical-inline <파일> [--json]  논리 인라인 컨트롤\n  rhwp-q-pack picture-crops <파일> [--json]  그림 자르기 좌표\n  rhwp-q-pack equation-scripts <파일> [--json]  수식 스크립트\n  rhwp-q-pack form-types <파일> [--json]  양식 타입\n  rhwp-q-pack hyperlink-hosts <파일> [--json]  하이퍼링크 URL 길이\n  rhwp-q-pack ruby-mains <파일> [--json]  덧말 본문\n  rhwp-q-pack pagehide-headers <파일> [--json]  머리말 감추기\n  rhwp-q-pack autonumber-nums <파일> [--json]  자동번호 값\n  rhwp-q-pack index-second-keys <파일> [--json]  찾아보기 둘째 키\n  rhwp-q-pack hidden-comment-len <파일> [--json]  숨은 설명 문단 수\n  rhwp-q-pack table-rows <파일> [--json]  표 행 수\n  rhwp-q-pack table-cells <파일> [--json]  표 셀 수\n  rhwp-q-pack shape-sizes <파일> [--json]  그리기 폭\n  rhwp-q-pack header-paras <파일> [--json]  머리말 문단 수\n  rhwp-q-pack footer-paras <파일> [--json]  꼬리말 문단 수\n  rhwp-q-pack footnote-paras <파일> [--json]  각주 문단 수\n  rhwp-q-pack endnote-paras <파일> [--json]  미주 문단 수\n  rhwp-q-pack picture-locks <파일> [--json]  그림 잠금\n  rhwp-q-pack picture-reverse <파일> [--json]  그림 반전\n  rhwp-q-pack equation-fonts <파일> [--json]  수식 글꼴\n  rhwp-q-pack form-enabled <파일> [--json]  양식 활성\n  rhwp-q-pack field-commands <파일> [--json]  필드 command\n  rhwp-q-pack field-ids <파일> [--json]  필드 id\n  rhwp-q-pack form-sizes <파일> [--json]  양식 너비\n  rhwp-q-pack section-defs <파일> [--json]  구역 정의 개수\n  rhwp-q-pack caption-tables <파일> [--json]  표 캡션 유무\n  rhwp-q-pack ctrl-kinds <파일> [--json]  컨트롤 종류 개수\n  rhwp-q-pack page-starts-on <파일> [--json]  쪽 시작\n  rhwp-q-pack hidden-comment-count <파일> [--json]  숨은 설명 개수\n  rhwp-q-pack ruby-ratio <파일> [--json]  덧말 비율\n  rhwp-q-pack char-overlap-len <파일> [--json]  겹침 글자 수\n  rhwp-q-pack table-cols <파일> [--json]  표 열 수\n  rhwp-q-pack picture-instance <파일> [--json]  그림 instance id\n  rhwp-q-pack index-first-keys <파일> [--json]  찾아보기 첫째 키");
    let _ = write_stdout("  rhwp-q-pack volume-probe <파일> --slot <0-49> [--json]");
}

fn volume_probe(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-pack volume-probe <파일> --slot <0-49> [--json]";
    let (path, json_mode, slot) = match parse_slot(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let acc = gen::probe_slot(slot, core.document());
    let payload = json!({"source": path, "slot": slot, "acc": acc});
    if json_mode {
        print_json(&envelope("volume-probe", payload, &[]))
    } else {
        write_stdout(&format!("slot={slot} acc={acc}"))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        None | Some("--help") | Some("-h") => {
            if args.is_empty() {
                eprintln!("오류: 명령을 지정해주세요.");
                std::process::exit(EXIT_USAGE);
            }
            print_help();
            0
        }
        Some("volume-probe") => volume_probe(&args[1..]),
        Some("forms-all") => inventory::forms_all(&args[1..]),
        Some("shapes-all") => inventory::shapes_all(&args[1..]),
        Some("char-overlaps") => inventory::char_overlaps(&args[1..]),
        Some("headers-list") => inventory::headers_list(&args[1..]),
        Some("footers-list") => inventory::footers_list(&args[1..]),
        Some("footnotes-list") => inventory::footnotes_list(&args[1..]),
        Some("endnotes-list") => inventory::endnotes_list(&args[1..]),
        Some("new-numbers") => inventory::new_numbers(&args[1..]),
        Some("page-num-ctrls") => inventory::page_num_ctrls(&args[1..]),
        Some("page-number-pos") => inventory::page_number_pos(&args[1..]),
        Some("column-defs") => inventory::column_defs(&args[1..]),
        Some("unknown-ctrls") => inventory::unknown_ctrls(&args[1..]),
        Some("tables-model") => inventory::tables_model(&args[1..]),
        Some("field-ctrls") => inventory::field_ctrls(&args[1..]),
        Some("bookmark-names") => inventory::bookmark_names(&args[1..]),
        Some("treat-as-char") => inventory::treat_as_char(&args[1..]),
        Some("logical-inline") => inventory::logical_inline(&args[1..]),
        Some("picture-crops") => inventory::picture_crops(&args[1..]),
        Some("equation-scripts") => inventory::equation_scripts(&args[1..]),
        Some("form-types") => inventory::form_types(&args[1..]),
        Some("hyperlink-hosts") => inventory::hyperlink_hosts(&args[1..]),
        Some("ruby-mains") => inventory::ruby_mains(&args[1..]),
        Some("pagehide-headers") => inventory::pagehide_headers(&args[1..]),
        Some("autonumber-nums") => inventory::autonumber_nums(&args[1..]),
        Some("index-second-keys") => inventory::index_second_keys(&args[1..]),
        Some("hidden-comment-len") => inventory::hidden_comment_len(&args[1..]),
        Some("table-rows") => inventory::table_rows(&args[1..]),
        Some("table-cells") => inventory::table_cells(&args[1..]),
        Some("shape-sizes") => inventory::shape_sizes(&args[1..]),
        Some("header-paras") => inventory::header_paras(&args[1..]),
        Some("footer-paras") => inventory::footer_paras(&args[1..]),
        Some("footnote-paras") => inventory::footnote_paras(&args[1..]),
        Some("endnote-paras") => inventory::endnote_paras(&args[1..]),
        Some("picture-locks") => inventory::picture_locks(&args[1..]),
        Some("picture-reverse") => inventory::picture_reverse(&args[1..]),
        Some("equation-fonts") => inventory::equation_fonts(&args[1..]),
        Some("form-enabled") => inventory::form_enabled(&args[1..]),
        Some("field-commands") => inventory::field_commands(&args[1..]),
        Some("field-ids") => inventory::field_ids(&args[1..]),
        Some("form-sizes") => inventory::form_sizes(&args[1..]),
        Some("section-defs") => inventory::section_defs(&args[1..]),
        Some("caption-tables") => inventory::caption_tables(&args[1..]),
        Some("ctrl-kinds") => inventory::ctrl_kinds(&args[1..]),
        Some("page-starts-on") => inventory::page_starts_on(&args[1..]),
        Some("hidden-comment-count") => inventory::hidden_comment_count(&args[1..]),
        Some("ruby-ratio") => inventory::ruby_ratio(&args[1..]),
        Some("char-overlap-len") => inventory::char_overlap_len(&args[1..]),
        Some("table-cols") => inventory::table_cols(&args[1..]),
        Some("picture-instance") => inventory::picture_instance(&args[1..]),
        Some("index-first-keys") => inventory::index_first_keys(&args[1..]),
        Some(other) => {
            eprintln!("오류: 알 수 없는 명령입니다 - {other}");
            EXIT_USAGE
        }
    };
    std::process::exit(code);
}
