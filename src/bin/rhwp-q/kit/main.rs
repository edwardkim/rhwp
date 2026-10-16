//! rhwp-q kit — 에이전트 조회 CLI 50개. 편집 로직은 없다.
//!
//! 도구 이름 선점: `C:/Users/swsz9/.rhwp-cli-registry.json` (`claimed_pack50`).
//! 이미 있는 rhwp-agent / rhwp-q-* / 진행 중 단건 PR 과 겹치지 않는다.

mod envelope;

mod auto_numbers;
mod bin_data;
mod canvaskit_preflight;
mod caret_stops;
mod cell_shape;
mod char_index;
mod chart_csv;
mod chart_list;
mod covering_pages;
mod cur_field_state;
mod cursor_in_cell;
mod cursor_in_footnote;
mod cursor_in_hf;
mod cursor_in_note;
mod empty_doc;
mod equations;
mod field_by_id;
mod field_info_at;
mod field_list_json;
mod flow_images;
mod fn_selection_rects;
mod footnote_footholds;
mod footnote_hit;
mod footnote_info;
mod form_at;
mod form_value;
mod hf_edit_target;
mod hidden_comments;
mod hit_fn_marker;
mod hit_header;
mod hit_in_footnote;
mod hit_in_hf;
mod hyperlinks;
mod index_marks;
mod layer_tree;
mod line_starts;
mod note_edit_info;
mod overflow_cells;
mod overlay_images;
mod page_border_fill;
mod page_hide;
mod page_text;
mod para_bounds;
mod pictures;
mod ruby;
mod search_all;
mod source_image_bytes;
mod table_overlaps;
mod word_end;
mod word_starts;

use envelope::{write_stdout, EXIT_USAGE};

struct Spec {
    name: &'static str,
    usage: &'static str,
    summary: &'static str,
    handler: fn(&[String]) -> i32,
}

