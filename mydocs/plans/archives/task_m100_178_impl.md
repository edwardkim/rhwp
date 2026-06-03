---
타스크: #178 HWPX→HWP IR 매핑 어댑터 (정도 접근)
브랜치: local/task178
작성일: 2026-04-19
선행: mydocs/plans/task_m100_178.md (수행계획서, 승인됨)
---

# 구현계획서: HWPX→HWP IR 매핑 어댑터

## 0. 작업지시자 결정 사항 (수행계획서 §11)

| 질문 | 결정 | 영향 |
|---|---|---|
| Q1. 단계 분할 | **(C) 7단계 세분화** — 표 attr/raw_ctrl_data 와 셀 list_attr 을 분리 | Stage 4 → Stage 4 + Stage 5 |
| Q2. 자동 차이 추출 도구 | **(A) 신설** | Stage 1 에 `tools/hwpx_hwp_ir_diff.rs` 포함 |
| Q3. 자기 재로드 검증 | **(B) 명시 호출** (자동 X) | `serialize_hwp_with_verify(doc)` 별도 진입점, `serialize_hwp` 는 그대로 |

## 1. 핵심 설계 — 잘 작동하는 HWP 직렬화의 어깨 위에 서자

### 1.0 본 타스크의 정체성 (작업지시자 통찰)

> **"이미 hwpx 의 IR 에서 렌더링은 잘 됩니다. hwpx 를 저장할 때 hwp 의 직렬화를 이용한 것이 핵심입니다."**

본 타스크는 **신규 직렬화기를 만들지 않는다**. 이미 검증된 두 자산 위에 어댑터 한 겹만 얹는다:

| 자산 | 상태 | 역할 |
|---|---|---|
| HWPX 파서 → IR | 렌더링 정상 (rhwp 화면 OK) | 입력 |
| HWP 직렬화기 (`serializer/cfb_writer.rs`, `body_text.rs`, `control.rs`) | HWP→HWP 라운드트립 정상 | 출력 |
| **HWPX-IR → HWP-IR 어댑터 (본 타스크)** | 신규 | **다리** |

### 1.0.1 이 정체성에서 도출되는 절대 원칙

1. **HWP 직렬화기 코드 0줄 수정** — `serializer/cfb_writer.rs`, `body_text.rs`, `control.rs`, `header.rs`, `doc_info.rs` 중 어느 하나라도 수정 PR 이 발생하면 본 타스크 범위 위반. (필요하다면 별도 이슈로 분리.)
2. **어댑터는 IR 만 만진다** — 어댑터의 모든 출력은 `&mut Document` 의 필드 갱신뿐. 새 직렬 경로·새 바이트 작성기 도입 금지.
3. **검증 기준은 "HWP 직렬화기가 받아들이는 IR 모양"** — "HWPX 가 어떻게 채우는가" 가 아니라 "HWP 직렬화기가 어떤 필드를 어떻게 읽는가" 가 매핑 명세서. §1.3 의 6개 위치가 그 명세.
4. **이미 잘 되는 것은 건드리지 않는다** — HWP 출처 IR (즉 HWP 파서가 채워준 IR) 에 어댑터를 호출해도 비변경(idempotent + no-op). source_format 검사로 분기.

### 1.0.2 이 원칙의 직접 귀결 (§1.3 발견의 의미)

`section.raw_stream` 합성 불필요 발견은 단순한 최적화가 아니다 — **HWP 직렬화기가 이미 동적 경로를 가지고 있고, 그 동적 경로가 동작하므로 어댑터는 끼어들 필요가 없다**는 본 원칙의 적용. 이런 식으로 매핑 영역을 늘리지 않고 줄이는 것이 본 타스크의 성공 신호다.

### 1.1 신규 모듈 (1개)

`src/document_core/converters/mod.rs` (신규 디렉터리)
`src/document_core/converters/hwpx_to_hwp.rs` (신규 본체)

진입점:
```rust
pub fn convert_hwpx_to_hwp_ir(doc: &mut Document) -> AdapterReport;
```

전제: `doc.source_format == SourceFormat::Hwpx` 일 때만 호출. 호출자 책임으로 분기. 어댑터 자체는 idempotent — 같은 IR 에 두 번 호출해도 같은 결과.

### 1.2 기존 코드 변경 영역 (최소)

