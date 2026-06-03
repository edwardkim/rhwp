# 구현계획서: HWPX lineseg 비표준 감지·고지 + 사용자 동의 기반 보정

- **타스크**: [#177](https://github.com/edwardkim/rhwp/issues/177)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task177`
- **선행 문서**: `mydocs/plans/task_m100_177.md` (수행계획서)
- **관련 Discussion**: [#188](https://github.com/edwardkim/rhwp/discussions/188)

## 1. 설계 확정 (작업지시자 결정 반영)

| 항목 | 결정 | 근거 |
|---|---|---|
| Validation 구조 위치 | **Document 별도 구조** | IR 순수성 유지, 검증 결과를 IR과 분리 |
| Reflow 기본 동작 | **명백한 경우만 유지** (현행 `needs_line_seg_reflow` 유지) | 기존 동작 보존으로 영향도 축소, 사용자 선택이 핵심 |
| 구현 단계 | **4단계** | 수행계획서 초안 유지 |

## 2. 공통 원칙

1. **기존 동작 최대 보존**: 현행 `reflow_zero_height_paragraphs` 와 `needs_line_seg_reflow` 의 조건(`line_segs.len()==1 && line_height==0`) 은 **그대로 유지**. 이미 "매우 명백한 경우" 만 트리거하므로 기업 윤리 원칙과도 일치
2. **경고는 항상 기록**: reflow 여부와 무관하게 비표준 감지 시 경고를 `ValidationReport` 에 기록
3. **Serializer는 원본 보존**: `Paragraph.line_segs` 가 비어있지 않으면 그대로 출력. 새 부정확 값 생성 금지
4. **사용자 선택 존중**: reflow on-demand API 제공, 기본값은 **자동 호출 안함**

## 3. Stage 1 — 감지 인프라 (ValidationReport + 검증 규칙)

### 3.1 신규 구조체

**새 파일**: `src/document_core/validation.rs`

```rust
//! 문서 검증 리포트 — HWPX 비표준 감지 경고 기록.
//!
//! IR과 분리된 별도 구조로 관리하여 IR 순수성 유지.
//! Document 로드 시 자동 생성되며, 사용자에게 고지하고 명시적 선택 시 reflow 적용.

use std::fmt;

/// 검증 리포트의 한 항목 — 문단 경로 + 경고 종류.
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// 섹션 인덱스
    pub section_idx: usize,
    /// 문단 인덱스 (섹션 내)
    pub paragraph_idx: usize,
    /// 표 셀 내부 문단일 경우 셀 경로 (table_ctrl_idx, row, col, inner_para_idx)
    pub cell_path: Option<CellPath>,
    /// 경고 종류
    pub kind: WarningKind,
}

#[derive(Debug, Clone, Copy)]
pub struct CellPath {
    pub table_ctrl_idx: usize,
    pub row: u16,
    pub col: u16,
    pub inner_para_idx: usize,
}

/// 경고 종류 — 비표준 감지 유형.
#[derive(Debug, Clone, PartialEq)]
pub enum WarningKind {
    /// lineseg 배열이 비어있음 (한컴은 reflow 하지만 rhwp는 겹침 발생 가능)
    LinesegArrayEmpty,
    /// lineseg 가 1개만 있고 line_height=0 — 명백한 "미계산 상태"
    LinesegUncomputed,
    /// lineseg 개수가 예상과 불일치 (문단 내 '\n' 개수와 다름)
    LinesegCountMismatch { expected: usize, actual: usize },
    /// lineseg 의 textpos 가 UTF-16 offset 경계와 안 맞음
    LinesegTextposMismatch { seg_idx: usize, textpos: u32 },
}

impl fmt::Display for WarningKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WarningKind::*;
        match self {
            LinesegArrayEmpty => write!(f, "lineseg 배열이 비어있음"),
            LinesegUncomputed => write!(f, "lineseg 가 미계산 상태 (line_height=0)"),
            LinesegCountMismatch { expected, actual } =>
                write!(f, "lineseg 개수 불일치: 예상 {}, 실제 {}", expected, actual),
            LinesegTextposMismatch { seg_idx, textpos } =>
                write!(f, "lineseg[{}]의 textpos={} 가 UTF-16 경계와 불일치", seg_idx, textpos),
        }
    }
}

/// 문서 검증 리포트.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    pub fn new() -> Self { Self::default() }
    pub fn is_empty(&self) -> bool { self.warnings.is_empty() }
    pub fn len(&self) -> usize { self.warnings.len() }
    pub fn push(&mut self, w: ValidationWarning) { self.warnings.push(w); }

    /// 경고 종류별 개수 요약.
    pub fn summary(&self) -> std::collections::HashMap<String, usize> {
        let mut map = std::collections::HashMap::new();
        for w in &self.warnings {
            let key = format!("{}", w.kind);
            *map.entry(key).or_insert(0) += 1;
        }
        map
    }
}
```

**`DocumentCore` 필드 추가**: `src/document_core/mod.rs`

```rust
pub struct DocumentCore {
    // ... 기존 필드
    /// HWPX 비표준 감지 등 문서 검증 경고.
    /// Document 로드 직후 채워지며, 사용자 고지·선택적 reflow에 사용.
    pub(crate) validation_report: crate::document_core::validation::ValidationReport,
}
```

### 3.2 검증 규칙 구현

**위치**: `src/document_core/commands/document.rs::from_bytes` 내부, `reflow_zero_height_paragraphs` 호출 **직전**에 추가

```rust
// 비표준 감지 — IR 수정 없이 경고만 기록
let validation_report = Self::validate_linesegs(&document);
```

```rust
fn validate_linesegs(document: &Document) -> ValidationReport {
    let mut report = ValidationReport::new();
    for (si, section) in document.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            Self::check_paragraph_linesegs(para, si, pi, None, &mut report);
            // 표 셀 내부 문단도 재귀 검사
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let Control::Table(table) = ctrl {
                    for cell in &table.cells {
                        for (inner_pi, cell_para) in cell.paragraphs.iter().enumerate() {
                            let cell_path = CellPath {
                                table_ctrl_idx: ci,
                                row: cell.row,
                                col: cell.col,
                                inner_para_idx: inner_pi,
                            };
                            Self::check_paragraph_linesegs(
                                cell_para, si, pi, Some(cell_path), &mut report,
                            );
                        }
                    }
                }
            }
        }
    }
    report
}

fn check_paragraph_linesegs(
    para: &Paragraph,
    section_idx: usize,
    para_idx: usize,
    cell_path: Option<CellPath>,
    report: &mut ValidationReport,
) {
    // 규칙 1: lineseg 배열이 비어있는 경우
    if para.line_segs.is_empty() && !para.text.is_empty() {
        report.push(ValidationWarning {
            section_idx, paragraph_idx: para_idx, cell_path,
            kind: WarningKind::LinesegArrayEmpty,
        });
        return; // 후속 규칙 건너뜀
    }
    // 규칙 2: 미계산 상태 (기존 needs_line_seg_reflow 와 동일 조건)
    if para.line_segs.len() == 1 && para.line_segs[0].line_height == 0 {
        report.push(ValidationWarning {
            section_idx, paragraph_idx: para_idx, cell_path,
            kind: WarningKind::LinesegUncomputed,
        });
    }
    // 규칙 3/4는 Stage 4 에서 실문서 검증 후 추가 (false positive 조정 필요)
}
```

### 3.3 단위 테스트 (Stage 1)

`src/document_core/validation.rs` 내부 `#[cfg(test)]`:

1. `report_default_is_empty` — 기본값 검증
2. `summary_groups_by_kind` — 경고 종류별 집계
3. `warning_display_messages` — `Display` 구현 확인

`src/document_core/commands/document.rs` 내부 `#[cfg(test)]`:

4. `validate_detects_empty_linesegs` — 빈 lineseg 배열 검출
5. `validate_detects_uncomputed_lineseg` — line_height=0 검출
6. `validate_recurses_into_table_cells` — 셀 내부 문단도 검증

### 3.4 완료 기준

- [ ] `src/document_core/validation.rs` 신규 생성
- [ ] `DocumentCore::validation_report` 필드 추가, `from_bytes` 에서 채움
- [ ] `validate_linesegs` 메서드 추가, 2가지 규칙 검출
- [ ] 단위 테스트 6개 통과
- [ ] 기존 850개 테스트 유지

---

## 4. Stage 2 — Serializer 원본 보존

### 4.1 `src/serializer/hwpx/section.rs` 변경

**핵심 원칙**: IR의 `Paragraph.line_segs` 가 비어있지 않으면 **그대로** 직렬화. 기존 `push_lineseg(out, textpos, vertpos)` 의 정적 `VERT_STEP=1600` 누적 로직을 **제거**하고 IR 값 사용으로 대체.

#### 현행 문제 지점

```rust
// 현재 (line 114~119)
fn push_lineseg(out: &mut String, textpos: u32, vertpos: u32) {
    out.push_str(&format!(
        r#"<hp:lineseg textpos="{}" vertpos="{}" vertsize="1000" textheight="1000" baseline="850" spacing="600" horzpos="0" horzsize="{}" flags="{}"/>"#,
        textpos, vertpos, HORZ_SIZE, LINE_FLAGS,
    ));
}
```

`vertsize`, `textheight`, `baseline`, `spacing`, `horzsize`, `flags` 모두 **정적 하드코딩값**. IR에 담긴 실제 값과 무관하게 덮어씀.

#### 변경 후 로직

```rust
fn render_lineseg_array(para: &Paragraph) -> String {
    if para.line_segs.is_empty() {
        // IR 에 아예 없으면 빈 요소 (한컴 DocumentCore::from_bytes 의
        // reflow_zero_height_paragraphs 가 채워주므로 실제로는 발생 드묾)
        return String::new();
    }
    let mut out = String::new();
    for seg in &para.line_segs {
        out.push_str(&format!(
            r#"<hp:lineseg textpos="{}" vertpos="{}" vertsize="{}" textheight="{}" baseline="{}" spacing="{}" horzpos="{}" horzsize="{}" flags="{}"/>"#,
            seg.text_start,
            seg.vertical_pos,
            seg.line_height,   // 현행: 1000 하드코딩
            seg.text_height,   // 현행: 1000
            seg.baseline_distance, // 현행: 850
            seg.line_spacing,  // 현행: 600
            seg.column_start,  // 현행: 0
            seg.segment_width, // 현행: HORZ_SIZE (42520)
            seg.tag,           // 현행: LINE_FLAGS (393216)
        ));
    }
    out
}
```

**결과**: 한컴 원본의 lineseg 값이 rhwp 저장본에서 그대로 보존됨. 원본이 비표준이면 rhwp도 비표준을 보존하지만 **rhwp가 새로 만들어낸 것이 아님**.

### 4.2 기존 `render_paragraph_parts` 수정

기존 함수가 `\n` 만날 때마다 `push_lineseg` 를 호출하여 lineseg 를 생성하는데, 이를 **IR 기반 출력으로 전환**. 다만:

- **편집 후 IR의 `line_segs` 가 stale 한 경우**: 파서가 파싱 직후 `needs_line_seg_reflow` 조건에 해당하면 `reflow_line_segs` 로 채움. 편집 시나리오에서도 `reflow_zero_height_paragraphs` 가 채워주므로 serializer 시점에는 항상 값이 있음
- **예외 케이스**: IR에 아예 `line_segs` 가 없는 `Document::default()` + 수동 Paragraph 생성 시나리오 → 빈 요소로 fallback

```rust
fn render_paragraph_parts(para: &Paragraph, vert_start: u32) -> (String, String, u32) {
    let t_xml = render_hp_t_content(&para.text);
    let linesegs_xml = render_lineseg_array(para);
    let vert_end = estimate_vert_end(para, vert_start);
    (t_xml, linesegs_xml, vert_end)
}
```

여기서 `render_hp_t_content` 는 기존 `\t`/`\n`/텍스트 변환 로직만 분리한 것. `push_lineseg` 호출은 제거.

### 4.3 기존 테스트 조정

현재 `src/serializer/hwpx/mod.rs` 의 `linesegs_emitted_per_linebreak` 테스트는 **정적 하드코딩값 기반**:

```rust
// 기존 기대값 (하드코딩 vertpos)
assert!(xml.contains(r#"textpos="0" vertpos="0""#));
assert!(xml.contains(r#"textpos="2" vertpos="1600""#));
assert!(xml.contains(r#"textpos="4" vertpos="3200""#));
```

이 테스트는 **IR 기반 출력으로 전환 후 의미가 바뀜**:
- `Document::default()` + 수동 Paragraph 생성 시 IR의 `line_segs` 는 **존재할 수도 없을 수도 있음**
- `DocumentCore::from_bytes` 를 거치지 않은 테스트이므로 `reflow_zero_height_paragraphs` 호출이 없음

**조정 방향**:
- (a) 테스트를 `DocumentCore::from_bytes` 를 거치도록 재작성 (정식 경로)
- (b) 테스트에서 직접 `Paragraph.line_segs` 를 채우고 기대값 검증
- (c) 테스트를 제거하고 라운드트립 통합 테스트로 대체

**제안**: (b) 가 가장 명확. 테스트가 **"IR이 주면 그대로 출력한다"** 를 검증하도록 재작성.

### 4.4 단위 테스트 (Stage 2)

1. `lineseg_array_reflects_ir` — IR에 담긴 lineseg 가 XML에 정확히 반영
2. `lineseg_attributes_all_from_ir` — vertsize/textheight/baseline/spacing/horzsize/flags 모두 IR 값 사용
3. `empty_linesegs_produces_no_entries` — IR에 비어있으면 lineseg 요소 없음
4. 기존 `linesegs_emitted_per_linebreak` 재작성 (IR 기반)

### 4.5 통합 테스트 (Stage 2)

새 `tests/hwpx_roundtrip_integration.rs` 에 추가:

```rust
#[test]
fn task177_lineseg_preserved_on_roundtrip() {
    // 원본 HWPX 파싱 → serialize → 재파싱 → line_segs 값이 원본과 동일한지
    let bytes = include_bytes!("../samples/hwpx/ref/ref_text.hwpx");
    let doc1 = parse_hwpx(bytes).unwrap();
    let out = serialize_hwpx(&doc1).unwrap();
    let doc2 = parse_hwpx(&out).unwrap();
    // 첫 문단의 line_segs 비교
    let p1 = &doc1.sections[0].paragraphs[0];
    let p2 = &doc2.sections[0].paragraphs[0];
    assert_eq!(p1.line_segs.len(), p2.line_segs.len());
    for (a, b) in p1.line_segs.iter().zip(p2.line_segs.iter()) {
        assert_eq!(a.vertical_pos, b.vertical_pos);
        assert_eq!(a.line_height, b.line_height);
        // ... 기타 필드
    }
}
```

### 4.6 완료 기준

- [ ] `render_lineseg_array` 신규 함수로 교체, `push_lineseg` 제거
- [ ] `render_paragraph_parts` 시그니처 변경 (Paragraph 참조 받도록)
- [ ] 기존 `linesegs_emitted_per_linebreak` 재작성
- [ ] 신규 단위 테스트 3~4개 추가
- [ ] 통합 테스트 `task177_lineseg_preserved_on_roundtrip` 추가
- [ ] 기존 라운드트립 테스트 (stage0/1/5) 유지

---

## 5. Stage 3 — Reflow on-demand + WASM API + rhwp-studio 모달 UI

### 5.1 설계 결정 재확인

작업지시자 결정:
- Reflow 기본 동작 = **명백한 경우만 유지**. 기존 `needs_line_seg_reflow` 조건(`line_segs.len()==1 && line_height==0`) 유지
- 사용자 고지 방식 = **rhwp-studio 모달창** (2026-04-18 확정)

**따라서 자동 reflow 자체는 변경 없음**. 추가 작업은:

1. **사용자가 명시적으로 "전체 reflow 요청" 시** 더 넓은 범위(모든 `LinesegArrayEmpty` / `LinesegUncomputed` 경고)를 reflow
2. **WASM API** 로 validation_report 조회 + reflow 실행
3. **rhwp-studio 모달 UI** — 문서 로드 시 경고가 있으면 모달창으로 알리고 사용자 선택

### 5.2 `DocumentCore` 에 신규 메서드

**파일**: `src/document_core/commands/document.rs`

```rust
impl DocumentCore {
    /// 사용자 명시 요청에 의한 전체 lineseg reflow.
    ///
    /// `validation_report` 에 기록된 경고 대상 문단들을 전부 reflow 한다.
    /// 기본 파싱 경로의 `reflow_zero_height_paragraphs` 와 달리 이 메서드는
    /// 사용자가 "자동 보정" 버튼을 눌렀을 때만 호출되어야 한다.
    pub fn reflow_linesegs_on_demand(&mut self) -> usize {
        let mut reflowed = 0;
        let styles = &self.styles;
        let dpi = self.dpi;
        for section in &mut self.document.sections {
            let page_def = &section.section_def.page_def;
            let column_def = Self::find_initial_column_def(&section.paragraphs);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, dpi);
            let col_width = layout.column_areas.first()
                .map(|a| a.width)
                .unwrap_or(layout.body_area.width);
            for para in &mut section.paragraphs {
                if Self::needs_reflow_broadly(para) {
                    let para_style = styles.para_styles.get(para.para_shape_id as usize);
                    let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                    let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                    let available_width = (col_width - margin_left - margin_right).max(1.0);
                    reflow_line_segs(para, available_width, styles, dpi);
                    reflowed += 1;
                }
                // 셀 내부 문단도 동일 처리 (기존 코드와 동일 패턴)
            }
        }
        self.mark_dirty(); // 저장 시점에 반영
        reflowed
    }

    /// 넓은 기준의 "reflow 필요" 판정 — 사용자 명시 요청 시 사용.
    /// 기존 `needs_line_seg_reflow` 보다 더 많은 케이스 포함:
    /// - line_segs 가 비어있음 (text 가 있는데도)
    /// - line_segs 가 1개 + line_height=0 (기존 조건)
    fn needs_reflow_broadly(para: &Paragraph) -> bool {
        if !para.text.is_empty() && para.line_segs.is_empty() { return true; }
        Self::needs_line_seg_reflow(para)
    }

    /// 검증 리포트 참조.
    pub fn validation_report(&self) -> &ValidationReport {
        &self.validation_report
    }
}
```

### 5.3 WASM API 추가

**파일**: `src/wasm_api.rs`

```rust
#[wasm_bindgen]
impl HwpDocument {
    /// 문서 검증 경고 목록을 JSON 문자열로 반환.
    ///
    /// 구조:
    /// {
    ///   "count": 3,
    ///   "summary": { "lineseg 미계산 상태": 2, "lineseg 배열이 비어있음": 1 },
    ///   "warnings": [ { "section": 0, "paragraph": 5, "kind": "LinesegUncomputed" }, ... ]
    /// }
    #[wasm_bindgen(js_name = getValidationWarnings)]
    pub fn get_validation_warnings(&self) -> String {
        let report = self.core.validation_report();
        // serde_json 또는 수동 직렬화
        format_report_as_json(report)
    }

    /// 사용자 명시 요청에 의한 전체 lineseg reflow.
    /// 반환: 실제로 reflow 된 문단 개수.
    #[wasm_bindgen(js_name = reflowLinesegs)]
    pub fn reflow_linesegs(&mut self) -> usize {
        self.core.reflow_linesegs_on_demand()
    }
}
```

### 5.4 rhwp-studio 모달 UI

**목표**: 문서 로드 완료 직후 `getValidationWarnings()` 호출 → 경고가 있으면 **모달창**으로 고지.

**위치**: `rhwp-studio/src/` — 문서 로드 파이프라인(예: `command/commands/file.ts` 의 open 완료 시점)에서 모달 트리거.

**모달 내용**:
- 제목: "HWPX 비표준 감지"
- 본문: "이 문서는 HWPX 명세를 일부 준수하지 않는 값을 포함합니다 (경고 N건). 렌더링 품질을 위해 자동 보정을 권장합니다."
- 경고 요약 표시 (경고 종류별 개수)
- 버튼 (왼쪽부터):
  - **[자동 보정] (기본 선택 · 강조)** — 사용자가 지시한 대로 이것이 Default.
    클릭 시 `reflowLinesegs()` 호출 → 렌더 재계산 → 모달 닫기
  - [그대로 보기] — 모달만 닫음
  - [상세 보기] — 경고 목록 펼침 (선택 UI, 상세 항목 나열)

**기본 선택**: **자동 보정**. 모달이 열리면 포커스가 [자동 보정] 버튼에 있어야 하며, Enter 키 입력 시 자동 보정이 실행된다.

**비침습 원칙**: 경고가 0건이면 모달 생성 안 함.

**코드 위치** (예상):
- 신규: `rhwp-studio/src/ui/validation-modal.ts` — 모달 생성·이벤트 바인딩
- 수정: 문서 로드 완료 훅(예: `wasm-bridge.ts` 의 `initFromBytes` 또는 `file.ts` 의 open 콜백)

### 5.5 단위 테스트 (Stage 3)

Rust 측 (wasm_api 및 document_core):

1. `reflow_on_demand_processes_empty_linesegs` — 빈 line_segs 문단이 reflow 됨
2. `reflow_on_demand_returns_count` — reflow 된 문단 수 정확 반환
3. `validation_report_accessor` — `validation_report()` 가 Stage 1 의 경고 반환
4. `needs_reflow_broadly_covers_empty_and_uncomputed` — 판정 조건 검증
5. `get_validation_warnings_returns_json` — JSON 포맷 검증

TypeScript 측 (rhwp-studio):

6. 수동 검증(E2E 또는 수동) — 실문서 로드 시 모달 표시 확인

### 5.6 완료 기준

- [ ] `DocumentCore::reflow_linesegs_on_demand()` 추가
- [ ] `DocumentCore::validation_report()` 접근자 추가 (Stage 1 에서 완료)
- [ ] WASM API `getValidationWarnings()`, `reflowLinesegs()` 추가
- [ ] rhwp-studio 모달 UI 신규 (`validation-modal.ts`)
- [ ] 문서 로드 훅에서 모달 트리거
- [ ] 기본 포커스 = [자동 보정] 버튼, Enter = 자동 보정 실행
- [ ] 경고 0건 시 모달 미생성 (비침습)
- [ ] Rust 단위 테스트 5개
- [ ] 기존 `reflow_zero_height_paragraphs` 동작 유지 (변경 없음)
- [ ] WASM 빌드 통과

---

## 6. Stage 4 — 통합 검증 + 문서화

### 6.1 실문서 회귀 테스트

작업지시자 제공 샘플 `samples/hwpx/hwpx-02.hwpx` 기반:

```rust
#[test]
fn task177_hwpx_02_linesegs_detected_and_preserved() {
    // 1. 원본 파싱 → validation_report 에 경고가 몇 개 있는지 확인 (baseline 기록)
    // 2. serialize → 재파싱 → 원본과 line_segs 동일
    // 3. reflow_linesegs_on_demand 호출 후 → line_segs 가 재계산됨 (line_height > 0 검증)
}
```

### 6.2 대형 샘플 false positive 조사

기존 대형 실문서 4건(`ref_table`, `form-002`, `2025 1Q/2Q`)에서 경고 개수를 측정:

- 기대: 0에 가깝거나 설명 가능한 수준
- false positive 가 많으면 규칙 완화

### 6.3 문서

- `mydocs/tech/hwpx_lineseg_validation.md` 신규 — 감지 규칙, reflow 동작, API 사용법
- `mydocs/working/task_m100_177_stage{1..4}.md` 단계별 보고서
- `mydocs/report/task_m100_177_report.md` 최종 보고서

### 6.4 완료 기준

- [ ] `hwpx-02.hwpx` 회귀 테스트 통과
- [ ] 대형 샘플 4건 false positive 측정·문서화
- [ ] 기술 문서 1건 (`hwpx_lineseg_validation.md`)
- [ ] 최종 결과 보고서 작성
- [ ] 전체 라이브러리 테스트 그린

---

## 7. Stage 간 의존 관계 및 순서

```
Stage 1 (감지 인프라)
   ↓
Stage 2 (Serializer 원본 보존) — Stage 1 경고 기록은 파싱 단계이므로 독립
   ↓
Stage 3 (Reflow on-demand + WASM API) — Stage 1 의 validation_report 접근자 필요
   ↓
Stage 4 (통합 검증 + 문서화)
```

Stage 1 과 Stage 2 는 서로 독립적이나, 관례대로 순차 진행.

## 8. 위험 요소

| 위험 | 완화 |
|---|---|
| `linesegs_emitted_per_linebreak` 등 기존 테스트 깨짐 | Stage 2 에서 재작성. 새 기대값은 "IR 기반" 으로 명확 |
| Serializer 원본 보존으로 Stage 4 대형 샘플 회귀 가능성 | 기존 8/0 통합 테스트 유지 + 신규 테스트. 회귀 시 규칙 완화 |
| `DocumentCore` 에 field 추가로 PartialEq/Clone 등 impl 영향 | `validation_report` 는 `Default` 있으므로 파급 최소 |
| WASM API 메서드 추가 시 JS 빌드 크기 증가 | 최소 필요 API(2개) 만 추가 |
| 편집 시 lineseg stale 해지는 기존 이슈 | 본 범위 외 (별도 이슈). 편집 경로는 `reflow_zero_height_paragraphs` 기존 동작에 의존 |

## 9. 수정·추가 파일 요약

| Stage | 수정 (기존) | 추가 (신규) |
|---|---|---|
| 1 | `src/document_core/mod.rs` (필드), `src/document_core/commands/document.rs` (from_bytes) | `src/document_core/validation.rs` |
| 2 | `src/serializer/hwpx/section.rs` | — |
| 3 | `src/document_core/commands/document.rs`, `src/wasm_api.rs` | — |
| 4 | 통합 테스트 | `mydocs/tech/hwpx_lineseg_validation.md`, 단계별 보고서, 최종 보고서 |

## 10. 승인 요청

본 구현계획서 승인 후 Stage 1 착수.