const COMMANDS: &[Spec] = &[
    Spec {
        name: "empty-doc",
        usage: empty_doc::USAGE,
        summary: "빈 문서 여부 (IsEmpty)",
        handler: empty_doc::run,
    },
    Spec {
        name: "cell-shape",
        usage: cell_shape::USAGE,
        summary: "셀 CellShape 파라미터셋",
        handler: cell_shape::run,
    },
    Spec {
        name: "overlay-images",
        usage: overlay_images::USAGE,
        summary: "쪽 overlay 그림",
        handler: overlay_images::run,
    },
    Spec {
        name: "flow-images",
        usage: flow_images::USAGE,
        summary: "쪽 본문 흐름 그림 ops",
        handler: flow_images::run,
    },
    Spec {
        name: "footnote-info",
        usage: footnote_info::USAGE,
        summary: "쪽 각주 참조 좌표",
        handler: footnote_info::run,
    },
    Spec {
        name: "footnote-hit",
        usage: footnote_hit::USAGE,
        summary: "좌표가 각주 영역인지",
        handler: footnote_hit::run,
    },
    Spec {
        name: "footnote-footholds",
        usage: footnote_footholds::USAGE,
        summary: "쪽에 각주 발판이 있는지",
        handler: footnote_footholds::run,
    },
    Spec {
        name: "note-edit-info",
        usage: note_edit_info::USAGE,
        summary: "각주·미주 편집 대상 정보",
        handler: note_edit_info::run,
    },
    Spec {
        name: "cursor-in-cell",
        usage: cursor_in_cell::USAGE,
        summary: "셀 안 캐럿 사각형",
        handler: cursor_in_cell::run,
    },
    Spec {
        name: "cursor-in-hf",
        usage: cursor_in_hf::USAGE,
        summary: "머리말·꼬리말 캐럿 사각형",
        handler: cursor_in_hf::run,
    },
    Spec {
        name: "cursor-in-footnote",
        usage: cursor_in_footnote::USAGE,
        summary: "각주 안 캐럿 사각형",
        handler: cursor_in_footnote::run,
    },
    Spec {
        name: "cursor-in-note",
        usage: cursor_in_note::USAGE,
        summary: "노트 안 캐럿 사각형",
        handler: cursor_in_note::run,
    },
    Spec {
        name: "hit-header",
        usage: hit_header::USAGE,
        summary: "머리말·꼬리말 히트테스트",
        handler: hit_header::run,
    },
    Spec {
        name: "hit-in-hf",
        usage: hit_in_hf::USAGE,
        summary: "머리말·꼬리말 내부 히트",
        handler: hit_in_hf::run,
    },
    Spec {
        name: "hit-in-footnote",
        usage: hit_in_footnote::USAGE,
        summary: "각주 내부 히트",
        handler: hit_in_footnote::run,
    },
    Spec {
        name: "hit-fn-marker",
        usage: hit_fn_marker::USAGE,
        summary: "본문 각주 표식 히트",
        handler: hit_fn_marker::run,
    },
    Spec {
        name: "hf-edit-target",
        usage: hf_edit_target::USAGE,
        summary: "머리말·꼬리말 편집 대상",
        handler: hf_edit_target::run,
    },
    Spec {
        name: "fn-selection-rects",
        usage: fn_selection_rects::USAGE,
        summary: "각주 선택 사각형",
        handler: fn_selection_rects::run,
    },
    Spec {
        name: "field-info-at",
        usage: field_info_at::USAGE,
        summary: "커서 자리 필드 정보",
        handler: field_info_at::run,
    },
    Spec {
        name: "field-by-id",
        usage: field_by_id::USAGE,
        summary: "필드 id로 현재 값",
        handler: field_by_id::run,
    },
    Spec {
        name: "field-list-json",
        usage: field_list_json::USAGE,
        summary: "한글 필드 리스트 JSON",
        handler: field_list_json::run,
    },
    Spec {
        name: "cur-field-state",
        usage: cur_field_state::USAGE,
        summary: "커서 자리 필드 상태 비트",
        handler: cur_field_state::run,
    },
    Spec {
        name: "form-at",
        usage: form_at::USAGE,
        summary: "좌표의 양식 개체",
        handler: form_at::run,
    },
    Spec {
        name: "form-value",
        usage: form_value::USAGE,
        summary: "양식 개체 값",
        handler: form_value::run,
    },
    Spec {
        name: "para-bounds",
        usage: para_bounds::USAGE,
        summary: "문단 경계",
        handler: para_bounds::run,
    },
    Spec {
        name: "line-starts",
        usage: line_starts::USAGE,
        summary: "문단 줄 시작 오프셋",
        handler: line_starts::run,
    },
    Spec {
        name: "word-starts",
        usage: word_starts::USAGE,
        summary: "문단 단어 시작",
        handler: word_starts::run,
    },
    Spec {
        name: "word-end",
        usage: word_end::USAGE,
        summary: "단어 끝 오프셋",
        handler: word_end::run,
    },
    Spec {
        name: "caret-stops",
        usage: caret_stops::USAGE,
        summary: "캐럿 정지 위치",
        handler: caret_stops::run,
    },
    Spec {
        name: "char-index",
        usage: char_index::USAGE,
        summary: "스트림 위치의 글자 인덱스",
        handler: char_index::run,
    },
    Spec {
        name: "page-text",
        usage: page_text::USAGE,
        summary: "한글 스캔 쪽 텍스트",
        handler: page_text::run,
    },
    Spec {
        name: "bin-data",
        usage: bin_data::USAGE,
        summary: "임베드 바이너리 길이·종류",
        handler: bin_data::run,
    },
    Spec {
        name: "source-image-bytes",
        usage: source_image_bytes::USAGE,
        summary: "원본 그림 바이트 메타",
        handler: source_image_bytes::run,
    },
    Spec {
        name: "overflow-cells",
        usage: overflow_cells::USAGE,
        summary: "넘친 셀 줄 수",
        handler: overflow_cells::run,
    },
    Spec {
        name: "table-overlaps",
        usage: table_overlaps::USAGE,
        summary: "표 겹침 목록",
        handler: table_overlaps::run,
    },
    Spec {
        name: "page-border-fill",
        usage: page_border_fill::USAGE,
        summary: "구역 쪽 테두리·배경",
        handler: page_border_fill::run,
    },
    Spec {
        name: "covering-pages",
        usage: covering_pages::USAGE,
        summary: "문단이 걸친 쪽",
        handler: covering_pages::run,
    },
    Spec {
        name: "chart-csv",
        usage: chart_csv::USAGE,
        summary: "차트 데이터 CSV",
        handler: chart_csv::run,
    },
    Spec {
        name: "chart-list",
        usage: chart_list::USAGE,
        summary: "문서 안 차트 목록",
        handler: chart_list::run,
    },
    Spec {
        name: "search-all",
        usage: search_all::USAGE,
        summary: "전 문단 검색 좌표",
        handler: search_all::run,
    },
    Spec {
        name: "canvaskit-preflight",
        usage: canvaskit_preflight::USAGE,
        summary: "CanvasKit 문서 사전점검",
        handler: canvaskit_preflight::run,
    },
    Spec {
        name: "layer-tree",
        usage: layer_tree::USAGE,
        summary: "쪽 레이어 트리",
        handler: layer_tree::run,
    },
    Spec {
        name: "hyperlinks",
        usage: hyperlinks::USAGE,
        summary: "하이퍼링크 목록 (읽기)",
        handler: hyperlinks::run,
    },
    Spec {
        name: "equations",
        usage: equations::USAGE,
        summary: "수식 목록 (읽기)",
        handler: equations::run,
    },
    Spec {
        name: "pictures",
        usage: pictures::USAGE,
        summary: "그림 목록 (읽기)",
        handler: pictures::run,
    },
    Spec {
        name: "hidden-comments",
        usage: hidden_comments::USAGE,
        summary: "숨은 설명 목록 (읽기)",
        handler: hidden_comments::run,
    },
    Spec {
        name: "index-marks",
        usage: index_marks::USAGE,
        summary: "찾아보기 표식 (읽기)",
        handler: index_marks::run,
    },
    Spec {
        name: "auto-numbers",
        usage: auto_numbers::USAGE,
        summary: "자동번호 (읽기)",
        handler: auto_numbers::run,
    },
    Spec {
        name: "ruby",
        usage: ruby::USAGE,
        summary: "덧말 (읽기)",
        handler: ruby::run,
    },
    Spec {
        name: "page-hide",
        usage: page_hide::USAGE,
        summary: "감추기 컨트롤 (읽기)",
        handler: page_hide::run,
    },
];

