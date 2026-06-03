# Stage 1 단계별 완료보고서: 감지 인프라 (ValidationReport)

- **타스크**: [#177](https://github.com/edwardkim/rhwp/issues/177)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task177`
- **일자**: 2026-04-18
- **단계**: Stage 1 / 4

## 1. 수행 범위

구현계획서의 Stage 1에 해당 — `ValidationReport` 구조 신설, `DocumentCore::validation_report` 필드 추가, `from_bytes` 에 비표준 lineseg 감지 로직 삽입.

## 2. 산출물

### 2.1 신규 파일

**`src/document_core/validation.rs`** (170줄)

구성:
- `CellPath` — 표 셀 내부 문단 경로 구조체 (table_ctrl_idx, row, col, inner_para_idx)
- `WarningKind` enum — 경고 종류 (`LinesegArrayEmpty`, `LinesegUncomputed`)
- `WarningKind::Display` — 한국어 메시지 구현
- `ValidationWarning` — 경고 한 건 (section/paragraph 경로 + kind)
- `ValidationReport` — 경고 컬렉션 + `summary()` 집계

### 2.2 수정 파일

**`src/document_core/mod.rs`**:
- `pub mod validation;` 모듈 선언
- `DocumentCore` 구조체에 `validation_report: ValidationReport` 필드 추가
- `DocumentCore::new_empty` 에 `ValidationReport::new()` 초기화 추가
- `DocumentCore::validation_report() -> &ValidationReport` 공개 접근자 추가

**`src/document_core/commands/document.rs`**:
- `from_bytes` 에서 `reflow_zero_height_paragraphs` **호출 이전** 에 `validate_linesegs(&document)` 추가 (reflow 전 원시 IR 검증)
- struct 생성 시 `validation_report` 필드 채움
- `validate_linesegs(document: &Document) -> ValidationReport` 신규 — 2가지 규칙 적용
- `check_paragraph_linesegs` 헬퍼 — 단일 문단 검사
- 표 셀 내부 문단 재귀 검사 포함

### 2.3 감지 규칙

| 규칙 | 조건 | 경고 종류 |
|---|---|---|
| R1 | `!text.is_empty() && line_segs.is_empty()` | `LinesegArrayEmpty` |
| R2 | `line_segs.len() == 1 && line_segs[0].line_height == 0` | `LinesegUncomputed` |

R2 는 기존 `needs_line_seg_reflow` 조건과 동일 — 기존 자동 reflow 로직이 처리하는 케이스도 **경고로 항상 기록**되어 사용자에게 고지 가능.

## 3. 검증 결과

### 3.1 단위 테스트 (10개 추가)

`validation.rs` 내부 (4개):
- `report_default_is_empty` ✅
- `summary_groups_by_kind` ✅
- `warning_display_messages` ✅
- `cell_path_equality` ✅

`commands/document.rs` 내부 `validate_linesegs_tests` (6개):
- `validate_detects_empty_linesegs` ✅
- `validate_detects_uncomputed_lineseg` ✅
- `validate_skips_healthy_lineseg` ✅
- `validate_skips_empty_paragraph` ✅
- `validate_recurses_into_table_cells` ✅
- `validate_records_multiple_warnings` ✅

### 3.2 전체 라이브러리

**860 passed, 0 failed, 1 ignored** — Stage 0 (#182 완료 시점) 의 850 대비 **+10** (본 단계 신규). 회귀 0건.

### 3.3 통합 테스트

HWPX 라운드트립 하네스 8/8 유지:
- `stage0_blank_hwpx_roundtrip`, `stage1_ref_empty/text/mixed`, `stage5_ref_table/form-002/2025 1Q/2Q` 모두 그린.

## 4. 완료 기준 대조

구현계획서 Stage 1 완료 기준:

| 기준 | 상태 | 근거 |
|---|---|---|
| `src/document_core/validation.rs` 신규 생성 | ✅ | 170줄, 4개 단위 테스트 포함 |
| `DocumentCore::validation_report` 필드 추가, `from_bytes` 에서 채움 | ✅ | mod.rs + commands/document.rs |
| `validate_linesegs` 메서드 추가, 2가지 규칙 검출 | ✅ | R1(empty), R2(uncomputed) |
| 단위 테스트 6개 통과 | ✅ | 실제 10개 (validation.rs 4개 + document.rs 6개) |
| 기존 850개 테스트 유지 | ✅ | 860/0/1 |

## 5. 주요 설계 결정

### 5.1 Validation 호출 시점 — `reflow_zero_height_paragraphs` 이전

기존 `from_bytes` 는 다음 순서:
```
parse_document → resolve_styles → reflow_zero_height_paragraphs → compose_section
```

`reflow_zero_height_paragraphs` 는 IR을 수정해 `line_height` 를 채운다. 따라서 **이후 시점에 validate 하면 원본 상태를 잃음**. 구현계획서 대로 reflow 이전에 `validate_linesegs` 호출하여 원시 IR 검증.

### 5.2 R2의 경고 의미

기존 `needs_line_seg_reflow` 조건이 트리거하면 `reflow_zero_height_paragraphs` 가 IR을 수정한다. 그러나 rhwp 는:

- **자동 보정 행위를 여전히 수행** (기존 동작 보존 — 작업지시자 결정)
- **경고로 기록하여 사용자에게 고지** (비표준 감지 흔적)

즉 "조용히 고치는" 것이 아니라 "고쳤는데 원래 비표준이었음을 기록" 하는 방식. Discussion #188 의 원칙과 일치.

### 5.3 IR 순수성 유지

경고를 `Paragraph` 필드가 아닌 `DocumentCore::validation_report` 에 저장. 이점:
- Paragraph 의 `PartialEq`, `Clone` 에 영향 없음
- IR 스냅샷·직렬화 경로가 단순
- 검증은 "로드 시점의 관찰" 로 개념 분리

### 5.4 셀 내부 문단 재귀 검증

표 셀은 독립적 문단 계층을 가지므로 본문 문단만 검사하면 놓친다. `validate_linesegs` 가 `controls[Control::Table]` 을 순회하며 셀 내부 문단도 검사하고 `CellPath` 로 경로 기록.

## 6. 알려진 제한

- **R3/R4 규칙** (lineseg 개수 불일치, textpos 어긋남) 은 구현계획서에서 **Stage 4 로 이월** (실문서 false positive 조정 후 규칙 확정)
- **경고 레벨** (error/warn/info) 분리는 현재 없음 — 필요 시 후속 이슈
- **사용자 지역화** — 한국어 메시지만. 다국어는 후속

## 7. 다음 단계 (Stage 2)

**Serializer 원본 보존**:
- `src/serializer/hwpx/section.rs` 의 `push_lineseg` 제거
- `render_lineseg_array(para)` 로 교체 — IR의 6개 필드 전부 그대로 출력
- 기존 `linesegs_emitted_per_linebreak` 테스트 재작성
- 신규 통합 테스트 `task177_lineseg_preserved_on_roundtrip` 추가

## 8. 승인 요청

본 Stage 1 완료보고서 검토 후 승인 시 Stage 2 착수.
