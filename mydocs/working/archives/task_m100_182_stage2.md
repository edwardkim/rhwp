# Stage 2 단계별 완료보고서: section.xml IR 기반 속성 + charPrIDRef 매핑

- **타스크**: [#182](https://github.com/edwardkim/rhwp/issues/182)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task182`
- **일자**: 2026-04-17
- **단계**: Stage 2 / 5

## 1. 범위 — 수행계획서 대비 축소 적용

### 수행계획서의 Stage 2 계획

- `section.rs` 템플릿 치환 제거, `SectionWriter` 루트부터 빌드
- 추가: `paragraph.rs` (`write_paragraph` + `ParagraphChild` enum)
- `secPr/pagePr/grid/startNum/visibility/footNotePr/endNotePr/pageBorderFill` 를 `Section.section_def`/`PageDef` 에서 매핑

### 실제 적용한 범위 (현실적 진화)

1단계 목표가 **"기존 HWPX 수정 후 저장 시 온전히 열림"** 인 점을 고려해, **템플릿 보존 + IR 기반 속성만 교체** 하는 최소 침습 진화 방식을 택했다:

- ✅ `<hp:p>` 속성(`paraPrIDRef`, `styleIDRef`, `pageBreak`, `columnBreak`)을 IR에서 가져옴
- ✅ `<hp:run charPrIDRef>` 을 `Paragraph.char_shapes[0].char_shape_id` 에서 가져옴
- ✅ 추가 문단도 IR 기반 속성
- ⚠️ **`secPr/pagePr/grid/startNum/visibility/footNotePr/endNotePr/pageBorderFill` 완전 동적화는 유지**
  - 템플릿에 박힌 고정값 유지. 이유: 이들 속성의 IR 필드가 부분적으로만 존재하며, 완전 동적화 시 기존 레퍼런스 샘플과 어긋날 위험
  - 기존 HWPX 로드·편집·저장 시나리오에서는 원본 HWPX의 secPr 가 그대로 유지되는 방식이 아니라 **템플릿이 한컴 빈 문서의 secPr를 유지**하는 방식이므로, `SectionDef`/`PageDef` 값이 현재 출력에 반영되지 않음
  - 이는 1단계의 **"빈 문서 새로 만들기는 범위 외"** 원칙과 호환 — 실제 "편집 후 저장" 시나리오는 원본 `raw_stream` 보존 경로로 처리됨
- ⚠️ **다중 run 분할** (`char_shapes` 범위별 run 나누기)은 **이번 Stage에 미포함**
  - 현재 구현은 문단 내 첫 run 의 `charPrIDRef` 만 IR 기반
  - 범위별 분할은 `Paragraph.text` 안에서 UTF-16 offset 기준 여러 run 으로 나누어야 하는데, 기존 `render_paragraph_parts` 로직이 문단 전체 텍스트를 하나의 `<hp:t>` 로 만드는 구조라 리팩터링이 큼
  - 다중 run 분할은 **Stage 3 Table 구현 후** 또는 별도 task 로 분리 고려 제안

### 축소의 정당성

수행계획서의 완전 동적화는 "완전히 새 빈 문서 출력 품질"까지 고려한 목표인데, 작업지시자가 1단계 범위를 **"기존 HWPX 수정 후 저장"** 으로 한정했다. 본 축소는 그 범위에 부합한다.

다만 **완전 동적화는 2단계(후속 범위)에 이월 필요** — Stage 2 완료 기준의 일부가 본 단계에서 해결되지 않았음을 명시한다.

## 2. 산출물

### 수정 파일

**`src/serializer/hwpx/section.rs`** (133줄 → 228줄)

- `write_section(section, doc, index, ctx)` — `ctx` 파라미터 추가
- `render_hp_p_open(p, id)` 신규 — `Paragraph` → `<hp:p>` 시작 태그 문자열
- `first_run_char_shape_id(p)` 신규 — 첫 run 의 charPrIDRef 산출
- 템플릿 앵커 치환 방식으로 최소 침습 변경

**`src/serializer/hwpx/mod.rs`**: `section::write_section(sec, doc, i)` → `section::write_section(sec, doc, i, &ctx)`

### 신규 단위 테스트 (5개)

`section.rs` 내부:
- `hp_p_attrs_reflect_para_shape_id_and_style_id`
- `hp_run_reflects_first_char_shape_id`
- `page_break_paragraph_emits_attr`
- `default_paragraph_keeps_zero_attrs`
- `additional_paragraphs_use_their_own_char_shape`

## 3. 검증 결과

### 3.1 단위 테스트

```
serializer::hwpx 관련: 34 passed, 0 failed
- canonical_defaults::tests: 5 ✅
- context::tests: 5 ✅
- fixtures::tests: 2 ✅
- roundtrip::tests: 3 ✅
- header::tests: 4 ✅
- section::tests: 5 ✅ (신규)
- mod::tests (기존): 11 ✅
```

### 3.2 통합 테스트 (Stage 0/1 전부 유지)

```
running 4 tests
test stage0_blank_hwpx_roundtrip ... ok
test stage1_ref_empty_roundtrip ... ok
test stage1_ref_text_roundtrip ... ok
test stage1_ref_mixed_header_level_regression_probe ... ok

test result: ok. 4 passed; 0 failed
```

### 3.3 전체 라이브러리

**827 passed, 0 failed, 1 ignored** — 회귀 없음.

## 4. 완료 기준 대조

수행계획서 Stage 2 완료 기준:

| 기준 | 상태 | 근거 |
|---|---|---|
| Stage 0, 1 하네스 유지 | ✅ | 4/4 통합 테스트 그린 |
| 기존 문단/탭/줄바꿈 테스트 유지 | ✅ | `serializer::hwpx::tests` 11개 전부 통과 |
| `ref_text.hwpx` IR 라운드트립 | ✅ | `stage1_ref_text_roundtrip` (Stage 1 테스트가 Stage 2 에서도 통과) |
| 다중 run 문서: run 개수·charPrIDRef 일치 | ⚠️ | 첫 run 의 charPrIDRef 만 IR 기반. 다중 run 분할은 이월 |
| `secPr` 완전 동적화 | ❌ | 이월 (템플릿 유지) |

## 5. 주요 설계 결정

### 5.1 템플릿 보존 + 앵커 치환

기존 코드가 이미 잘 동작하는 부분(텍스트 escape, linesegarray, 탭, 소프트 브레이크)은 그대로 유지. 속성 교체가 필요한 지점만 **정확한 앵커 문자열로 `replacen` 치환**.

앵커 예:
- `<hp:p id="3121190098" paraPrIDRef="0" ...>` (템플릿 리터럴과 완전 일치)
- `<hp:run charPrIDRef="0"><hp:t>...<hp:t>` (TEXT_SLOT 치환 이후 문맥)

### 5.2 `first_run_char_shape_id` fallback

IR의 `Paragraph.char_shapes` 가 비어있을 수 있다 (예: `Document::default()`). 이 경우 0 을 반환해 템플릿 원래 값과 동일 → 회귀 방지.

### 5.3 ColumnBreakType → pageBreak/columnBreak 매핑

- `ColumnBreakType::Page` → `pageBreak="1"`
- `ColumnBreakType::Column` → `columnBreak="1"`
- `ColumnBreakType::Section`/`MultiColumn` → 둘 다 0 (Stage 3+ 에서 별도 처리)

## 6. 알려진 제한 (Stage 3+ 이월)

1. **`secPr` 완전 동적화**: `Section.section_def`/`PageDef` 필드를 `<hp:pagePr width/height/landscape/margin>`, `<hp:grid>`, `<hp:visibility>` 등으로 매핑하는 `SectionWriter` 본격 구현 필요. 1단계 "기존 문서 편집 후 저장" 시나리오에선 우선순위 낮으나, 완전한 독립 저장을 위해선 필요.
2. **다중 run 분할**: `char_shapes[].start_pos` 기반 UTF-16 범위로 문단을 나눠 여러 `<hp:run charPrIDRef>` 로 출력. 현재는 전체 문단이 하나의 run.
3. **`<hp:ctrl>` 위치의 ColumnDef**: 템플릿의 `<hp:ctrl><hp:colPr .../></hp:ctrl>` 가 고정. IR의 `Paragraph.controls` 에 ColumnDef 가 있어도 반영되지 않음.
4. **lineSegArray 정확도**: 현재 vert_cursor 계산은 단순(`VERT_STEP = 1600`). `Paragraph.line_segs` 의 실제 vpos 반영은 #177 에서 처리.

## 7. 다음 단계 (Stage 3)

**Stage 3 — Table 직렬화 (`<hp:tbl>`)**:

- 추가: `src/serializer/hwpx/table.rs` — `write_table(ctx, w, table)`
- `table.common` (CommonObjAttr) → `pageBreak/repeatHeader/rowCnt/colCnt/cellSpacing/borderFillIDRef`
- ⚠️ `table.attr` 비트 연산 **금지** (HWPX에서 0)
- `<hp:tr>`/`<hp:tc>` 내부 `<hp:subList>` → `write_paragraph` 재귀
- `paragraph.rs` 의 `ParagraphChild::Ctrl(Control::Table(_))` dispatcher 연결

완료 기준:
- `hwp_table_test.hwp` HWPX 경로 라운드트립 IrDiff 0
- `ref_table.hwpx` 라운드트립 IrDiff 0
- 중첩 표 inner paragraph 보존
- 미등록 `borderFillIDRef` → `assert_all_refs_resolved` 실패

### 선택사항 제안

Stage 3 진입 전 **Stage 2 잔여 작업**(secPr 완전 동적화 + 다중 run 분할)을 **별도 이슈**로 분리하거나, Stage 3 완료 후 순차 진행 가능. 작업지시자 판단에 맡깁니다.

## 8. 승인 요청

본 Stage 2 단계별 완료보고서 검토 후 다음 중 선택 부탁드립니다:

1. **Stage 2 승인 + Stage 3 착수**
2. **Stage 2 잔여 작업(secPr/다중 run)을 Stage 2.1 로 추가 후 Stage 3 진입**
3. **Stage 2 잔여 작업을 별도 이슈로 등록 후 Stage 3 진입**
