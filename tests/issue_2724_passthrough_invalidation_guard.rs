//! [#2724] 패스스루 무효화 누락을 저작 시점에 차단하는 소스 가드.
//!
//! rhwp 는 완전 라운드트립을 위해 원본 바이트를 세 층에 보존한다.
//!
//! | 층 | 필드 | 직렬화 시 동작 |
//! |---|---|---|
//! | 레코드 | `Control::*.raw_ctrl_data` | 비어 있지 않으면 원본 그대로 방출 |
//! | 구역 | `Section::raw_stream` | `Some` 이면 **구역 전체**를 원본 그대로 반환 |
//! | DocInfo | `DocInfo::raw_stream_dirty` | `false` 면 원본 스트림 그대로 반환 |
//!
//! `serialize_section`(`src/serializer/body_text.rs:26-30`)은 `raw_stream` 이 `Some` 이면
//! 함수 첫 줄에서 원본을 반환한다. 그리고 `raw_stream` 은 HWP5 CFB 파싱 경로에서 **항상**
//! 채워진다(`src/parser/mod.rs:465`, `521`). 따라서 IR 을 고친 `&mut self` 메서드가 해당
//! 층을 무효화하지 않으면 **컴파일 에러도 테스트 실패도 런타임 경고도 없이** 사용자의
//! 편집이 저장 시점에 사라진다. `#2698`(`object_ops/connector.rs` 347줄에 무효화 0건)이
//! 이 계급의 최신 사례다.
//!
//! ## 선례 — TS 쪽에는 이미 같은 방어가 있다
//!
//! `rhwp-studio/src/core/mutation-method-registry.ts`(권위 목록) +
//! `rhwp-studio/tests/mutation-routing-guard.test.ts`(저작 시점 소스 가드)가 studio 의
//! "뮤테이션 undo 미기록" 재발 계급(`#2027`/`#2037`/`#2053`/`#2077`)을 봉인한다.
//! 이 파일은 그 설계를 코드베이스의 나머지 절반(Rust)으로 포팅한 것이다. TS 가드가
//! 런타임 가드를 버리고 저작 시점 소스 스캔만 남긴 경위(오탐 → 경고 소진 → 진짜 결함까지
//! 침묵, PR #2329)를 그대로 따른다 — **오탐 나는 가드는 꺼지고, 꺼진 가드는 없는 것보다
//! 나쁘다.**
//!
//! ## 검사 5개
//!
//! 1. `classification_drift_is_blocked` — 범위 내 `pub fn (&mut self)` 는 본문에서 직접
//!    무효화하거나 `EXEMPT` 에 근거와 함께 등재돼야 한다.
//! 2. `stale_exemptions_are_reclaimed` — `EXEMPT` 항목은 실재해야 하고, 지금 무효화하고
//!    있으면 안 되며, 판정 보류(`Pending`)는 상한을 넘을 수 없다
//!    (래칫: 면제는 줄어들기만 한다).
//! 3. `delegation_targets_actually_invalidate` — `DelegatesTo` 대상이 실재하고 실제로
//!    무효화에 도달하는지 확인(rename/리팩터로 위임 주장이 껍데기가 되는 것 차단).
//! 4. `invalidation_density_ledger_is_ratcheted` — 파일별 무효화 사이트 수 하한 동결.
//!    한 함수 안 여러 갈래 중 일부만 지우는 경우를 잡는다(PR #2704 의 곡선 갈래 누락이
//!    실제로 그 형태였다).
//! 5. `guard_scanner_self_check` — 스캐너가 조용히 0건을 반환해 1~4 가 공허하게 통과하는
//!    것을 막는다.
//!
//! ## 목록 갱신 방법
//!
//! - 새 `pub fn (&mut self)` 가 문서 IR 을 바꾼다 → 본문에서 무효화하라(등재 불필요).
//! - 바꾸지 않는다 / 다른 뮤테이터에 위임한다 → `EXEMPT` 에 **분류와 근거를 적어** 추가한다.
//!   근거 없는 allowlist 는 가치가 없다.
//! - 무효화를 추가·이관해 `INVALIDATION_LEDGER` 수치가 달라졌다 → 의식적으로 갱신한다.
//!
//! ## 못 잡는 것 (정직 고지)
//!
//! - 무효화 **대상이 틀린** 경우(A 구역을 고치고 B 구역을 무효화) — 데이터플로 분석 필요.
//! - 처음부터 **일부 갈래만** 무효화하는 신규 코드 — 검사 4가 "감소"로만 근사한다.
//! - `src/wasm_api.rs` 어댑터 층의 직접 뮤테이션 — 범위 밖(후속 과제).
//! - 레코드 층(`raw_ctrl_data`) — 무효화 관용구가 단일하지 않아 이번 범위에서 제외.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// 스캔 루트 (저장소 루트 기준). 이 아래 `.rs` 전부가 대상이다.
const SCAN_ROOT: &str = "src/document_core";

/// 패스스루 무효화 관용구. rustfmt 정규화로 공백 1칸 형태가 보장된다.
const INVALIDATION_TOKENS: [&str; 2] = ["raw_stream = None", "raw_stream_dirty = true"];

