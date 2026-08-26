//! rhwp-q-more — 다음 묶음 조회 CLI 50개 + Control 필드 조회 코드.
//!
//! 선점: claimed_pack_next. rhwp-agent / rhwp-q-* / rhwp-q-kit 과 이름 중복 없음.

mod envelope;
mod inventory;
mod probe;

use envelope::{envelope, load_core, parse_slot, print_json, write_stdout, EXIT_OK, EXIT_USAGE};
use serde_json::json;

fn print_help() {
    let _ = write_stdout(&format!(
        "rhwp-q-more v{} — 조회 CLI 50개 + volume-probe",
        rhwp::version()
    ));
    let _ = write_stdout("사용법: rhwp-q-more <명령> [옵션]");
    let _ = write_stdout("  rhwp-q-more para-empty <파일> [--json]  빈 문단 전수\n  rhwp-q-more para-has-ctrl <파일> [--json]  컨트롤 있는 문단\n  rhwp-q-more section-para-lens <파일> [--json]  구역별 문단 수\n  rhwp-q-more body-text-len <파일> [--json]  본문 글자 수\n  rhwp-q-more ctrl-per-para <파일> [--json]  문단당 컨트롤 수\n  rhwp-q-more table-border-fill <파일> [--json]  표 테두리/배경 id\n  rhwp-q-more table-spacing <파일> [--json]  표 셀 간격\n  rhwp-q-more table-attr <파일> [--json]  표 속성 비트\n  rhwp-q-more picture-border-width <파일> [--json]  그림 테두리 두께\n  rhwp-q-more picture-opacity <파일> [--json]  그림 테두리 투명도\n  rhwp-q-more picture-href-set <파일> [--json]  그림 href 유무\n  rhwp-q-more equation-baseline <파일> [--json]  수식 기준선\n  rhwp-q-more equation-color <파일> [--json]  수식 색\n  rhwp-q-more equation-attr <파일> [--json]  수식 attr\n  rhwp-q-more form-height <파일> [--json]  양식 높이\n  rhwp-q-more form-caption <파일> [--json]  양식 캡션\n  rhwp-q-more form-fore-color <파일> [--json]  양식 글자색\n  rhwp-q-more field-properties <파일> [--json]  필드 properties\n  rhwp-q-more field-ctrl-id <파일> [--json]  필드 ctrl_id\n  rhwp-q-more ruby-align <파일> [--json]  덧말 정렬\n  rhwp-q-more ruby-pos <파일> [--json]  덧말 위치\n  rhwp-q-more pagehide-border <파일> [--json]  테두리 감추기\n  rhwp-q-more pagehide-fill <파일> [--json]  배경 감추기\n  rhwp-q-more pagehide-master <파일> [--json]  바탕쪽 감추기\n  rhwp-q-more autonumber-super <파일> [--json]  자동번호 위첨자\n  rhwp-q-more char-overlap-border <파일> [--json]  겹침 테두리\n  rhwp-q-more hyperlink-text-len <파일> [--json]  링크 표시 길이\n  rhwp-q-more bookmark-empty-name <파일> [--json]  빈 책갈피 이름\n  rhwp-q-more header-nonempty <파일> [--json]  비어 있지 않은 머리말\n  rhwp-q-more footer-nonempty <파일> [--json]  비어 있지 않은 꼬리말\n  rhwp-q-more footnote-nonempty <파일> [--json]  비어 있지 않은 각주\n  rhwp-q-more endnote-nonempty <파일> [--json]  비어 있지 않은 미주\n  rhwp-q-more hidden-nonempty <파일> [--json]  비어 있지 않은 숨은 설명\n  rhwp-q-more shape-height <파일> [--json]  그리기 높이\n  rhwp-q-more table-zones <파일> [--json]  표 영역 수\n  rhwp-q-more table-grid-len <파일> [--json]  표 그리드 길이\n  rhwp-q-more field-extra-props <파일> [--json]  필드 extra_properties\n  rhwp-q-more form-back-color <파일> [--json]  양식 배경색\n  rhwp-q-more equation-version <파일> [--json]  수식 버전 정보\n  rhwp-q-more index-both-keys <파일> [--json]  첫째+둘째 키 길이\n  rhwp-q-more para-char-count <파일> [--json]  문단 문자 수\n  rhwp-q-more section-ctrl-total <파일> [--json]  구역 컨트롤 합\n  rhwp-q-more caption-para-count <파일> [--json]  표 캡션 문단 수\n  rhwp-q-more enabled-forms-only <파일> [--json]  활성 양식만\n  rhwp-q-more nonempty-urls <파일> [--json]  비어 있지 않은 URL\n  rhwp-q-more nonempty-scripts <파일> [--json]  비어 있지 않은 수식\n  rhwp-q-more lock-pictures-only <파일> [--json]  잠긴 그림만\n  rhwp-q-more picture-crop-left <파일> [--json]  그림 crop.left\n  rhwp-q-more form-name-len <파일> [--json]  양식 이름 길이\n  rhwp-q-more field-command-len <파일> [--json]  필드 command 길이");
    let _ = write_stdout("  rhwp-q-more volume-probe <파일> --slot <0-49> [--json]");
}