| 파일 | 변경 | 이유 |
|---|---|---|
| `src/document_core/mod.rs` | `pub mod converters;` 추가 | 모듈 노출 |
| `src/document_core/commands/document.rs:449` | `export_hwp_native` 그대로 — **변경 없음** | Q3 결정: 자동 호출 X |
| `src/document_core/commands/document.rs` | `export_hwp_with_adapter()` 신규 추가 | HWPX 출처 자동 어댑터 + 직렬화 |
| `src/document_core/commands/document.rs` | `serialize_hwp_with_verify()` 신규 추가 | Q3 결정: 명시 검증 진입점 |
| `src/wasm_api.rs:2786` 부근 | `export_hwp` 가 source_format 검사 후 어댑터 적용 분기 | UI 사용자 경로 |

`src/serializer/cfb_writer.rs`, `body_text.rs`, `control.rs` 는 **단 한 줄도 수정하지 않는다**. 안정성 보존.

### 1.3 매핑 명세서 — HWP 직렬화기가 IR 에서 무엇을 읽는가

본 표는 "HWPX 가 무엇을 안 채우는가" 가 아니라 **"HWP 직렬화기가 어떤 필드를 어떻게 소비하는가"** 의 직접 인용이다. 이 표가 본 타스크의 단 하나의 명세서다. 이 표 밖의 영역은 손대지 않는다.

검사 결과 (코드 직접 읽음):

| 위치 | 코드 | HWPX 출처 시 값 |
|---|---|---|
| `body_text.rs:28` | `if let Some(ref raw) = section.raw_stream { return raw.clone(); }` | `None` → 동적 경로 진입 (OK) |
| `body_text.rs:194` | `if para.raw_break_type != 0 { para.raw_break_type } else { match para.column_type { ... } }` | `0` → column_type 매핑 사용 (OK, HWPX 가 column_type 채움 가정) |
| `body_text.rs:215` | `if para.raw_header_extra.len() >= 10 { ... } else { write_u32(0) }` | 빈 Vec → instanceId 0 fallback (OK) |
| `control.rs:349` | `if !table.raw_ctrl_data.is_empty() { &table.raw_ctrl_data } else { &[] }` | 빈 Vec → **빈 ctrl_data 작성 (BUG: CommonObjAttr 0바이트)** |
| `control.rs:375` | `if table.raw_table_record_attr != 0 { ... } else { 재구성 from page_break + repeat_header }` | `0` → 재구성 (OK, page_break 채움 가정) |
| `control.rs:429` | `let list_attr: u32 = ((cell.text_direction as u32) << 16) \| (v_align_code << 21);` | apply_inner_margin 비트 16 누락 (BUG) |

→ **진짜 결손은 `table.raw_ctrl_data` 합성 + `cell.list_attr` bit 16** 이 핵심.
→ `section.raw_stream` 은 `None` 이어도 동적 경로로 OK (별도 합성 불필요).
→ lineseg vpos 는 페이지 폭주의 직접 원인일 가능성 높음 — 사전계산 필요.

이 발견으로 **수행계획서의 Stage 2 가 단순화**된다 (raw_stream 합성 불필요, ctrl_data 만 채우면 됨). §1.0.2 에서 설명한대로, 이는 "HWP 직렬화기 어깨 위에 서자" 원칙의 자연스러운 귀결이다 — 직렬화기가 이미 처리하는 영역에는 어댑터가 끼어들지 않는다.

## 2. 단계 분할 (7 Stage)

각 Stage 끝에는 통합 테스트 추가 + 회귀 테스트 그린 + 페이지 폭주 측정 + 단계별 보고서.

### Stage 1 — 진단 인프라 + 자동 차이 추출 도구

**목표**: 매핑이 실패한 영역을 자동 식별하는 도구 + 베이스라인 측정.

**신규 파일**:
- `src/document_core/converters/mod.rs` — `pub mod hwpx_to_hwp;` (어댑터 자체는 Stage 2 에서 본격 구현, Stage 1 은 빈 진입점만)
- `src/document_core/converters/hwpx_to_hwp.rs` — 빈 `convert_hwpx_to_hwp_ir(_: &mut Document) -> AdapterReport { ... }` (no-op + 영역별 카운터)
- `src/document_core/converters/diagnostics.rs` — IR 필드 비교 유틸
  ```rust
  pub struct IrFieldDiff {
      pub area: &'static str,            // "section.raw_stream" | "table.raw_ctrl_data" | "cell.list_attr_bit16" | ...
      pub hwpx_value: String,
      pub hwp_value: String,
      pub para_path: String,             // "sec=2,para=45"
  }
  pub fn diff_hwpx_vs_hwp(hwpx: &Document, hwp: &Document) -> Vec<IrFieldDiff>;
  pub fn diff_hwpx_vs_serializer_assumptions(hwpx: &Document) -> Vec<IrFieldDiff>;
  ```
