//! 문서 핵심 도메인 모델
//!
//! HWP 문서의 도메인 상태와 로직을 캡슐화한다.
//! WASM/PyO3/MCP 등 어떤 어댑터에서도 독립적으로 사용할 수 있다.

pub(crate) mod helpers;
pub(crate) use helpers::*;

pub mod builders;
mod commands;
pub mod converters;
pub(crate) mod html_table_import;
pub mod queries;
pub mod table_calc;
pub mod validation;

use crate::model::document::Document;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::model::table::TableTransposeData;
use crate::renderer::composer::ComposedParagraph;
use crate::renderer::height_measurer::{MeasuredSection, MeasuredTable};
use crate::renderer::layout::LayoutEngine;
use crate::renderer::pagination::PaginationResult;
use crate::renderer::render_tree::PageRenderTree;
use crate::renderer::style_resolver::ResolvedStyleSet;
use crate::renderer::DEFAULT_DPI;
use std::cell::RefCell;
use std::collections::HashMap;

/// 기본 폰트 fallback 경로
pub const DEFAULT_FALLBACK_FONT: &str = "/usr/share/fonts/truetype/nanum/NanumGothic.ttf";

/// 내부 클립보드 데이터
pub(crate) struct ClipboardData {
    /// 복사된 문단들 (서식 정보 포함)
    pub(crate) paragraphs: Vec<Paragraph>,
    /// 플레인 텍스트
    pub(crate) plain_text: String,
}

/// 표 셀 행/열 바꿈 전용 내부 버퍼
pub(crate) struct TableTransposeClipboard {
    pub(crate) data: TableTransposeData,
}

/// HWP 문서 핵심 도메인 모델
///
/// 문서 데이터, 레이아웃 상태, 설정, 캐시를 포함한다.
/// WASM 바인딩 없이 순수 Rust 타입만 사용한다.
pub struct DocumentCore {
    /// IR 문서
    pub(crate) document: Document,
    /// 페이지 분할 결과
    pub(crate) pagination: Vec<PaginationResult>,
    /// 해소된 스타일 세트
    pub(crate) styles: ResolvedStyleSet,
    /// 구역별 구성된 문단 목록
    pub(crate) composed: Vec<Vec<ComposedParagraph>>,
    /// [#2004] 부동(tac=false) 전면 이미지 스택을 인라인(tac=true)으로 재분류한 render-전용
    /// 문단/구성. 섹션별 Some 이면 pagination·layout 이 원본 대신 이 정규화본을 사용한다.
    /// **원본 `document` 는 무손상** → 직렬화(save) 정합 유지. paginate 시 재계산.
    pub(crate) render_normalized: Vec<
        Option<(
            Vec<crate::model::paragraph::Paragraph>,
            Vec<ComposedParagraph>,
        )>,
    >,
    /// DPI
    pub(crate) dpi: f64,
    /// 대체 폰트 경로
    pub(crate) fallback_font: String,
    /// 레이아웃 엔진 (자동 번호 카운터 포함)
    pub(crate) layout_engine: LayoutEngine,
    /// 내부 클립보드
    pub(crate) clipboard: Option<ClipboardData>,
    /// 표 셀 행/열 바꿈 복사 버퍼
    pub(crate) table_transpose_clipboard: Option<TableTransposeClipboard>,
    /// [Task #1161] 떠 있는 개체(treat_as_char=false) 반복 붙여넣기 cascade 카운터.
    /// 새 컨트롤 복사 시 0 으로 리셋, 붙여넣기마다 +1 하여 위치 오프셋 누적(한컴 정합).
    pub(crate) paste_cascade_count: u32,
    /// 문단부호(¶) 표시 여부
    pub(crate) show_paragraph_marks: bool,
    /// 조판부호 표시 여부 (개체 마커 [표]/[그림] 등, 문단부호 포함)
    pub(crate) show_control_codes: bool,
    /// 투명선 표시 여부
    pub(crate) show_transparent_borders: bool,
    /// 잘림 보기 (body/셀 클리핑 활성화 여부)
    pub(crate) clip_enabled: bool,
    /// 디버그 오버레이 표시 여부 (문단/표 경계 + pi/ci 라벨)
    pub(crate) debug_overlay: bool,
    /// LINE_SEG vpos-reset 강제 분리 적용 여부 (페이지네이션 옵션)
    pub(crate) respect_vpos_reset: bool,
    /// 구역별 표 측정 데이터 (페이지네이션 결과 보존)
    pub(crate) measured_tables: Vec<Vec<MeasuredTable>>,
    /// 구역별 dirty 플래그 (true = 재페이지네이션 필요)
    pub(crate) dirty_sections: Vec<bool>,
    /// 구역별 측정 캐시 (증분 측정용)
    pub(crate) measured_sections: Vec<MeasuredSection>,
    /// 구역별 문단 dirty 비트맵.
    /// None = 전체 dirty (초기 로드 또는 전체 재구성 시).
    /// Some(vec) = vec[para_idx] = true이면 해당 문단만 재측정.
    pub(crate) dirty_paragraphs: Vec<Option<Vec<bool>>>,
    /// 구역별 문단→단 인덱스 매핑 (페이지네이션에서 결정)
    /// para_column_map[section_idx][para_idx] = column_index
    pub(crate) para_column_map: Vec<Vec<u16>>,
    /// 페이지별 렌더 트리 캐시 (지연 구축, 부분 무효화)
    pub(crate) page_tree_cache: RefCell<Vec<Option<PageRenderTree>>>,
    /// Batch 모드 플래그 — true이면 paginate() 스킵
    pub(crate) batch_mode: bool,
    /// 이벤트 로그 (Command 실행 시 누적)
    pub(crate) event_log: Vec<DocumentEvent>,
    /// 글상자 오버플로우 연결 캐시 (섹션별, 지연 계산)
    pub(crate) overflow_links_cache:
        RefCell<HashMap<usize, Vec<queries::doc_tree_nav::OverflowLink>>>,
    /// Undo/Redo용 Document 스냅샷 저장소 (ID → Document 클론)
    pub(crate) snapshot_store: Vec<(u32, Document)>,
    /// 다음 스냅샷 ID
    pub(crate) next_snapshot_id: u32,
    /// 머리말/꼬리말 감추기: (global_page_index, is_header) 조합
    pub(crate) hidden_header_footer: std::collections::HashSet<(u32, bool)>,
    /// 파일 이름 (머리말/꼬리말 필드 치환용)
    pub(crate) file_name: String,
    /// 현재 활성 필드 위치 (커서가 진입한 누름틀 — 안내문 렌더링 스킵용)
    /// (section_idx, para_idx, field_control_idx)
    pub(crate) active_field: Option<ActiveFieldInfo>,
    /// 구역별 문단 인덱스 오프셋 (삽입=+N, 삭제=-N, 페이지네이션 수렴 감지용)
    /// paginate() 후 리셋.
    pub(crate) para_offset: Vec<i32>,
    /// 원본 파일 형식 (HWP/HWPX) — 저장 시 형식 분기용
    pub(crate) source_format: crate::parser::FileFormat,
    /// HWPX 비표준 감지 등 문서 검증 경고.
    /// `from_bytes` 에서 자동 생성되며, 사용자 고지·선택적 reflow 에 사용 (#177).
    pub(crate) validation_report: validation::ValidationReport,
}