/// 면제 분류. 각 항목은 반드시 근거 문자열을 동반한다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Exempt {
    /// 문서 IR 을 바꾸지 않는다 — 클립보드·스냅샷·배치·활성필드·DPI 등 세션/캐시 상태.
    SessionState,
    /// 문서 전체를 교체·생성한다 — 패스스루가 새 문서와 함께 재설정된다.
    WholeDocument,
    /// 무효화 대신 원본 스트림을 직접 수술(surgical)해 반영한다.
    SurgicalRawEdit,
    /// 패스스루 자체가 존재하지 않는 경로(HWP3 원본은 파서가 `raw_stream: None`).
    NoPassthrough,
    /// 원시 가변 핸들만 넘겨줄 뿐 스스로는 아무것도 바꾸지 않는다 — 무효화는 호출자 책임.
    CallerResponsibility,
    /// 실제 무효화를 수행하는 다른 함수에 위임한다(대상 이름은 검사 3이 검증).
    DelegatesTo(&'static str),
    /// 판정 보류 — 지속성 계약이 미확정이라 추정으로 고치지 않는다.
    Pending,
}

/// 무효화하지 않는 `pub fn (&mut self)` 전수 목록 — (파일, 함수, 분류, 근거).
///
/// 파일 경로는 [`SCAN_ROOT`] 기준 상대 경로다. `devel` 기준 40건(2026-07-23 동결).
const EXEMPT: &[(&str, &str, Exempt, &str)] = &[
    // ── 세션/캐시 상태만 변경 (문서 IR 비변경) ──────────────────────────────
    (
        "mod.rs",
        "set_dpi",
        Exempt::SessionState,
        "렌더 DPI·해소 스타일·페이지네이션만 갱신. 문서 IR 무변경.",
    ),
    (
        "queries/rendering.rs",
        "begin_deferred_pagination",
        Exempt::SessionState,
        "편집이 만든 descriptor로 shadow pagination job과 측정 캐시만 준비. 문서 IR 무변경.",
    ),
    (
        "queries/rendering.rs",
        "step_deferred_pagination",
        Exempt::SessionState,
        "shadow 결과를 pagination·측정·dirty 캐시에 commit. `Table::dirty`는 직렬화 비대상 런타임 플래그.",
    ),
    (
        "queries/rendering.rs",
        "cancel_deferred_pagination",
        Exempt::SessionState,
        "pending shadow job만 폐기. 문서 IR과 패스스루 무변경.",
    ),
    (
        "queries/rendering.rs",
        "flush_deferred_pagination",
        Exempt::SessionState,
        "pending job을 drain하거나 기존 paginate로 fallback. 편집 IR은 선행 뮤테이터가 이미 무효화.",
    ),
    (
        "commands/clipboard.rs",
        "clear_clipboard_native",
        Exempt::SessionState,
        "`self.clipboard = None` 뿐.",
    ),
    (
        "commands/clipboard.rs",
        "copy_selection_native",
        Exempt::SessionState,
        "복사 — 읽기 후 `self.clipboard` 에만 기록.",
    ),
    (
        "commands/clipboard.rs",
        "copy_selection_in_cell_native",
        Exempt::SessionState,
        "복사 — 읽기 후 `self.clipboard` 에만 기록.",
    ),
    (
        "commands/clipboard.rs",
        "copy_control_native",
        Exempt::SessionState,
        "복사 — `self.clipboard` / `self.paste_cascade_count` 만 변경.",
    ),
    (
        "commands/document.rs",
        "begin_batch_native",
        Exempt::SessionState,
        "`batch_mode` 플래그와 이벤트 로그만 조작.",
    ),
    (
        "commands/document.rs",
        "end_batch_native",
        Exempt::SessionState,
        "`batch_mode` 해제 + 재페이지네이션 + 이벤트 로그 직렬화.",
    ),
    (
        "commands/document.rs",
        "save_snapshot_native",
        Exempt::SessionState,
        "문서를 clone 해 `snapshot_store` 에 적재. 원본 IR 무변경.",
    ),
    (
        "commands/document.rs",
        "discard_snapshot_native",
        Exempt::SessionState,
        "`snapshot_store` 에서 항목 제거.",
    ),
    (
        "commands/formatting.rs",
        "get_cell_char_properties_at_by_path",
        Exempt::SessionState,
        "순수 조회. `&mut` 는 가변 접근자(`get_cell_paragraph_mut_by_path`) 재사용 때문.",
    ),
    (
        "commands/header_footer_ops.rs",
        "toggle_hide_header_footer_native",
        Exempt::SessionState,
        "세션 집합 `hidden_header_footer` + 렌더 트리 캐시만 변경. 직렬화 비대상.",
    ),
    (
        "commands/table_ops.rs",
        "copy_table_cells_transposed_native",
        Exempt::SessionState,
        "복사 — `table_transpose_clipboard` 에만 기록.",
    ),
    (
        "queries/field_query.rs",
        "set_active_field",
        Exempt::SessionState,
        "편집 세션의 활성 필드 포커스 + 렌더 캐시 무효화. 직렬화 비대상.",
    ),
    (
        "queries/field_query.rs",
        "set_active_field_in_cell",
        Exempt::SessionState,
        "편집 세션의 활성 필드 포커스. 직렬화 비대상.",
    ),
    (
        "queries/field_query.rs",
        "set_active_field_by_path",
        Exempt::SessionState,
        "편집 세션의 활성 필드 포커스. 직렬화 비대상.",
    ),
    (
        "queries/field_query.rs",
        "clear_active_field",
        Exempt::SessionState,
        "활성 필드 해제 + 렌더 캐시 무효화. 직렬화 비대상.",
    ),
    // ── 문서 전체 교체·생성 ────────────────────────────────────────────────
    (
        "commands/document.rs",
        "create_blank_document_native",
        Exempt::WholeDocument,
        "내장 템플릿을 파싱해 `self.document` 를 통째로 교체. 패스스루도 함께 새로 설정된다.",
    ),
    (
        "commands/document.rs",
        "set_document",
        Exempt::WholeDocument,
        "주입된 `Document` 로 통째 교체. 패스스루는 주입 측 IR 의 것을 그대로 따른다.",
    ),
    (
        "commands/document.rs",
        "restore_snapshot_native",
        Exempt::WholeDocument,
        "스냅샷 문서로 통째 복원. 스냅샷은 저장 시점의 패스스루 상태를 그대로 담고 있다.",
    ),
    // ── 무효화 대신 원본 스트림 직접 수술 ──────────────────────────────────
    (
        "commands/document.rs",
        "convert_to_editable_native",
        Exempt::SurgicalRawEdit,
        "`Document::convert_to_editable` 이 `header.raw_data = None` + \
         `distribute_doc_data_removed` 를 세우고, 직렬화기가 원본 스트림에서 \
         DISTRIBUTE_DOC_DATA 만 surgical remove 한다(`serializer/doc_info.rs:27-30`).",
    ),
    (
        "commands/formatting.rs",
        "find_or_create_font_id_native",
        Exempt::SurgicalRawEdit,
        "DocInfo 를 dirty 로 만드는 대신 `surgical_insert_font_all_langs` 로 원본 \
         스트림에 FACE_NAME 을 직접 삽입한다(전체 재직렬화 회피 — FIX-4 계열 위험).",
    ),
    (
        "commands/formatting.rs",
        "find_or_create_font_id_for_lang",
        Exempt::SurgicalRawEdit,
        "위와 동일 — `surgical_insert_font_all_langs` 로 원본 스트림 직접 반영.",
    ),
    // ── 패스스루 부재 ──────────────────────────────────────────────────────
    (
        "commands/document.rs",
        "populate_external_images_from_dir",
        Exempt::NoPassthrough,
        "HWP3 외부 경로 그림 전용(비-wasm CLI 경로). HWP3 파서는 \
         `raw_stream: None`(`parser/hwp3/mod.rs:3241`)이라 무효화할 패스스루가 없고, \
         적재 대상 `bin_data_content` 는 DocInfo 레코드가 아니라 BinData 저장소다.",
    ),
    // ── 호출자 책임 ────────────────────────────────────────────────────────
    (
        "commands/document.rs",
        "document_mut",
        Exempt::CallerResponsibility,
        "`&mut Document` 를 그대로 넘기는 탈출구. 스스로는 아무것도 바꾸지 않으므로 \
         무효화 책임은 전적으로 호출자에게 있다.",
    ),
    // ── 위임 ───────────────────────────────────────────────────────────────
    (
        "commands/document.rs",
        "export_hwp_with_adapter",
        Exempt::DelegatesTo("convert_if_hwpx_source"),
        "저장 직전 어댑터 변환. IR 변경은 전부 어댑터 안에서 일어나며 그쪽이 \
         `raw_stream_dirty` 를 세운다.",
    ),
    (
        "commands/document.rs",
        "serialize_hwp_with_verify",
        Exempt::DelegatesTo("export_hwp_with_adapter"),
        "export 후 재로드 검증만 수행. 자체 IR 변경 없음.",
    ),
    (
        "commands/table_ops.rs",
        "paste_table_cells_transposed_as_new_table_native",
        Exempt::DelegatesTo("create_table_native"),
        "표 생성 경로를 그대로 태운다.",
    ),
    (
        "commands/table_ops.rs",
        "fit_table_to_page_native",
        Exempt::DelegatesTo("set_table_column_widths_native"),
        "열 폭 계산만 하고 실제 반영은 열 폭 설정 뮤테이터가 한다.",
    ),
    (
        "commands/text_editing.rs",
        "insert_text_in_cell_native",
        Exempt::DelegatesTo("insert_text_in_cell_native_impl"),
        "얇은 래퍼 — 실제 삽입·무효화는 `_impl` 이 수행.",
    ),
    (
        "commands/text_editing.rs",
        "insert_text_in_cell_native_deferred_pagination",
        Exempt::DelegatesTo("insert_text_in_cell_native_impl"),
        "페이지네이션 지연 플래그만 다른 래퍼 — 본체는 `_impl`.",
    ),
    (
        "queries/field_query.rs",
        "set_field_value_by_id",
        Exempt::DelegatesTo("set_field_text_at"),
        "필드 위치를 조회한 뒤 텍스트 치환 헬퍼에 위임.",
    ),
    (
        "queries/rendering.rs",
        "set_section_def_native",
        Exempt::DelegatesTo("apply_section_def_json"),
        "JSON 파싱·적용은 공통 헬퍼가 담당.",
    ),
    (
        "queries/rendering.rs",
        "set_section_def_all_native",
        Exempt::DelegatesTo("apply_section_def_json"),
        "전 구역 루프 — 구역별 적용·무효화는 공통 헬퍼가 담당.",
    ),
    (
        "queries/search_query.rs",
        "replace_text_native",
        Exempt::DelegatesTo("delete_text_native"),
        "삭제 + 삽입 조합. 두 뮤테이터가 각각 무효화한다.",
    ),
    (
        "queries/search_query.rs",
        "replace_one_native",
        Exempt::DelegatesTo("delete_text_native"),
        "검색 후 삭제 + 삽입 조합. 두 뮤테이터가 각각 무효화한다.",
    ),
    // ── 판정 보류 ──────────────────────────────────────────────────────────
    (
        "commands/document.rs",
        "reflow_linesegs_on_demand",
        Exempt::Pending,
        "[#2724 3.4] wasm 공개 API `reflowLinesegs`. `para.line_segs` 를 재작성하지만 \
         `raw_stream` 을 비우지 않아 `raw_stream` 보유 문서에서는 보정이 저장되지 않는다. \
         다만 docstring 이 효과를 렌더 기준으로 서술하고, studio 는 #2527 이후 이 API 를 \
         호출하지 않아 관측된 유실이 없다. 구역 전체 재직렬화(FIX-4 계열) 위험이 있어 \
         추정으로 고치지 않고 지속성 계약 확정까지 보류한다.",
    ),
];