- `tools/hwpx_hwp_ir_diff.rs` (cargo example 또는 bin) — CLI:
  ```
  cargo run --example hwpx_hwp_ir_diff -- a.hwpx b.hwp
  ```
  영역별 차이를 휴먼 리더블로 출력.

**신규 테스트** (`tests/hwpx_to_hwp_adapter.rs`):
- `baseline_page_count_explosion_hwpx_h_01` — hwpx-h-01.hwpx → export_hwp_native → DocumentCore::from_bytes → page_count 측정 (현재 209 기록)
- `baseline_page_count_explosion_hwpx_h_02` — hwpx-h-02.hwpx (현재 155 기록)
- `baseline_page_count_explosion_hwpx_h_03` — hwpx-h-03.hwpx (베이스라인 측정)
- `baseline_diff_inventory_hwpx_h_01` — `diff_hwpx_vs_serializer_assumptions(hwpx)` 호출, 영역별 카운트 출력 (assert 없음, 현황 기록)

**기존 폐기 테스트 복구**:
- `tests/hwpx_roundtrip_integration.rs` 의 `task178_*` 테스트 3건 (수행계획서 §1 의 것) — 동일 측정으로 재활성화. 단 assert 는 베이스라인 기준으로 작성 (회복 단계마다 갱신).

**완료 기준**:
- `cargo test hwpx_to_hwp_adapter::baseline` 그린 (베이스라인 기록)
- `cargo run --example hwpx_hwp_ir_diff -- samples/hwpx/hwpx-h-01.hwpx` 정상 동작
- 단위 테스트 회귀 0건

**일정**: 0.5일.

### Stage 2 — table.raw_ctrl_data 합성 + table.attr 재구성

**목표**: 표가 HWP 에서 정상 인식되도록 `raw_ctrl_data` (CommonObjAttr 직렬화) 합성.

**hwpx_to_hwp.rs 추가**:
```rust
fn synthesize_table_raw_ctrl_data(table: &mut Table) {
    if !table.raw_ctrl_data.is_empty() { return; }   // 이미 있으면 보존
    table.raw_ctrl_data = serialize_common_obj_attr(&table.common);
}
fn synthesize_table_attr(table: &mut Table) {
    if table.attr != 0 { return; }
    table.attr = pack_table_attr_from_common(&table.common);
}
```
`serialize_common_obj_attr` — `parser/control/common_obj_attr.rs` 의 역함수 (별도 헬퍼 모듈 `src/document_core/converters/common_obj_attr_writer.rs`).

**적용 범위**: `Document.sections[].paragraphs[].controls[]` 순회 + 셀 내부 `cell.paragraphs[].controls[]` 재귀.

**신규 테스트**:
- `synthesize_table_raw_ctrl_data_roundtrip` — HWPX 에서 로드한 표 1개 → 어댑터 적용 → bytes 작성 → 재파싱 → table.common 동일성 검증
- `regression_hwp_source_table_unchanged` — HWP 원본에서 로드한 doc 에 어댑터 호출 → raw_ctrl_data 비변경 (idempotent + HWP 출처 보호)

**완료 기준**:
- 위 테스트 그린
- `baseline_page_count_explosion_*` 의 페이지 수가 변화 (개선 방향, 완전 회복 안 돼도 진척이면 OK)
- HWP→HWP 라운드트립 회귀 0건

**일정**: 1.5~2일.

### Stage 3 — 셀 list_attr bit 16 (apply_inner_margin)

**목표**: HWPX `cell.apply_inner_margin` → `list_attr bit 16` 동기화.