/// `DocumentCore` 는 스레드 경계 너머로 소유될 수 있어야 한다 — native 소비자(MCP 서버,
/// 워커 풀)가 `Mutex<DocumentCore>` 를 다른 스레드로 보낸다. 내부 캐시 어딘가에 `Rc` 나
/// 다른 `!Send` 타입이 들어오면 이 단언이 컴파일 타임에 깨진다.
/// 내부 가변성(`Cell`/`RefCell`)은 `!Sync` 만 만들 뿐 `Send` 는 유지하므로 여기서 의도대로 통과한다.
#[cfg(not(target_arch = "wasm32"))]
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<DocumentCore>();
};

/// 활성 필드 위치 정보
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveFieldInfo {
    pub section_idx: usize,
    pub para_idx: usize,
    /// field_ranges의 control_idx (controls[] 내 Field 컨트롤 인덱스)
    pub control_idx: usize,
    /// 셀 내부 필드인 경우의 전체 경로
    /// 단일 표: vec![(parent_para_idx, ctrl, cell)]
    /// 중첩 표: vec![(outer_ctrl, outer_cell, ..), (inner_ctrl, inner_cell, ..)]
    /// parent_para_idx는 별도 필드에 포함하지 않고 첫 번째 요소의 context로 사용
    pub cell_path: Option<Vec<(usize, usize, usize)>>, // Vec<(parent_para_idx_or_ctrl, ctrl_or_cell, cell_or_para)>
}

impl DocumentCore {
    /// 총 페이지 수를 반환한다.
    pub fn page_count(&self) -> u32 {
        self.pagination
            .iter()
            .map(|pr| pr.pages.len() as u32)
            .sum::<u32>()
            .max(1)
    }