pub(super) fn help_specs() -> impl Iterator<Item = (&'static str, &'static str, &'static str)> {
    COMMANDS
        .iter()
        .map(|spec| (spec.name, spec.usage, spec.summary))
}

fn find(name: &str) -> Option<&'static Spec> {
    COMMANDS.iter().find(|c| c.name == name)
}

fn print_help() {
    let _ = write_stdout(&format!(
        "rhwp-q kit v{} — 조회 전용 CLI 50개 (선점 목록 claimed_pack50)",
        rhwp::version()
    ));
    let _ = write_stdout("사용법: rhwp-q kit <명령> [옵션]");
    let _ = write_stdout("");
    for c in COMMANDS {
        let _ = write_stdout(&format!("  {}", c.usage));
        let _ = write_stdout(&format!("      {}", c.summary));
    }
}

fn print_command_help(spec: &Spec) {
    let _ = write_stdout(&format!("{} — {}", spec.usage, spec.summary));
    let _ = write_stdout("종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류");
}

pub(super) fn run_cli(args: Vec<String>) {
    let code = match args.first().map(String::as_str) {
        None | Some("--help") | Some("-h") => {
            if args.is_empty() {
                eprintln!("오류: 명령을 지정해주세요.");
                eprintln!("사용법: rhwp-q kit <명령> [옵션]  ('rhwp-q kit --help' 참고)");
                std::process::exit(EXIT_USAGE);
            }
            print_help();
            0
        }
        Some("--version") | Some("-V") => {
            let _ = write_stdout(&rhwp::version());
            0
        }
        Some(name) => match find(name) {
            Some(spec)
                if matches!(args.get(1).map(String::as_str), Some("--help") | Some("-h")) =>
            {
                print_command_help(spec);
                0
            }
            Some(spec) => (spec.handler)(&args[1..]),
            None => {
                eprintln!("오류: 알 수 없는 명령입니다 - {name}");
                eprintln!("사용법: rhwp-q kit <명령> [옵션]  ('rhwp-q kit --help' 참고)");
                EXIT_USAGE
            }
        },
    };
    std::process::exit(code);
}