**hwpx_to_hwp.rs 추가**:
```rust
fn synthesize_cell_list_attr_bit16(cell: &mut Cell) {
    // serializer/control.rs:429 가 text_direction(<<16) 와 v_align(<<21) 만 사용 → bit 16 의미 충돌
    // 해결: cell 에 별도 raw_list_attr 필드 두거나, raw_list_extra 활용
    // 결정: serializer 변경 금지 정책상, cell.raw_list_extra 에 추가 비트 패킹
    //       OR cell 모델에 apply_inner_margin 우선 반영 필드 도입
}
```
**설계 결정 (Stage 3 시작 시 확정)**:
- 옵션 A: `cell.raw_list_extra` 에 bit 16 보강 → serializer 가 raw_list_extra 그대로 이어붙이므로 안전
- 옵션 B: model 에 `cell.synthesized_list_attr_extra: u32` 신규 필드 → serializer 가 OR 로 합성
- **현재 잠정**: 옵션 A (모델 변경 없음, 어댑터에서만 처리). serializer 변경 0 정책에 부합.

단, 기존 `serializer/control.rs:429` 가 `list_attr` 를 0 부터 재구성하므로 raw_list_extra 의 비트가 list_attr 와 별개로 출력된다. → list_attr 직렬화 후 raw_list_extra 가 이어지는 구조 확인 필요. **Stage 3 첫 작업은 실제 LIST_HEADER 바이트 레이아웃 검증 (cargo test 로)**.

**신규 테스트**:
- `synthesize_apply_inner_margin_to_list_attr` — apply_inner_margin=true 인 셀 → 어댑터 적용 → 바이트 직렬화 → 파싱 → bit 16 보존
- `cell_padding_correctly_applied_in_hancom_path` — 표 셀 안 여백이 한컴에서 인식되는 비트 위치 검증

**완료 기준**: 위 테스트 그린, 회귀 0.

**일정**: 1일.

### Stage 4 — 문단 break_type + lineseg vpos 사전계산

**목표**: 페이지 폭주의 직접 원인 — lineseg vpos=0 보정.

**hwpx_to_hwp.rs 추가**:
```rust
fn precompute_lineseg_vpos(doc: &mut Document) {
    // 1. reflow_line_segs(doc) 강제 호출
    // 2. paginator 로 vertical_pos 계산
    // 3. 각 paragraph.line_segs[].vertical_pos 갱신
}
fn synthesize_paragraph_break_type(para: &mut Paragraph) {
    if para.raw_break_type != 0 { return; }
    // column_type → break_type 매핑은 serializer 가 이미 하므로 (body_text.rs:194)
    // 추가 작업 없음. column_type 이 정확한지 검증만.
}
```

**신규 테스트**:
- `precompute_vpos_no_zero_after_first_para` — 어댑터 적용 후 모든 line_seg.vertical_pos > 0 (첫 줄 제외)
- `page_count_recovered_hwpx_h_03` — hwpx-h-03 베이스라인 → 어댑터 → page_count = 원본
- `page_count_recovered_hwpx_h_02` — hwpx-h-02 (6 페이지) → 어댑터 → page_count = 6

**완료 기준**: 최소 hwpx-h-02, hwpx-h-03 의 페이지 수 회복. hwpx-h-01 (표 포함) 은 Stage 2/3 와 결합되어 회복.

**일정**: 1.5~2일.

### Stage 5 — 통합 진입점 + 전 영역 결합 검증

**목표**: 어댑터 모든 영역을 단일 호출에 묶고, 3개 디버그 샘플 모두 페이지 수 회복.

**hwpx_to_hwp.rs 통합**:
```rust
pub fn convert_hwpx_to_hwp_ir(doc: &mut Document) -> AdapterReport {
    let mut report = AdapterReport::new();
    if doc.source_format != SourceFormat::Hwpx {
        return report.no_op();
    }
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            for ctrl in &mut para.controls {
                if let Control::Table(t) = ctrl {
                    synthesize_table_raw_ctrl_data(t);
                    synthesize_table_attr(t);
                    for cell in &mut t.cells {
                        synthesize_cell_list_attr_bit16(cell);
                        // 셀 내부 재귀
                        adapt_paragraphs(&mut cell.paragraphs, &mut report);
                    }
                }
            }
            synthesize_paragraph_break_type(para);
        }
    }
    precompute_lineseg_vpos(doc);
    report
}
```

**신규 진입점** (`document_core/commands/document.rs`):
```rust
/// HWPX 출처 IR 을 HWP 호환 IR 로 변환 후 직렬화.
/// HWP 출처는 어댑터 no-op (idempotent).
pub fn export_hwp_with_adapter(&mut self) -> Result<Vec<u8>, HwpError> {
    if self.document.source_format == SourceFormat::Hwpx {
        let _report = converters::hwpx_to_hwp::convert_hwpx_to_hwp_ir(&mut self.document);
    }
    self.export_hwp_native()
}
```