    /// 문서 정보를 JSON 문자열로 반환한다.
    pub fn get_document_info(&self) -> String {
        use crate::renderer::style_resolver::resolve_font_substitution;

        let mut fonts = std::collections::BTreeSet::new();
        for (lang_idx, lang_fonts) in self.document.doc_info.font_faces.iter().enumerate() {
            for font in lang_fonts {
                let resolved = resolve_font_substitution(&font.name, font.alt_type, lang_idx)
                    .unwrap_or(&font.name);
                fonts.insert(resolved.to_string());
            }
        }
        let fonts_json: Vec<String> = fonts
            .iter()
            .map(|f| {
                // 폰트 이름의 특수문자를 JSON 이스케이프 처리
                let escaped: String = f
                    .chars()
                    .flat_map(|c| match c {
                        '"' => vec!['\\', '"'],
                        '\\' => vec!['\\', '\\'],
                        '\n' => vec!['\\', 'n'],
                        '\r' => vec!['\\', 'r'],
                        '\t' => vec!['\\', 't'],
                        c if c < '\x20' => vec![],
                        c => vec![c],
                    })
                    .collect();
                format!("\"{}\"", escaped)
            })
            .collect();

        let escaped_fallback: String = self
            .fallback_font
            .chars()
            .flat_map(|c| match c {
                '"' => vec!['\\', '"'],
                '\\' => vec!['\\', '\\'],
                c => vec![c],
            })
            .collect();
        format!(
            "{{\"version\":\"{}.{}.{}.{}\",\"sectionCount\":{},\"pageCount\":{},\"encrypted\":{},\"hwp3Variant\":{},\"fallbackFont\":\"{}\",\"fontsUsed\":[{}]}}",
            self.document.header.version.major,
            self.document.header.version.minor,
            self.document.header.version.build,
            self.document.header.version.revision,
            self.document.sections.len(),
            self.page_count(),
            self.document.header.encrypted,
            self.document.is_hwp3_variant,
            escaped_fallback,
            fonts_json.join(","),
        )
    }

    /// 이벤트 로그를 JSON 배열로 직렬화한다.
    pub fn serialize_event_log(&self) -> String {
        crate::model::event::serialize_event_log(&self.event_log)
    }

    /// DPI를 설정하고 스타일을 재해소한 후 재페이지네이션한다.
    pub fn set_dpi(&mut self, dpi: f64) {
        use crate::renderer::style_resolver::resolve_styles_with_variant;
        self.dpi = dpi;
        self.styles = resolve_styles_with_variant(
            &self.document.doc_info,
            dpi,
            self.document.is_hwp3_variant,
        );
        self.paginate();
    }

    /// 빈 문서를 생성한다 (테스트/미리보기용).
    pub fn new_empty() -> Self {
        DocumentCore {
            document: Document::default(),
            pagination: Vec::new(),
            styles: ResolvedStyleSet::default(),
            composed: Vec::new(),
            render_normalized: Vec::new(),
            dpi: DEFAULT_DPI,
            fallback_font: DEFAULT_FALLBACK_FONT.to_string(),
            layout_engine: LayoutEngine::new(DEFAULT_DPI),
            clipboard: None,
            table_transpose_clipboard: None,
            paste_cascade_count: 0,
            show_paragraph_marks: false,
            show_control_codes: false,
            show_transparent_borders: false,
            clip_enabled: true,
            debug_overlay: false,
            respect_vpos_reset: false,
            measured_tables: Vec::new(),
            dirty_sections: Vec::new(),
            measured_sections: Vec::new(),
            dirty_paragraphs: Vec::new(),
            para_column_map: Vec::new(),
            page_tree_cache: RefCell::new(Vec::new()),
            batch_mode: false,
            event_log: Vec::new(),
            overflow_links_cache: RefCell::new(HashMap::new()),
            snapshot_store: Vec::new(),
            next_snapshot_id: 0,
            hidden_header_footer: std::collections::HashSet::new(),
            file_name: String::new(),
            active_field: None,
            para_offset: Vec::new(),
            source_format: crate::parser::FileFormat::Hwp,
            validation_report: validation::ValidationReport::new(),
        }
    }

    /// 문서 검증 리포트에 대한 참조를 반환한다.
    ///
    /// `from_bytes` 시점에 HWPX 비표준 lineseg 감지가 수행되며, 경고가 있으면
    /// 사용자에게 고지되어야 한다. 자동 reflow 는 적용되지 않고 사용자가
    /// 명시적으로 `reflow_linesegs_on_demand()` 를 호출해야 보정된다.
    pub fn validation_report(&self) -> &validation::ValidationReport {
        &self.validation_report
    }
}