/// 파일별 무효화 사이트 **하한** 원장 ([`SCAN_ROOT`] 기준 상대 경로, `#[cfg(test)]` 제외).
///
/// `devel` 기준 21파일 135사이트(2026-07-21 동결). 감소는 실패(무효화가 지워졌다는 뜻),
/// 증가는 통과하며 갱신을 안내한다. 함수 단위 검사(검사 1)가 못 잡는 "한 함수 안 여러
/// 무효화 갈래 중 일부만 제거" 를 잡는 것이 목적이다.
const INVALIDATION_LEDGER: &[(&str, usize)] = &[
    ("commands/clipboard.rs", 4),
    ("commands/footnote_ops.rs", 6),
    ("commands/formatting.rs", 16),
    ("commands/header_footer_ops.rs", 9),
    ("commands/html_import.rs", 5),
    ("commands/object_ops/common.rs", 2),
    ("commands/object_ops/connector.rs", 4),
    ("commands/object_ops/equation.rs", 3),
    ("commands/object_ops/note.rs", 3),
    ("commands/object_ops/picture.rs", 8),
    ("commands/object_ops/shape.rs", 7),
    ("commands/object_ops/table.rs", 7),
    ("commands/table_ops.rs", 19),
    ("commands/text_editing.rs", 21),
    ("converters/hwpx_to_hwp.rs", 3),
    ("html_table_import.rs", 2),
    ("queries/bookmark_query.rs", 3),
    ("queries/field_query.rs", 7),
    ("queries/form_query.rs", 2),
    ("queries/rendering.rs", 3),
    ("queries/search_query.rs", 1),
];