**신규 테스트**:
- `full_adapter_hwpx_h_01` — hwpx-h-01 (9 페이지) → export_hwp_with_adapter → 재로드 → page_count = 9
- `full_adapter_hwpx_h_02` — page_count = 6
- `full_adapter_hwpx_h_03` — page_count 보존
- `idempotent_double_call` — 어댑터 2회 호출 후 결과 동일
- `hwp_source_no_change` — HWP 원본 → 어댑터 호출 → bytes 동일

**완료 기준**: 3개 샘플 모두 페이지 수 회복, 이전 단계 회귀 0.

**일정**: 1일.

### Stage 6 — 명시적 검증 함수 + 사용자 알림 인프라

**목표**: Q3 결정 — 자기 재로드 검증을 별도 명시 호출로. 운영 환경에서는 `export_hwp_with_adapter` 만, 검증 모드에서는 `serialize_hwp_with_verify`.

**신규 진입점**:
```rust
pub struct HwpExportVerification {
    pub bytes: Vec<u8>,
    pub page_count_before: usize,
    pub page_count_after: usize,
    pub diff_areas: Vec<String>,   // 회복 안 된 영역
}
pub fn serialize_hwp_with_verify(&mut self) -> Result<HwpExportVerification, HwpError> {
    let before = self.document.cached_page_count();   // 또는 paginate
    let bytes = self.export_hwp_with_adapter()?;
    let reloaded = DocumentCore::from_bytes(&bytes)?;
    let after = reloaded.page_count();
    Ok(HwpExportVerification { bytes, page_count_before: before, page_count_after: after, diff_areas: vec![] })
}
```

**WASM 노출** (`wasm_api.rs`):
- `export_hwp` 는 source_format 검사로 자동 어댑터 분기 (편의)
- `export_hwp_verify()` 는 검증 결과를 JSON 으로 반환 (UI 가 결정)

**테스트**:
- `verify_returns_matched_page_count_for_clean_hwp` — HWP 원본 라운드트립
- `verify_detects_unrecovered_loss` — 의도적으로 어댑터 일부 비활성화 → 검증이 차이 보고

**완료 기준**: 검증 진입점 그린, 어댑터 자동 호출 경로 그린, 회귀 0.

**일정**: 0.5일.

### Stage 7 — 사용자 경험 복원 + 한컴 수동 검증 + 배포 준비

**목표**: #178 첫 시도의 안전한 부분 (UI 분기) 부활. 작업지시자 한컴 수동 검증.

