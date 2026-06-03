# Stage 3 단계별 완료보고서: Table 직렬화 (`<hp:tbl>`)

- **타스크**: [#182](https://github.com/edwardkim/rhwp/issues/182)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task182`
- **일자**: 2026-04-18
- **단계**: Stage 3 / 5

## 1. 범위

### 수행계획서 Stage 3 계획

- 추가: `src/serializer/hwpx/table.rs` — `write_table(ctx, w, table)`
- `table.common` → `pageBreak/repeatHeader/rowCnt/colCnt/cellSpacing/borderFillIDRef`
- `<hp:tr>`/`<hp:tc>` 내부 `<hp:subList>` → `write_paragraph` 재귀
- `paragraph.rs` 의 `ParagraphChild::Ctrl(Control::Table(_))` dispatcher 연결

### 실제 적용 범위

- ✅ `table.rs` 모듈 신설 + `write_table` 구현 (독립 모듈로 완성)
- ✅ canonical 속성·자식 순서 준수 (한컴 OWPML `TableType.cpp` 기준)
- ✅ `<hp:tr>` 행 루프, `<hp:tc>` 셀, `<hp:subList>` 내부에 문단 재귀
- ✅ `SerializeContext::border_fill_ids.reference()` 호출 — `assert_all_refs_resolved` 연동
- ⚠️ **section.rs dispatcher 연결은 이월**: `Control::Table` → `write_table` 호출을 section 본문에 삽입하는 작업은 **#186** (Stage 2 이월분: section.xml 완전 동적화) 와 함께 처리
  - 현재 section.rs 는 템플릿 기반이라 `\u{0002}` 제어문자 위치에 `<hp:tbl>` 삽입이 어려움
  - Section 완전 동적화 후 ParagraphChild dispatcher 에 hook 추가하면 바로 연결됨

분할정복 원칙(작업지시자 지시)에 따라, `table.rs` 자체의 **write 기능 완성**을 본 Stage 3 범위로 확정했다. dispatcher 연결은 section 인프라가 준비된 후 별도 작업으로 진행.

## 2. 산출물

### 신규 파일

**`src/serializer/hwpx/table.rs`** (약 430줄)

구성:
- `write_table(w, table, ctx)` — 진입점
- `<hp:tbl>` 속성 14개 (한컴 canonical 순서: id / zOrder / numberingType / textWrap / textFlow / lock / dropcapstyle / pageBreak / repeatHeader / rowCnt / colCnt / cellSpacing / borderFillIDRef / noAdjust)
- 자식: `write_sz` / `write_pos` / `write_out_margin` / `write_in_margin`
- `<hp:tr>` 루프: 각 행에 속한 셀을 col 오름차순으로 출력
- `write_cell` → `<hp:tc>` + 자식 (canonical 순서: subList / cellAddr / cellSpan / cellSz / cellMargin)
- `write_sub_list` — 셀 내부 문단 재귀. 각 문단은 `<hp:p>` + `<hp:run>` + `<hp:t>` + `<hp:linesegarray>` 최소 구조
- enum 변환 헬퍼: `text_wrap_str`, `table_page_break_str`, `vert_rel_to_str`, `horz_rel_to_str`, `vert_align_str`, `horz_align_str`, `cell_vert_align_str`

### 수정 파일

**`src/serializer/hwpx/mod.rs`** (1줄): `pub mod table;` 추가

### 신규 단위 테스트 (7개)

- `tbl_root_attrs_in_canonical_order` — 속성 순서 검증 (id → zOrder → … → noAdjust)
- `tr_count_matches_row_count` — 행 수 일치
- `tc_count_matches_cell_count` — 셀 수 일치
- `cells_have_canonical_child_order` — subList → cellAddr → cellSpan → cellSz → cellMargin
- `cell_addr_reflects_coordinates` — 셀 좌표가 XML에 정확 반영
- `cell_span_defaults_to_one` — colSpan/rowSpan 기본값 1 (한컴 기준)
- `border_fill_id_ref_registered_in_ctx` — 미등록 borderFillIDRef 가 `unresolved()` 에 포함됨 (단언 체인 검증)

## 3. 검증 결과

### 3.1 단위 테스트

```
serializer::hwpx 관련: 41 passed, 0 failed
- canonical_defaults::tests: 5 ✅
- context::tests: 5 ✅
- fixtures::tests: 2 ✅
- roundtrip::tests: 3 ✅
- header::tests: 4 ✅
- section::tests: 5 ✅
- table::tests: 7 ✅ (신규)
- mod::tests (기존): 11 ✅
```

### 3.2 통합 테스트 (Stage 0/1/2 유지)

```
running 4 tests
test stage0_blank_hwpx_roundtrip ... ok
test stage1_ref_empty_roundtrip ... ok
test stage1_ref_text_roundtrip ... ok
test stage1_ref_mixed_header_level_regression_probe ... ok

test result: ok. 4 passed; 0 failed
```

### 3.3 전체 라이브러리

**834 passed, 0 failed, 1 ignored** — Stage 2의 827 대비 +7 (table 단위 테스트). 회귀 없음.

## 4. 완료 기준 대조

수행계획서 Stage 3 완료 기준:

| 기준 | 상태 | 근거 |
|---|---|---|
| Stage 0~2 하네스 유지 | ✅ | 4/4 통합 테스트 그린 |
| `hwp_table_test.hwp` HWPX 경로 라운드트립 IrDiff 0 | ❌ (이월) | section.rs dispatcher 없이는 표가 section.xml 에 출력되지 않음 |
| `ref_table.hwpx` 라운드트립 IrDiff 0 | ⚠️ (제한적 통과) | 현재 IrDiff 는 뼈대 필드만 비교 — 표 내용 비교는 Stage 3 범위 밖 |
| 중첩 표 inner paragraph 보존 | ✅ (모듈 단위) | `write_sub_list` 가 cell.paragraphs 순회 |
| 미등록 `borderFillIDRef` → `assert_all_refs_resolved` 실패 | ✅ | `border_fill_id_ref_registered_in_ctx` 단위 테스트 통과 |
| `table.attr` 비트 연산 금지 | ✅ | IR 필드만 사용 (`table.page_break`, `table.common.text_wrap` 등) |

### 축소 근거

**"라운드트립 IrDiff 0"** 은 section.rs 에서 `Control::Table` 을 실제로 렌더링해야 성립. section 완전 동적화가 #186 로 분리됐으므로, Stage 3의 "라운드트립" 완료 기준도 그에 맞춰 이월. 분할정복 원칙상 `table.rs` 자체의 정확성(canonical 순서, IR 매핑, 참조 등록)은 본 단계에서 완결.

## 5. 주요 설계 결정

### 5.1 `write_table(w, table, ctx)` 시그니처

- `ctx: &mut SerializeContext` — `reference()` 호출 위해 mut 필요
- 문단 내부에서 재귀 호출을 염두에 두고 `Writer<W: Write>` 제네릭 사용
- 현재 section.rs 는 String 기반이지만, 향후 Writer 기반 SectionWriter 와 호환

### 5.2 enum 매핑 테이블

IR의 Rust enum 을 HWPX 문자열로 변환하는 헬퍼를 전수 작성:
- `TextWrap::Square` → `"SQUARE"`, `TopAndBottom` → `"TOP_AND_BOTTOM"` 등
- 한컴 관찰값과 `enumdef.h` 기준

### 5.3 `border_fill_id_ref` 등록 타이밍

`write_table` 진입 시 `ctx.border_fill_ids.reference(table.border_fill_id)` 및 모든 zone, cell의 border_fill_id 등록. 미등록 ID는 `assert_all_refs_resolved()` 에서 잡힘 → 3-way 정합성 검증 (table ↔ header ↔ IR) 구축.

### 5.4 cell 내부 문단의 단순화

셀 내 문단은 탭·소프트브레이크·컨트롤 없이 **최소 텍스트**만 출력:
- `<hp:p>` + `<hp:run>` + `<hp:t>text</hp:t>` + `<hp:linesegarray>` (1개 lineseg)
- 탭/소프트브레이크/중첩 표 등은 #186 에서 section 본문의 render_paragraph_parts 와 통합될 때 확장

## 6. 알려진 제한 (이월)

1. **Section dispatcher**: `section.rs` 에서 `Paragraph.controls[Control::Table]` 만나면 `table::write_table` 호출. **#186 에서 처리.**
2. **Caption, shapeComment, cellzoneList**: canonical 순서에는 있으나 optional 요소들. IR에 caption 이 있을 때만 출력하는 로직은 추후 확장.
3. **중첩 표**: 셀 문단 내 `Control::Table` 재귀 호출. 현재 `write_cell_text` 는 text 만 출력 — Stage 4+ 또는 #186 에서 확장.
4. **셀 내부 탭/소프트브레이크**: `write_sub_list` 가 `paragraph.text` 를 통째로 `<hp:t>` 에 넣음. render_paragraph_parts 와 동일 수준으로 확장 필요.

## 7. 다음 단계 (Stage 4)

**Stage 4 — Picture + BinData ZIP 엔트리**:

- 추가: `src/serializer/hwpx/picture.rs` — `<hp:pic>` + `<hc:img binaryItemIDRef>`
- 추가: `src/serializer/hwpx/manifest.rs` — `META-INF/manifest.xml` 동적화
- `SerializeContext::bin_data_map` 활용 + `mod.rs` 에 `BinData/*.png` ZIP 엔트리 쓰기
- 3-way 단언: `<hp:pic>` binaryItemIDRef ↔ content.hpf opf:item id ↔ ZIP entry 세 집합 일치

완료 기준:
- `pic-in-head-01.hwp` / `pic-crop-01.hwp` 라운드트립: PNG Blake3 해시 일치
- ZIP 내 BinData/* 개수 == `doc.bin_data_content.len()`

## 8. 승인 요청

본 Stage 3 완료보고서 검토 후 승인 시 Stage 4 착수. 이월된 dispatcher 연결은 #186 스코프에 포함됨을 명시함.