/// 스캐너 자기검사 하한. 실측 `devel` 값은 각각 144 / 135 다.
/// 정규식·경로 변경으로 스캐너가 조용히 0건을 반환하면 검사 1~4 가 공허하게 통과한다.
const MIN_PUB_MUT_SELF_METHODS: usize = 130;
/// 무효화 사이트 총합 하한.
const MIN_TOTAL_INVALIDATION_SITES: usize = 120;

// ────────────────────────────────────────────────────────────────────────────
// 소스 스캐너
// ────────────────────────────────────────────────────────────────────────────

/// 스캔 대상 함수 1개.
struct FnItem {
    /// [`SCAN_ROOT`] 기준 상대 경로 (슬래시 구분).
    file: String,
    line: usize,
    name: String,
    is_pub: bool,
    mut_self: bool,
    in_document_core: bool,
    /// 노이즈 제거 + 공백 정규화된 본문.
    body: String,
}

fn scan_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SCAN_ROOT)
}

/// `dir` 아래 `.rs` 파일을 재귀 수집한다(경로 정렬 — 출력 결정성).
fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("스캔 루트를 읽을 수 없음 {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().map(|e| e == "rs").unwrap_or(false) {
            out.push(p);
        }
    }
}

/// 주석·문자열·문자 리터럴을 공백으로 치환한다(바이트 오프셋·줄 번호 보존).
///
/// 치환 구간의 개행은 살려 두어 줄 번호 계산이 어긋나지 않게 한다.
fn strip_noise(src: &str) -> String {
    let b = src.as_bytes();
    let n = b.len();
    let mut out = vec![b' '; n];
    let mut i = 0;
    while i < n {
        // 라인 주석
        if b[i] == b'/' && i + 1 < n && b[i + 1] == b'/' {
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // 블록 주석 (Rust 는 중첩 허용)
        if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
            i += 2;
            let mut depth = 1usize;
            while i < n && depth > 0 {
                if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // raw 문자열 r"..." / r#"..."# / br"..." / br#"..."#
        // (`r#ident` raw identifier 는 `#` 뒤가 따옴표가 아니라 여기 걸리지 않는다.)
        let raw_r = if b[i] == b'r' {
            Some(i)
        } else if b[i] == b'b' && i + 1 < n && b[i + 1] == b'r' {
            Some(i + 1)
        } else {
            None
        };
        if let Some(rs) = raw_r {
            if i == 0 || !is_word_byte(b[i - 1]) {
                let mut k = rs + 1;
                while k < n && b[k] == b'#' {
                    k += 1;
                }
                if k < n && b[k] == b'"' {
                    let hashes = k - rs - 1;
                    let mut j = k + 1;
                    while j < n {
                        if b[j] == b'"' {
                            let mut h = 0;
                            while h < hashes && j + 1 + h < n && b[j + 1 + h] == b'#' {
                                h += 1;
                            }
                            if h == hashes {
                                j += 1 + hashes;
                                break;
                            }
                        }
                        j += 1;
                    }
                    blank_range(&mut out, b, i, j.min(n));
                    i = j.min(n);
                    continue;
                }
            }
        }
        // 일반 문자열
        if b[i] == b'"' {
            let mut j = i + 1;
            while j < n {
                if b[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if b[j] == b'"' {
                    j += 1;
                    break;
                }
                j += 1;
            }
            blank_range(&mut out, b, i, j.min(n));
            i = j.min(n);
            continue;
        }
        // 문자 리터럴 vs 라이프타임
        if b[i] == b'\'' {
            let mut j = i + 1;
            if j < n && b[j] == b'\\' {
                j += 2;
                while j < n && b[j] != b'\'' && b[j] != b'\n' {
                    j += 1;
                }
            } else if j < n {
                j += 1;
                while j < n && (b[j] & 0xC0) == 0x80 {
                    j += 1;
                }
            }
            if j < n && b[j] == b'\'' {
                blank_range(&mut out, b, i, j + 1);
                i = j + 1;
                continue;
            }
            // 라이프타임 — 그대로 둔다.
        }
        out[i] = b[i];
        i += 1;
    }
    String::from_utf8(out).expect("strip_noise 결과가 UTF-8 이 아님")
}

fn blank_range(out: &mut [u8], src: &[u8], from: usize, to: usize) {
    for k in from..to {
        out[k] = if src[k] == b'\n' { b'\n' } else { b' ' };
    }
}

fn is_word_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// `#[cfg(test)]` 가 붙은 블록(모듈·함수)을 공백으로 지운다.
///
/// 테스트 안의 `raw_stream = None` 은 무효화가 아니라 fixture 조작이므로 원장·본문
/// 검사 양쪽에서 제외돼야 한다.
fn blank_cfg_test(stripped: &str) -> String {
    let mut out = stripped.as_bytes().to_vec();
    let b = stripped.as_bytes();
    let needle = b"#[cfg(test)]";
    let mut i = 0;
    while i + needle.len() <= b.len() {
        if &b[i..i + needle.len()] == needle {
            if let Some((_, end)) = find_body(b, i + needle.len()) {
                blank_range(&mut out, b, i, end);
                i = end;
                continue;
            }
        }
        i += 1;
    }
    String::from_utf8(out).expect("blank_cfg_test 결과가 UTF-8 이 아님")
}

/// `from` 이후 첫 최상위 `{` 를 찾아 대응하는 `}` 다음 위치까지 반환한다.
///
/// 괄호 깊이 0 에서 `;` 를 만나면 본문 없는 선언이므로 `None`.
fn find_body(b: &[u8], from: usize) -> Option<(usize, usize)> {
    let n = b.len();
    let mut paren = 0i32;
    let mut i = from;
    while i < n {
        match b[i] {
            b'(' | b'[' => paren += 1,
            b')' | b']' => paren -= 1,
            b';' if paren <= 0 => return None,
            b'{' if paren <= 0 => {
                let start = i;
                let mut depth = 0i32;
                while i < n {
                    if b[i] == b'{' {
                        depth += 1;
                    } else if b[i] == b'}' {
                        depth -= 1;
                        if depth == 0 {
                            return Some((start, i + 1));
                        }
                    }
                    i += 1;
                }
                return None;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// 공백 런을 한 칸으로 접는다(줄바꿈으로 쪼개진 관용구 탐지용).
fn squash_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_ws {
                out.push(' ');
            }
            prev_ws = true;
        } else {
            out.push(c);
            prev_ws = false;
        }
    }
    out
}

/// `impl ... DocumentCore {` 블록 범위 목록.
fn document_core_impls(b: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut i = 0;
    while i + 5 <= b.len() {
        let at_line_start = i == 0 || b[i - 1] == b'\n';
        if at_line_start && &b[i..i + 5] == b"impl " {
            let mut j = i;
            while j < b.len() && b[j] != b'{' && b[j] != b'\n' {
                j += 1;
            }
            if j < b.len() && b[j] == b'{' {
                let head = String::from_utf8_lossy(&b[i + 5..j]);
                if head.trim().ends_with("DocumentCore") {
                    if let Some((_, end)) = find_body(b, j) {
                        ranges.push((j, end));
                    }
                }
            }
        }
        i += 1;
    }
    ranges
}

/// 스캔 루트 전체에서 본문이 있는 함수를 수집한다.
fn collect_functions() -> Vec<FnItem> {
    let root = scan_root();
    let mut files = Vec::new();
    rs_files(&root, &mut files);
    let mut items = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(&root)
            .expect("스캔 루트 하위 경로")
            .to_string_lossy()
            .replace('\\', "/");
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("파일을 읽을 수 없음 {}: {e}", path.display()));
        let cleaned = blank_cfg_test(&strip_noise(&src));
        let b = cleaned.as_bytes();
        let impls = document_core_impls(b);
        let mut i = 0;
        while i + 3 <= b.len() {
            if &b[i..i + 3] == b"fn " && (i == 0 || !is_word_byte(b[i - 1])) {
                // 같은 줄의 앞부분으로 가시성 판정 — 예상 밖 토큰이 있으면 fn 선언이 아니다.
                let mut ls = i;
                while ls > 0 && b[ls - 1] != b'\n' {
                    ls -= 1;
                }
                let prefix = String::from_utf8_lossy(&b[ls..i]);
                if let Some(is_pub) = parse_visibility(prefix.trim()) {
                    let mut j = i + 3;
                    while j < b.len() && b[j] == b' ' {
                        j += 1;
                    }
                    let ns = j;
                    while j < b.len() && is_word_byte(b[j]) {
                        j += 1;
                    }
                    if j > ns {
                        if let Some((bs, be)) = find_body(b, j) {
                            let sig = squash_ws(&String::from_utf8_lossy(&b[i..bs]));
                            items.push(FnItem {
                                file: rel.clone(),
                                line: cleaned[..i].matches('\n').count() + 1,
                                name: String::from_utf8_lossy(&b[ns..j]).to_string(),
                                is_pub,
                                mut_self: sig.contains("&mut self"),
                                in_document_core: impls.iter().any(|&(s, e)| s < i && i < e),
                                body: squash_ws(&cleaned[bs..be]),
                            });
                            i = be;
                            continue;
                        }
                    }
                }
            }
            i += 1;
        }
    }
    items
}

/// `fn` 앞 토큰열이 함수 선언으로 유효하면 `Some(공개 여부)`.
fn parse_visibility(prefix: &str) -> Option<bool> {
    let mut is_pub = false;
    let mut rest = prefix;
    if let Some(p) = rest.strip_prefix("pub") {
        // `pub`, `pub(crate)`, `pub(super)`, `pub(in ...)`
        if let Some(open) = p.strip_prefix('(') {
            let close = open.find(')')?;
            // 제한 가시성은 공개 API 가 아니다.
            rest = open[close + 1..].trim_start();
        } else {
            is_pub = true;
            rest = p.trim_start();
        }
    }
    for tok in rest.split_whitespace() {
        if !matches!(tok, "async" | "unsafe" | "const" | "default" | "extern") {
            return None;
        }
    }
    Some(is_pub)
}

/// 본문이 패스스루를 직접 무효화하는가.
fn invalidates(body: &str) -> bool {
    INVALIDATION_TOKENS.iter().any(|t| body.contains(t))
}

/// 본문에서 `이름(` 형태로 호출되는 식별자를 모은다.
fn called_names(body: &str) -> BTreeSet<String> {
    let b = body.as_bytes();
    let mut out = BTreeSet::new();
    for (i, &c) in b.iter().enumerate() {
        if c != b'(' {
            continue;
        }
        let mut s = i;
        while s > 0 && is_word_byte(b[s - 1]) {
            s -= 1;
        }
        if s == i {
            continue;
        }
        let name = &body[s..i];
        if name.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        if matches!(
            name,
            "if" | "for" | "while" | "match" | "return" | "fn" | "loop"
        ) {
            continue;
        }
        out.insert(name.to_string());
    }
    out
}

/// `name` 이 `depth` 홉 안에서 직접 무효화에 도달하는가(위임 주장 검증용).
fn reaches_invalidation(
    index: &BTreeMap<String, Vec<usize>>,
    all: &[FnItem],
    name: &str,
    depth: usize,
    seen: &mut BTreeSet<String>,
) -> bool {
    if depth == 0 || !seen.insert(name.to_string()) {
        return false;
    }
    let Some(ids) = index.get(name) else {
        return false;
    };
    for &id in ids {
        if invalidates(&all[id].body) {
            return true;
        }
    }
    for &id in ids {
        for callee in called_names(&all[id].body) {
            if reaches_invalidation(index, all, &callee, depth - 1, seen) {
                return true;
            }
        }
    }
    false
}

/// 범위 내 `pub fn (&mut self)` — 검사 1·2의 대상 집합.
fn scoped_mutators(all: &[FnItem]) -> Vec<&FnItem> {
    all.iter()
        .filter(|f| f.is_pub && f.mut_self && f.in_document_core)
        .collect()
}

fn exempt_key(file: &str, name: &str) -> String {
    format!("{file}::{name}")
}

// ────────────────────────────────────────────────────────────────────────────
// 검사 1 — 분류 드리프트
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn classification_drift_is_blocked() {
    let all = collect_functions();
    let registered: BTreeSet<String> = EXEMPT
        .iter()
        .map(|&(f, n, _, _)| exempt_key(f, n))
        .collect();

    let mut violations: Vec<String> = Vec::new();
    for f in scoped_mutators(&all) {
        if invalidates(&f.body) {
            continue;
        }
        if registered.contains(&exempt_key(&f.file, &f.name)) {
            continue;
        }
        violations.push(format!(
            "  + {}/{}:{} {}() — 패스스루 무효화 없음",
            SCAN_ROOT, f.file, f.line, f.name
        ));
    }
    violations.sort();

    assert!(
        violations.is_empty(),
        "미분류 문서 뮤테이터 {}건 (#2724):\n{}\n\n\
         `pub fn (&mut self)` 가 문서 IR 을 바꾸면 본문에서 패스스루를 무효화해야 한다\n\
         (`section.raw_stream = None` / `doc_info.raw_stream_dirty = true`).\n\
         빠뜨리면 `serialize_section`(serializer/body_text.rs:26-30)이 원본 바이트를 그대로\n\
         반환해 편집이 저장 결과에서 사라진다 — 컴파일 에러도 테스트 실패도 없이.\n\
         바꾸지 않거나 다른 뮤테이터에 위임한다면 이 파일\n\
         (tests/issue_2724_passthrough_invalidation_guard.rs)의 EXEMPT 에 분류와 근거를\n\
         적어 등재하라.",
        violations.len(),
        violations.join("\n"),
    );
}

// ────────────────────────────────────────────────────────────────────────────
// 검사 2 — stale 면제 회수 (래칫)
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn stale_exemptions_are_reclaimed() {
    // `Pending` 은 "지속성 계약이 미확정" 이라는 표시지 면제 사유가 아니다. 덤핑
    // 그라운드가 되지 않도록 상한을 둔다 — 늘리려면 그만한 근거를 리뷰에 올려야 한다.
    // 현재 1건: commands/document.rs::reflow_linesegs_on_demand (#2724 3.4).
    const MAX_PENDING: usize = 1;
    let pending: Vec<&str> = EXEMPT
        .iter()
        .filter(|e| e.2 == Exempt::Pending)
        .map(|e| e.1)
        .collect();
    assert!(
        pending.len() <= MAX_PENDING,
        "판정 보류(Exempt::Pending)가 {}건이다(상한 {}): {:?}\n\
         보류는 임시 상태다 — 계약을 확정해 다른 분류로 옮기거나 무효화를 추가하라.",
        pending.len(),
        MAX_PENDING,
        pending,
    );

    let all = collect_functions();
    let scoped = scoped_mutators(&all);
    let by_key: BTreeMap<String, &FnItem> = scoped
        .iter()
        .map(|f| (exempt_key(&f.file, &f.name), *f))
        .collect();

    let mut missing: Vec<String> = Vec::new();
    let mut now_invalidating: Vec<String> = Vec::new();
    let mut dupes: Vec<String> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    for &(file, name, _, reason) in EXEMPT {
        let key = exempt_key(file, name);
        if !seen.insert(key.clone()) {
            dupes.push(format!("  ! {key} — 중복 등재"));
        }
        assert!(
            reason.trim().chars().count() >= 10,
            "{key} 의 면제 근거가 비었거나 너무 짧다 — 근거 없는 allowlist 는 가치가 없다"
        );
        match by_key.get(&key) {
            None => missing.push(format!(
                "  - {key} — 범위 내 `pub fn (&mut self)` 로 실재하지 않음(rename/제거/가시성 변경?)"
            )),
            Some(f) if invalidates(&f.body) => now_invalidating.push(format!(
                "  ↓ {key} ({}/{}:{}) — 이제 무효화한다. 면제 항목을 삭제하라",
                SCAN_ROOT, f.file, f.line
            )),
            Some(_) => {}
        }
    }

    let mut problems = Vec::new();
    problems.extend(dupes);
    problems.extend(missing);
    problems.extend(now_invalidating);
    assert!(
        problems.is_empty(),
        "EXEMPT 레지스트리가 소스와 어긋났다 {}건 (#2724):\n{}\n\n\
         면제는 **줄어들기만** 해야 한다(래칫). 항목을 지우고 나면 그 뒤에 무효화를\n\
         제거할 때 classification_drift_is_blocked 가 잡는다. 방치하면 목록이 실재하지\n\
         않는 이름으로 채워져 가드가 무통보로 헐거워진다.",
        problems.len(),
        problems.join("\n"),
    );
}

// ────────────────────────────────────────────────────────────────────────────
// 검사 3 — 위임 대상 검증
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn delegation_targets_actually_invalidate() {
    let all = collect_functions();
    let mut index: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, f) in all.iter().enumerate() {
        index.entry(f.name.clone()).or_default().push(i);
    }

    let mut violations: Vec<String> = Vec::new();
    for &(file, name, kind, _) in EXEMPT {
        let Exempt::DelegatesTo(target) = kind else {
            continue;
        };
        let key = exempt_key(file, name);
        if !index.contains_key(target) {
            violations.push(format!(
                "  ? {key} → {target}() — 위임 대상이 {SCAN_ROOT} 안에 없다(rename/이동?)"
            ));
            continue;
        }
        let mut seen = BTreeSet::new();
        if !reaches_invalidation(&index, &all, target, 4, &mut seen) {
            violations.push(format!(
                "  x {key} → {target}() — 위임 대상이 무효화에 도달하지 않는다"
            ));
        }
    }
    violations.sort();

    assert!(
        violations.is_empty(),
        "위임 주장이 검증되지 않았다 {}건 (#2724):\n{}\n\n\
         `Exempt::DelegatesTo(\"X\")` 는 \"내가 아니라 X 가 무효화한다\"는 주장이다.\n\
         X 가 rename/제거되거나 무효화를 잃으면 이 주장은 껍데기가 되고, 그 사이 원래\n\
         함수는 아무 신호 없이 무방비가 된다.",
        violations.len(),
        violations.join("\n"),
    );
}

// ────────────────────────────────────────────────────────────────────────────
// 검사 4 — 무효화 밀도 원장 (하한 래칫)
// ────────────────────────────────────────────────────────────────────────────

/// 파일별 무효화 사이트 실측(`#[cfg(test)]` 제외).
fn measure_invalidation_sites() -> BTreeMap<String, usize> {
    let root = scan_root();
    let mut files = Vec::new();
    rs_files(&root, &mut files);
    let mut out = BTreeMap::new();
    for path in files {
        let rel = path
            .strip_prefix(&root)
            .expect("스캔 루트 하위 경로")
            .to_string_lossy()
            .replace('\\', "/");
        let src = std::fs::read_to_string(&path).expect("소스 읽기");
        let cleaned = squash_ws(&blank_cfg_test(&strip_noise(&src)));
        let n: usize = INVALIDATION_TOKENS
            .iter()
            .map(|t| cleaned.matches(t).count())
            .sum();
        if n > 0 {
            out.insert(rel, n);
        }
    }
    out
}

#[test]
fn invalidation_density_ledger_is_ratcheted() {
    let current = measure_invalidation_sites();
    let baseline: BTreeMap<&str, usize> = INVALIDATION_LEDGER.iter().copied().collect();

    let mut violations: Vec<String> = Vec::new();
    for (file, &base) in &baseline {
        let now = current.get(*file).copied().unwrap_or(0);
        if now < base {
            violations.push(format!(
                "  ↓ {SCAN_ROOT}/{file}: {base} → {now} (무효화 사이트 감소)"
            ));
        }
    }
    let mut grown: Vec<String> = Vec::new();
    for (file, &now) in &current {
        match baseline.get(file.as_str()) {
            Some(&base) if now > base => {
                grown.push(format!("  ↑ {SCAN_ROOT}/{file}: {base} → {now}"));
            }
            None => grown.push(format!("  + {SCAN_ROOT}/{file}: 0 → {now} (신규)")),
            _ => {}
        }
    }

    assert!(
        violations.is_empty(),
        "무효화 밀도가 원장 아래로 내려갔다 {}건 (#2724):\n{}\n\n\
         한 함수 안에 무효화 갈래가 여럿일 때 일부만 지우면 함수 단위 검사는 통과한다.\n\
         PR #2704 의 커넥터 곡선 갈래 누락이 실제로 그 형태였다(3갈래 중 1갈래만 방어).\n\
         무효화를 의도적으로 이관·통합했다면 INVALIDATION_LEDGER 를 갱신하라.{}",
        violations.len(),
        violations.join("\n"),
        if grown.is_empty() {
            String::new()
        } else {
            format!("\n\n참고(증가 — 실패 아님):\n{}", grown.join("\n"))
        },
    );

    if !grown.is_empty() {
        println!(
            "[#2724] 무효화 밀도 증가 — 원장 갱신 권장:\n{}",
            grown.join("\n")
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 검사 5 — 가드 자기검사
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn guard_scanner_self_check() {
    let all = collect_functions();
    let scoped = scoped_mutators(&all);
    assert!(
        scoped.len() >= MIN_PUB_MUT_SELF_METHODS,
        "스캐너가 범위 내 `pub fn (&mut self)` 를 {}개만 찾았다(하한 {}).\n\
         경로·파싱이 깨지면 검사 1~4 가 대상 0건으로 **공허하게 통과**한다.\n\
         구조 변경이 실제라면 하한을 의식적으로 낮춰라.",
        scoped.len(),
        MIN_PUB_MUT_SELF_METHODS,
    );

    let total: usize = measure_invalidation_sites().values().sum();
    assert!(
        total >= MIN_TOTAL_INVALIDATION_SITES,
        "무효화 사이트 총합이 {}건이다(하한 {}). 스캐너 손상 또는 대규모 무효화 제거.",
        total,
        MIN_TOTAL_INVALIDATION_SITES,
    );

    // 스캐너 자체의 최소 정합 — 앵커 함수가 기대대로 분류되는지.
    let connector: Vec<&&FnItem> = scoped
        .iter()
        .filter(|f| f.file == "commands/object_ops/connector.rs")
        .collect();
    assert_eq!(
        connector.len(),
        3,
        "#2698 의 커넥터 뮤테이터 3개를 찾지 못했다 — 스캐너 정합 실패(발견: {:?})",
        connector.iter().map(|f| &f.name).collect::<Vec<_>>(),
    );
    for f in &connector {
        assert!(
            invalidates(&f.body),
            "connector.rs::{}() 가 무효화하지 않는다 — #2698 회귀",
            f.name
        );
    }
}