**기존 코드 수정** (`rhwp-studio`):
- `rhwp-studio/src/command/commands/file.ts:50` — sourceFormat 검사 후 HWPX 출처도 `services.wasm.exportHwp()` 호출, `.hwpx → .hwp` 정규화. (#178 첫 시도와 동일한 변경, 단 이번엔 어댑터로 안전)
- `rhwp-studio/src/hwpctl/index.ts::SaveAs` — 동일 패턴
- 사용자 알림: 상태바 또는 토스트 — "HWPX 문서를 HWP 형식으로 저장했습니다 (rhwp 의 한컴 호환 정책)"

**작업지시자 검증 게이트**:
- WASM 빌드 (`docker compose --env-file .env.docker run --rm wasm`)
- rhwp-studio 빌드 + 로컬 시연
- hwpx-h-01/02/03 각각 열기 → 저장 → 한컴2020 으로 정상 오픈 확인
- (편집 + 저장 시나리오도 별도 검증)

**문서**:
- `mydocs/working/task_m100_178_stage[1..7].md` (각 단계 보고서)
- `mydocs/report/task_m100_178_report.md` (최종 보고서)
- `mydocs/orders/20260419.md` 또는 후속일 갱신 (#178 완료 표기)
- `mydocs/tech/hwp_hwpx_ir_differences.md` 갱신 (어댑터 구현된 영역 명시)
- 이슈 #178 close 코멘트

**완료 기준**: 작업지시자 수동 검증 통과, 보고서·문서 정비, 이슈 close 승인.

**일정**: 1일.

## 3. 파일 변경 요약

| Stage | 신규 파일 | 수정 파일 |
|---|---|---|
| 1 | `src/document_core/converters/mod.rs`, `hwpx_to_hwp.rs`, `diagnostics.rs`, `examples/hwpx_hwp_ir_diff.rs`, `tests/hwpx_to_hwp_adapter.rs` | `src/document_core/mod.rs` (1줄) |
| 2 | `src/document_core/converters/common_obj_attr_writer.rs` | `hwpx_to_hwp.rs` |
| 3 | — | `hwpx_to_hwp.rs` |
| 4 | — | `hwpx_to_hwp.rs` |
| 5 | — | `hwpx_to_hwp.rs`, `src/document_core/commands/document.rs` (+1 메서드) |
| 6 | — | `src/document_core/commands/document.rs` (+1 메서드), `src/wasm_api.rs` (+1 export) |
| 7 | — | `rhwp-studio/src/command/commands/file.ts`, `rhwp-studio/src/hwpctl/index.ts`, 문서 다수 |

`src/serializer/cfb_writer.rs`, `body_text.rs`, `control.rs` — **0 줄 수정**.

## 4. 위험 요소 추가

| 위험 | 단계 | 완화 |
|---|---|---|
| `cell.raw_list_extra` 비트 OR 가 LIST_HEADER 레이아웃과 충돌 | Stage 3 | Stage 3 첫 작업으로 바이트 레이아웃 단위 테스트 우선 작성 |
| `serialize_common_obj_attr` 작성 누락 필드 | Stage 2 | parser 의 read 코드와 1:1 대응 + 라운드트립 테스트로 검증 |
| `precompute_lineseg_vpos` 가 paginator 의존 — 순환 의존 가능성 | Stage 4 | Stage 4 시작 시 의존성 그래프 확인. 어댑터 → paginator 단방향이어야 함 |
| `convert_hwpx_to_hwp_ir` idempotent 보장 실패 | Stage 5 | 모든 synthesize 함수에 `if !empty { return; }` 가드 + idempotent 테스트 |
| `serialize_hwp_with_verify` 의 paginate 가 무거움 | Stage 6 | 명시 호출만 권장 (Q3 결정), 자동 경로 (`export_hwp`) 는 verify 생략 |

## 5. 일정 합계

- Stage 1: 0.5일
- Stage 2: 1.5~2일
- Stage 3: 1일
- Stage 4: 1.5~2일
- Stage 5: 1일
- Stage 6: 0.5일
- Stage 7: 1일
- **총: 7~8일**

## 6. 본 타스크 정체성 재확인 (셀프 체크리스트)

작업지시자 통찰 — "잘 작동하는 HWP 직렬화의 어깨 위에 서자" — 의 반영 점검:

- [x] HWP 직렬화기 코드 0줄 수정 정책 명문화 (§1.0.1, §1.2 표, §3 표, §4 위험 5번)
- [x] 어댑터는 IR 만 만진다 — `&mut Document` 진입점 단일화 (§1.1, §2 Stage 5)
- [x] 매핑 명세는 "HWP 직렬화기가 무엇을 읽는가" 기준 (§1.3 표 — 6개 위치 명시, 코드 라인 인용)
- [x] HWPX 가 채우지 않는 영역 중 직렬화기가 동적 경로로 처리 가능한 것은 어댑터에서 제외 (§1.3 `raw_stream` 발견 → §1.0.2 의미 설명)
- [x] HWP 출처에는 어댑터 no-op (§2 Stage 5 `source_format` 검사, idempotent 테스트)
- [x] 신규 직렬화 경로·신규 바이트 작성기 0건 (§3 표 — `serializer/` 하위 신규 파일 0건)

본 셀프 체크리스트 6항 모두 충족. 본 구현계획서는 "HWP 직렬화기 어깨 위 어댑터" 원칙을 충실히 반영함을 확인.

## 7. 승인 요청

본 구현계획서 승인 후 Stage 1 착수. 단계별 보고서 + 승인 게이트 절차 준수.

확인 필요:

1. **`src/document_core/converters/` 디렉터리 신설** 합의 (다른 후보: `src/serializer/adapters/`, `src/converters/`)
   - 본 타스크 정체성상 `serializer/` 하위는 부적절 — 직렬화기 자체를 건드리지 않으므로
   - `document_core/converters/` 가 "IR 변환 어댑터" 의미에 부합
2. **Stage 7 의 UI 알림 문구** — "HWPX 문서를 HWP 형식으로 저장했습니다" 가 적절한지, 더 자세한 안내 필요한지
3. **`export_hwp_native` 비변경 정책** 유지 — 호출자(WASM, CLI) 가 어댑터 호출 책임. 또는 자동 분기로 갈지

승인 요청 드립니다.