fn volume_probe(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-more volume-probe <파일> --slot <0-49> [--json]";
    let (path, json_mode, slot) = match parse_slot(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let acc = probe::probe_slot(slot, core.document());
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
        Some("para-empty") => inventory::para_empty(&args[1..]),
        Some("para-has-ctrl") => inventory::para_has_ctrl(&args[1..]),
        Some("section-para-lens") => inventory::section_para_lens(&args[1..]),
        Some("body-text-len") => inventory::body_text_len(&args[1..]),
        Some("ctrl-per-para") => inventory::ctrl_per_para(&args[1..]),
        Some("table-border-fill") => inventory::table_border_fill(&args[1..]),
        Some("table-spacing") => inventory::table_spacing(&args[1..]),
        Some("table-attr") => inventory::table_attr(&args[1..]),
        Some("picture-border-width") => inventory::picture_border_width(&args[1..]),
        Some("picture-opacity") => inventory::picture_opacity(&args[1..]),
        Some("picture-href-set") => inventory::picture_href_set(&args[1..]),
        Some("equation-baseline") => inventory::equation_baseline(&args[1..]),
        Some("equation-color") => inventory::equation_color(&args[1..]),
        Some("equation-attr") => inventory::equation_attr(&args[1..]),
        Some("form-height") => inventory::form_height(&args[1..]),
        Some("form-caption") => inventory::form_caption(&args[1..]),
        Some("form-fore-color") => inventory::form_fore_color(&args[1..]),
        Some("field-properties") => inventory::field_properties(&args[1..]),
        Some("field-ctrl-id") => inventory::field_ctrl_id(&args[1..]),
        Some("ruby-align") => inventory::ruby_align(&args[1..]),
        Some("ruby-pos") => inventory::ruby_pos(&args[1..]),
        Some("pagehide-border") => inventory::pagehide_border(&args[1..]),
        Some("pagehide-fill") => inventory::pagehide_fill(&args[1..]),
        Some("pagehide-master") => inventory::pagehide_master(&args[1..]),
        Some("autonumber-super") => inventory::autonumber_super(&args[1..]),
        Some("char-overlap-border") => inventory::char_overlap_border(&args[1..]),
        Some("hyperlink-text-len") => inventory::hyperlink_text_len(&args[1..]),
        Some("bookmark-empty-name") => inventory::bookmark_empty_name(&args[1..]),
        Some("header-nonempty") => inventory::header_nonempty(&args[1..]),
        Some("footer-nonempty") => inventory::footer_nonempty(&args[1..]),
        Some("footnote-nonempty") => inventory::footnote_nonempty(&args[1..]),
        Some("endnote-nonempty") => inventory::endnote_nonempty(&args[1..]),
        Some("hidden-nonempty") => inventory::hidden_nonempty(&args[1..]),
        Some("shape-height") => inventory::shape_height(&args[1..]),
        Some("table-zones") => inventory::table_zones(&args[1..]),
        Some("table-grid-len") => inventory::table_grid_len(&args[1..]),
        Some("field-extra-props") => inventory::field_extra_props(&args[1..]),
        Some("form-back-color") => inventory::form_back_color(&args[1..]),
        Some("equation-version") => inventory::equation_version(&args[1..]),
        Some("index-both-keys") => inventory::index_both_keys(&args[1..]),
        Some("para-char-count") => inventory::para_char_count(&args[1..]),
        Some("section-ctrl-total") => inventory::section_ctrl_total(&args[1..]),
        Some("caption-para-count") => inventory::caption_para_count(&args[1..]),
        Some("enabled-forms-only") => inventory::enabled_forms_only(&args[1..]),
        Some("nonempty-urls") => inventory::nonempty_urls(&args[1..]),
        Some("nonempty-scripts") => inventory::nonempty_scripts(&args[1..]),
        Some("lock-pictures-only") => inventory::lock_pictures_only(&args[1..]),
        Some("picture-crop-left") => inventory::picture_crop_left(&args[1..]),
        Some("form-name-len") => inventory::form_name_len(&args[1..]),
        Some("field-command-len") => inventory::field_command_len(&args[1..]),
        Some(other) => {
            eprintln!("오류: 알 수 없는 명령입니다 - {other}");
            EXIT_USAGE
        }
    };
    std::process::exit(code);
}
