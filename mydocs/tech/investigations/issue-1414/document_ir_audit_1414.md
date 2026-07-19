---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1414/README.md
last_verified: 2026-07-16
---

# Document IR 구조 감사 — Task #1414 Stage 1 필드 인벤토리

- 이슈: https://github.com/edwardkim/rhwp/issues/1414
- 작성일: 2026-06-15
- 범위: `src/model` 선언 기준 1차 인벤토리
- 상태: Stage 1. 생산자/소비자 매트릭스는 Stage 2에서 별도 작성

## 1. 감사 관점

rhwp의 `Document` IR은 HWP5/HWPX/HWP3 parser frontend가 공통으로 lowering하는
중간 표현이다. 이후 document core, renderer, serializer backend가 이 IR을 소비한다.

이번 Stage 1에서는 필드 제거 여부를 결정하지 않는다. 우선 현재 모델 필드를 다음
성격으로 분류한다.

| 태그 | 의미 |
|---|---|
| `semantic` | 문서의 의미/구조를 직접 표현하는 필드 |
| `layout` | 조판 또는 렌더링에 필요한 배치 정보 |
| `reference` | DocInfo, BinData, style, borderFill 등 다른 테이블을 참조하는 ID |
| `raw-preserve` | 원본 HWP/HWPX 바이트/XML을 라운드트립 보존하기 위한 필드 |
| `format-bridge` | 특정 입력 포맷의 값을 공통 IR/HWP5 저장 계약으로 연결하는 필드 |
| `derived-cache` | 원본 필드에서 계산된 캐시 또는 빠른 접근용 구조 |
| `edit-state` | 편집/증분 레이아웃을 위한 dirty/cache 상태 |
| `backend-contract` | 특정 serializer/renderer가 현재 source-of-truth로 삼는 필드 |

## 2. 모델 파일 구성

| 파일 | 주요 구조체 | 1차 책임 |
|---|---|---|
| `src/model/document.rs` | `Document`, `DocInfo`, `Section`, `SectionDef` | 문서 최상위 구조, DocInfo ID 테이블, 구역 |
| `src/model/paragraph.rs` | `Paragraph`, `LineSeg`, `CharShapeRef`, `RangeTag` | 문단 텍스트, 조판 줄, 제어문자/컨트롤 연결 |
| `src/model/control.rs` | `Control`, `Equation`, `Field`, `FormObject` | 문단 내 확장 컨트롤 |
| `src/model/shape.rs` | `CommonObjAttr`, `ShapeComponentAttr`, `ShapeObject`, `Caption`, `ChartShape`, `OleShape` | 그리기 개체 공통 속성 및 도형/OLE/차트 |
| `src/model/image.rs` | `Picture`, `ImageAttr`, `PictureEffects` | 그림 컨트롤과 이미지 속성 |
| `src/model/table.rs` | `Table`, `Cell`, `TableZone` | 표/셀 구조와 셀 문단 리스트 |
| `src/model/style.rs` | `Font`, `CharShape`, `ParaShape`, `Numbering`, `BorderFill`, `TabDef`, `Style` | DocInfo 스타일/서식 테이블 |
| `src/model/page.rs` | `PageDef`, `PageBorderFill`, `ColumnDef`, `PageAreas` | 용지, 쪽 테두리, 다단, 계산된 페이지 영역 |
| `src/model/footnote.rs` | `Footnote`, `Endnote`, `FootnoteShape` | 각주/미주 본문과 모양 |
| `src/model/header_footer.rs` | `Header`, `Footer`, `MasterPage` | 머리말/꼬리말/바탕쪽 |
| `src/model/bin_data.rs` | `BinData`, `BinDataContent` | 이미지/OLE 등 바이너리 참조와 실제 데이터 |
| `src/model/path.rs` | `PathSegment` | 벡터 경로 세그먼트 |
| `src/model/mod.rs` | `HwpUnit`, `Point`, `Rect`, `Padding` | 공통 단위와 기하 타입 |

## 3. 최상위 문서 IR

### 3.1 `Document`

| 필드 | 태그 | 메모 |
|---|---|---|
| `header` | semantic, raw-preserve | 파일 버전/압축/암호화/배포용 상태. `FileHeader.raw_data` 포함 |
| `doc_properties` | semantic, raw-preserve | 구역 수, 번호 시작값, caret 위치. `raw_data` 포함 |
| `doc_info` | semantic, reference, raw-preserve | 글꼴/서식/테두리/번호/BinData ID 테이블 |
| `sections` | semantic | 본문 구역과 문단 리스트 |
| `preview` | raw-preserve, semantic | 미리보기 이미지/텍스트 |
| `bin_data_content` | semantic, backend-contract | BinData 스토리지에서 로드된 실제 바이너리 |
| `extra_streams` | raw-preserve | 모델링하지 않는 HWP CFB 스트림 보존 |
| `hwpx_aux_entries` | raw-preserve | 모델링하지 않는 HWPX ZIP 엔트리 보존 |
| `is_hwp3_variant` | format-bridge, layout | HWP3→HWP5 변환본 휴리스틱. renderer/typeset에서 소비됨 |

1차 관찰:

- `Document`는 semantic IR과 raw package preservation을 동시에 가진다.
- `is_hwp3_variant`는 전역 포맷 출처/변환본 힌트가 공통 IR 최상위에 들어온 대표 필드다.
  제거 후보가 아니라 Stage 2에서 생산자와 소비자 경계를 정밀 확인해야 하는 항목이다.

### 3.2 `DocInfo`

| 필드 그룹 | 태그 | 메모 |
|---|---|---|
| `bin_data_list`, `font_faces`, `border_fills`, `char_shapes`, `tab_defs`, `numberings`, `bullets`, `para_shapes`, `styles` | semantic, reference | 문서 전역 ID 테이블 |
| `extra_records` | raw-preserve | 모델링하지 않는 DocInfo 레코드 |
| `raw_stream` | raw-preserve, backend-contract | DocInfo 원본 스트림. serializer가 재사용 가능 |
| `bullet_count`, `memo_shape_count` | format-bridge, raw-preserve | ID_MAPPINGS count와 실제 모델링 범위 사이 연결 |
| `distribute_doc_data_removed`, `raw_stream_dirty` | edit-state, backend-contract | 배포용 해제/편집 후 serializer 경로 제어 |
| `hwpx_head_tail`, `hwpml_version` | raw-preserve, format-bridge | HWPX header 재방출용 보존값 |

1차 관찰:

- `DocInfo`는 ID 테이블 semantic 모델과 원본 stream 보존 정책이 강하게 결합되어 있다.
- `raw_stream_dirty`는 document core 편집 상태와 serializer 정책을 연결하는 필드다.

### 3.3 `Section` / `SectionDef`

| 구조체 | 필드 그룹 | 태그 | 메모 |
|---|---|---|---|
| `Section` | `section_def`, `paragraphs` | semantic | 구역 정의와 본문 문단 |
| `Section` | `raw_stream` | raw-preserve, backend-contract | BodyText/SectionN 원본 스트림 |
| `SectionDef` | `flags`, `page_num`, `page_def`, `footnote_shape`, `endnote_shape`, `page_border_fill` | semantic, layout | 구역/쪽 조판 설정 |
| `SectionDef` | `hide_*`, `text_direction`, `outline_numbering_id` | semantic, layout | 표시/번호/방향 설정 |
| `SectionDef` | `raw_ctrl_extra`, `extra_child_records` | raw-preserve | 미모델링 tail/자식 레코드 |
| `SectionDef` | `extra_page_border_fills`, `master_pages` | semantic, layout, raw-preserve | 추가 쪽 테두리와 바탕쪽 |

## 4. 문단 IR

### 4.1 `Paragraph`

| 필드 | 태그 | 메모 |
|---|---|---|
| `char_count` | semantic, format-bridge | HWP 문단 문자 수. 문단 끝 마커 포함 |
| `control_mask` | semantic, backend-contract | controls의 char code bit OR. 저장 시 재계산 필요 |
| `para_shape_id`, `style_id` | reference | DocInfo ParaShape/Style 참조 |
| `column_type`, `raw_break_type` | semantic, raw-preserve | 구역/쪽/단 나누기와 원본 break byte |
| `text` | semantic | UTF-16에서 변환된 문단 텍스트 |
| `char_offsets` | format-bridge, layout | Rust `String` 문자와 HWP UTF-16 code unit 위치 연결 |
| `char_shapes` | semantic, reference | 위치별 CharShape ID run |
| `line_segs` | layout, backend-contract | HWP 조판 줄 배열. 별도 표준 문서 존재 |
| `range_tags` | semantic | PARA_RANGE_TAG |
| `field_ranges` | derived-cache, semantic | FIELD_BEGIN~FIELD_END 텍스트 범위 |
| `controls` | semantic | 표/그림/도형/각주/필드 등 확장 컨트롤 |
| `ctrl_data_records` | raw-preserve | 각 컨트롤에 대응하는 CTRL_DATA raw |
| `char_count_msb` | format-bridge, backend-contract | 문단 list scope 마지막 문단 표시 |
| `raw_header_extra` | raw-preserve, backend-contract | PARA_HEADER tail, instanceId/변경추적 suffix 보존 |
| `has_para_text` | raw-preserve, backend-contract | 빈 문단의 PARA_TEXT 생략 계약 보존 |
| `tab_extended` | raw-preserve, format-bridge | TAB 8 code unit 확장 데이터 |
| `numbering_restart` | semantic, format-bridge | 문단 번호 이어가기/새 시작 override |

1차 관찰:

- `Paragraph`는 semantic 텍스트 모델, HWP record contract, layout snapshot이 한 구조체에
  결합되어 있다.
- `char_offsets`와 `line_segs.text_start`는 반드시 함께 보아야 한다.
- `raw_header_extra`, `char_count_msb`, `has_para_text`는 겉보기에는 raw 필드지만 HWP 저장
  안정성에 직접 연결되어 있어 단순 제거 대상이 아니다.

### 4.2 `LineSeg`

| 필드 | 태그 | 메모 |
|---|---|---|
| `text_start` | layout, format-bridge | 문단 시작 기준 UTF-16 code unit |
| `vertical_pos` | layout | 페이지 상단 기준 누적 y 좌표로 표준화됨 |
| `line_height`, `text_height`, `baseline_distance`, `line_spacing` | layout | 줄 높이/텍스트 높이/베이스라인/간격 |
| `column_start`, `segment_width` | layout | wrap zone 및 단 내 line segment 폭 |
| `tag` | layout, format-bridge | HWP5 PARA_LINE_SEG flag |

1차 관찰:

- `LineSeg`는 현재 가장 명확하게 표준화된 IR 하위 구조다.
- 다만 `line_height` 주석에는 “line_spacing 포함”이라고 되어 있고, 일부 렌더러 측
  실증 주석은 `line_height + line_spacing` advance를 사용한다. 이 해석 차이는 Stage 2에서
  소비처별로 재확인해야 한다.

## 5. 컨트롤 IR

### 5.1 `Control`

`Control` enum은 문단 내 확장 컨트롤을 한곳에 모은다.

| variant 그룹 | 태그 | 메모 |
|---|---|---|
| `SectionDef`, `ColumnDef` | semantic, layout | 문단 안에 등장하는 구역/단 정의 |
| `Table`, `Shape`, `Picture`, `Equation`, `Form` | semantic, layout | TAC/부동 개체와 렌더링 대상 |
| `Header`, `Footer`, `Footnote`, `Endnote`, `HiddenComment` | semantic | 하위 paragraph list를 가진 컨트롤 |
| `AutoNumber`, `NewNumber`, `PageNumberPos` | semantic, format-bridge | 번호 필드 |
| `Bookmark`, `Hyperlink`, `Ruby`, `CharOverlap`, `PageHide`, `Field` | semantic | 인라인/필드 계열 |
| `Unknown` | raw-preserve gap | 현재는 `ctrl_id`만 보존. raw payload 보존성 확인 필요 |

### 5.2 주요 컨트롤 구조체

| 구조체 | 필드 그룹 | 태그 | 메모 |
|---|---|---|---|
| `Equation` | `common`, `script`, `font_size`, `color`, `baseline`, `version_info`, `font_name` | semantic, layout | 수식 컨트롤 |
| `Equation` | `unknown`, `raw_ctrl_data` | raw-preserve, format-bridge | HWP5 spec 누락/원본 CTRL_DATA 보존 |
| `Field` | `field_type`, `command`, `properties`, `field_id`, `ctrl_id` | semantic, backend-contract | fieldBegin/CTRL_DATA 연결 |
| `Field` | `ctrl_data_name`, `memo_index`, `memo_paragraphs` | semantic, format-bridge | 누름틀/메모 필드 확장 |
| `Field` | `raw_parameters_xml` | raw-preserve, format-bridge | HWPX `<hp:parameters>` 원문 보존 |
| `FormObject` | `form_type`, `name`, `caption`, `text`, `width`, `height`, 색상/상태 | semantic, layout | 양식 개체 |
| `FormObject` | `properties` | raw-preserve | 원본 키-값 보존 |

1차 관찰:

- `UnknownControl`이 `ctrl_id`만 갖는 점은 Stage 2/3에서 raw preservation gap 후보로 볼 수 있다.
- `Field.raw_parameters_xml`처럼 HWPX roundtrip을 위한 verbatim XML 보존 필드가 늘고 있다.

## 6. 개체/도형/그림 IR

### 6.1 `CommonObjAttr`

| 필드 그룹 | 태그 | 메모 |
|---|---|---|
| `ctrl_id`, `attr`, `vertical_offset`, `horizontal_offset`, `width`, `height`, `z_order`, `margin`, `instance_id`, `prevent_page_break` | semantic, layout, backend-contract | HWP5 CTRL_HEADER 기반 공통 개체 속성 |
| `treat_as_char`, `vert_rel_to`, `vert_align`, `horz_rel_to`, `horz_align`, `text_wrap`, `text_flow`, `width_criterion`, `height_criterion` | semantic, layout | 개체 배치 계약 |
| `flow_with_text`, `allow_overlap`, `hwp5_gen_shape_attr_bit26`, `size_protect`, `hwp5_gen_shape_attr_bit28` | format-bridge, raw-preserve | HWPX/HWP5 GenShape attr materialization 후보 |
| `description`, `numbering_type` | semantic, raw-preserve | 개체 설명/캡션 번호 범주 |
| `raw_extra` | raw-preserve | 파싱된 필드 이후 tail |

1차 관찰:

- `CommonObjAttr`는 layout semantic과 HWPX→HWP5 저장용 bit 후보가 같은 계층에 있다.
- `hwp5_gen_shape_attr_bit26/28` 이름은 매우 backend-contract 성격이 강하다.

### 6.2 `ShapeComponentAttr`

| 필드 그룹 | 태그 | 메모 |
|---|---|---|
| `ctrl_id`, `is_two_ctrl_id`, `local_file_version` | raw-preserve, format-bridge | SHAPE_COMPONENT record 호환 |
| `offset_x`, `offset_y`, `group_level`, `original_*`, `current_*`, `rotation_*`, `flip` | semantic, layout | 도형 위치/크기/회전 |
| `horz_flip`, `vert_flip`, `rotate_image` | semantic, format-bridge | 원본 bit에서 파생/보존되는 편의 필드 |
| `raw_rendering` | raw-preserve | 변환 행렬 원본 바이트 |
| `render_tx`, `render_ty`, `render_sx`, `render_sy`, `render_b`, `render_c` | derived-cache, layout | 렌더링 행렬 계산 결과 |

1차 관찰:

- `raw_rendering`과 `render_*`는 원본 보존과 계산 결과가 같이 존재한다.
- Stage 2에서 serializer가 어느 쪽을 source-of-truth로 삼는지 확인해야 한다.

### 6.3 `Picture`

| 필드 그룹 | 태그 | 메모 |
|---|---|---|
| `common`, `shape_attr`, 테두리/자르기/여백/이미지 속성 | semantic, layout | 그림 개체 |
| `href` | format-bridge, raw-preserve | HWPX href → HWP CTRL_DATA materialization |
| `raw_picture_extra` | raw-preserve | SHAPE_PICTURE tail |
| `effects`, `img_dim` | raw-preserve, semantic | HWPX 효과와 원본 이미지 픽셀 크기 |
| `caption` | semantic | 캡션 paragraph list |

### 6.4 도형/차트/OLE

| 구조체 | 태그 | 메모 |
|---|---|---|
| `DrawingObjAttr` | semantic, layout | 선/채우기/그림자/글상자/캡션 |
| `TextBox` | semantic, raw-preserve, format-bridge | `vertical_all`, `raw_list_header_extra` 포함 |
| `LineShape`, `RectangleShape`, `EllipseShape`, `ArcShape`, `PolygonShape`, `CurveShape`, `GroupShape` | semantic, layout | 기본 도형 |
| `ConnectorData`, `raw_trailing` | raw-preserve, semantic | 연결선 확장/끝 패딩 |
| `ChartShape.raw_chart_data` | raw-preserve | 차트 원본 데이터 보존 |
| `OleShape.raw_tag_data`, `preview` | raw-preserve, semantic | OLE 원본/프리뷰 |

## 7. 표 IR

### 7.1 `Table`

| 필드 | 태그 | 메모 |
|---|---|---|
| `attr`, `row_count`, `col_count`, `cell_spacing`, `padding`, `row_sizes`, `border_fill_id`, `zones`, `cells` | semantic, layout | 표 기본 구조 |
| `cell_grid` | derived-cache | 셀 빠른 조회용 2D 인덱스 |
| `page_break`, `repeat_header`, `caption`, `common`, outer margins | semantic, layout | 표 조판/캡션/공통 속성 |
| `raw_ctrl_data` | raw-preserve, backend-contract | 현재 HWP serializer의 Table CTRL_HEADER source-of-truth |
| `raw_table_record_attr`, `raw_table_record_extra` | raw-preserve | HWPTAG_TABLE 원본 보존 |
| `dirty` | edit-state, derived-cache | 증분 측정/레이아웃 재계산 플래그 |

1차 관찰:

- `Table.raw_ctrl_data`는 주석상 명시적으로 source-of-truth이다. `common`과 dual
  maintenance 구조이므로 가장 위험도가 높은 감사 대상이다.
- `dirty`는 문서 저장 포맷 필드가 아니라 editor/layout cache 상태다. 공통 IR에 들어와
  있으므로 Stage 2에서 갱신/초기화 경로를 확인해야 한다.

### 7.2 `Cell`

| 필드 그룹 | 태그 | 메모 |
|---|---|---|
| `col`, `row`, `col_span`, `row_span`, `width`, `height`, `padding`, `border_fill_id` | semantic, layout | 셀 위치/크기/서식 |
| `paragraphs` | semantic | 셀 하위 paragraph list |
| `list_header_width_ref`, `raw_list_extra` | raw-preserve, backend-contract | LIST_HEADER 보존 |
| `text_direction`, `vertical_align`, `apply_inner_margin`, `is_header`, `field_name` | semantic, layout, format-bridge | 셀 속성 |

## 8. 스타일/서식 IR

| 구조체 | 필드 성격 | 1차 메모 |
|---|---|---|
| `Font` | semantic, raw-preserve, format-bridge | `raw_data`, `subst_font` 포함 |
| `CharShape` | semantic, raw-preserve | `raw_data`와 decoded fields 공존. `PartialEq`는 raw 제외 |
| `ParaShape` | semantic, raw-preserve | `raw_data`, `attr1/2/3`, decoded fields 공존 |
| `Numbering` | semantic, raw-preserve | `raw_para_heads`로 HWPX 10수준/원문 XML 보존 |
| `Bullet` | semantic, raw-preserve | `raw_data` 포함 |
| `TabDef` | semantic, raw-preserve | `raw_data`, `tabs`, 자동 탭 |
| `Style` | semantic, raw-preserve | `raw_data`, `lang_id` 포함 |
| `BorderFill` | semantic, raw-preserve | `raw_data`, 대각선/채우기 |
| `Fill`, `GradientFill`, `ImageFill` | semantic, reference | fill 상세 |
| `CharShapeMods`, `ParaShapeMods` | edit command DTO | 실제 문서 내용 IR이라기보다 편집 명령용 patch 구조 |

1차 관찰:

- style 계층은 raw record 보존과 decoded semantic 필드가 병행된다.
- `CharShapeMods`/`ParaShapeMods`는 `src/model`에 있지만 저장 포맷 문서 IR보다 command patch
  object 성격이 강하다. 패키지/모듈 경계 검토 후보로 표시한다.

## 9. 페이지/구역 보조 IR

| 구조체 | 필드 성격 | 1차 메모 |
|---|---|---|
| `PageDef` | semantic, layout | 용지와 여백 |
| `PageDef.pagination_bottom_tolerance` | layout, backend-contract | 파일 포맷 필드가 아닌 paginator 허용치 |
| `PageBorderFill` | semantic, layout, format-bridge | `basis`와 `ui_basis`를 분리해 renderer contract와 UI/raw 의미 분리 |
| `ColumnDef` | semantic, layout, raw-preserve | `proportional_widths`, `raw_attr` 포함 |
| `PageAreas` | derived-cache, layout | `PageDef`에서 계산되는 렌더링 영역. 저장 IR은 아님 |

1차 관찰:

- `PageBorderFill.basis/ui_basis`는 포맷 차이를 공통 렌더러 계약으로 승격한 필드다.
- `pagination_bottom_tolerance`와 `PageAreas`는 포맷 저장 필드가 아닌 layout layer에 가까운
  값이다.

## 10. 각주/머리말/바이너리 IR

| 구조체 | 필드 성격 | 1차 메모 |
|---|---|---|
| `Footnote`, `Endnote` | semantic, format-bridge | 번호/장식문자/number_shape/instance/list_header_property와 paragraph list |
| `FootnoteShape` | semantic, format-bridge | HWP5/HWPX 원본 슬롯과 한컴 UI 의미가 함께 존재 |
| `FootnoteShape.raw_unknown` | raw-preserve, semantic | HWP5 미문서화 2바이트. UI의 주석 사이 값 |
| `Header`, `Footer` | semantic, raw-preserve, format-bridge | HWPX subList materialization 값 포함 |
| `MasterPage` | semantic, layout, raw-preserve, format-bridge | 확장/대체 바탕쪽, HWPX pageNumber, raw LIST_HEADER |
| `BinData` | semantic, raw-preserve, reference | 파일 참조/임베드/OLE 스토리지 참조 |
| `BinDataContent` | backend-contract | 실제 바이너리 payload |

## 11. Stage 1 예비 관심 영역

다음은 제거/통합 확정이 아니라 Stage 2 생산자/소비자 추적 우선순위다.

| 영역 | 이유 |
|---|---|
| `Document.is_hwp3_variant` | 포맷 출처/변환본 힌트가 renderer/layout에 직접 전달됨 |
| `PageDef.pagination_bottom_tolerance` | 파일 포맷 필드가 아닌 paginator hint가 model에 있음 |
| `Table.raw_ctrl_data` vs `Table.common` | 주석상 Table만 raw bytes가 serializer source-of-truth |
| `Table.dirty` | 편집/증분 layout 상태가 문서 IR 구조체에 포함됨 |
| `CommonObjAttr.hwp5_gen_shape_attr_bit26/28` | backend materialization 후보가 공통 속성에 포함됨 |
| `ShapeComponentAttr.raw_rendering` vs `render_*` | raw 행렬과 계산 행렬이 공존 |
| `UnknownControl` | 알 수 없는 컨트롤 raw payload 보존성이 낮아 보임 |
| `CharShapeMods`/`ParaShapeMods` | model 모듈 안의 편집 patch DTO |
| HWPX verbatim 보존 필드들 | `raw_parameters_xml`, `raw_para_heads`, `hwpx_head_tail`, `img_dim`, `vertical_all` 등 증가 추세 |

## 12. Stage 2 작업 항목

Stage 2에서는 위 필드별로 실제 생산자/소비자를 추적한다.

- parser/frontend가 값을 채우는 위치
- document_core가 값을 변경하는 위치
- renderer/layout이 값을 source-of-truth로 쓰는 위치
- serializer가 raw를 그대로 쓰는지 decoded field를 재생성하는지
- tests/diagnostics에서만 쓰이는 필드인지

특히 `raw-preserve` 필드는 소비처가 한 곳뿐이어도 HWP/HWPX 호환 계약일 수 있으므로,
단순 참조 횟수만으로 제거 후보를 확정하지 않는다.

## 13. Stage 2.1 parser/frontend 생산자 매트릭스

본 절은 소비자 판단이 아니라 "어느 frontend가 어떤 IR 필드를 생산하거나 materialize하는가"만
기록한다. 같은 필드라도 HWP5는 원본 raw 보존, HWPX는 XML 의미를 HWP5 writer 계약으로
lowering, HWP3는 legacy 문서를 HWP5 호환 IR로 변환하는 식으로 생산 의미가 다르다.

### 13.1 Document 생성 entry point

| Frontend | 주요 생산 지점 | 생산 필드 | 메모 |
|---|---|---|---|
| HWP5 strict/lenient | `src/parser/mod.rs:255`, `264`, `295`, `475`, `511`, `531`, `566` | `DocInfo.raw_stream`, `Section.raw_stream`, `Document.extra_streams`, `hwpx_aux_entries`, `is_hwp3_variant` | CFB 원본 stream 보존이 중심이다. HWP3 변환본 휴리스틱은 `apply_hwp3_origin_fixup` 이후 `is_hwp3_variant`와 layout 보정값을 만든다. |
| HWPX | `src/parser/hwpx/mod.rs:155`, `167`, `337`, `347` | `Document.extra_streams`, `hwpx_aux_entries`, `is_hwp3_variant=false` | ZIP/HWPX container를 공통 Document로 lowering한다. HWPX 보조 파일과 HWP5 저장 계약 stream을 별도 채널로 보존한다. |
| HWPX HWP3-origin 보정 | `src/parser/hwpx/mod.rs:265` | `PageDef.pagination_bottom_tolerance` | HWP3 변환본 HWPX에 한해 margin 원본은 유지하고 pagination 전용 허용치만 주입한다. |
| HWP3 | `src/parser/hwp3/mod.rs:2508`, `2520`, `2532`, `2820`, `2877`, `2881` | `Document.header.version.major=3`, HWP5 `header.raw_data`, `SectionDef.page_def`, `Section.raw_stream=None`, `DocInfo` 스타일/이미지 목록 | HWP3 parser는 원본 구조를 직접 보존하기보다 HWP5 serializer와 renderer가 읽을 수 있는 IR로 변환한다. |
| Ingest/generated | `src/document_core/builders/exam_paper.rs:23`, `92`, `97`, `109` | `Document`, `Paragraph.char_offsets`, `Paragraph.line_segs`, `has_para_text` | `parser/ingest`는 schema 파싱이고 실제 Document 생산은 builder가 담당한다. 파일 포맷 frontend가 아닌 생성 frontend로 분리해 봐야 한다. |

### 13.2 DocInfo/style 계층 생산자

| 필드군 | HWP5 생산자 | HWPX 생산자 | HWP3 생산자 | 생산 의미 |
|---|---|---|---|---|
| `font_faces`, `bin_data_list`, `border_fills`, `char_shapes`, `tab_defs`, `numberings`, `bullets`, `para_shapes`, `styles` | `src/parser/doc_info.rs:60`, `72`, `87`-`138` | `src/parser/hwpx/header.rs:101`, `161`, `185`, `447`, `791`, `876`, `1292`, `1539`, `1675`, `1798` | `src/parser/hwp3/mod.rs:2607`, `2615`-`2641`, `2678`, `2743`, `2811`, `2881`-`2886` | 세 frontend 모두 공통 `DocInfo`를 채우지만, HWP3는 기본값/변환 스타일을 생성하고 HWPX는 XML 의미를 HWP5 스타일 모델로 낮춘다. |
| `DocInfo.raw_stream`, `extra_records`, `memo_shape_count` | `src/parser/mod.rs:255`, `src/parser/doc_info.rs:82`, `142` | `src/parser/hwpx/header.rs:263`-`287`, `325`, `330` | 직접 raw stream 없음 | HWP5는 stream bytes, HWPX는 roundtrip용 pseudo raw record를 생산한다. |
| `hwpx_head_tail`, `Numbering.raw_para_heads` | 없음 | `src/parser/hwpx/header.rs:208`, `1794` | 없음 | HWPX XML verbatim 보존 채널이다. HWP5/HWP3 semantic 필드와 성격이 다르다. |

### 13.3 Paragraph와 LineSeg 생산자

| 필드군 | 생산자 | 메모 |
|---|---|---|
| HWP5 paragraph text/offset | `src/parser/body_text.rs:148`-`153`, `263`-`382` | `PARA_TEXT` record에서 `text`, `char_offsets`, `field_ranges`, `tab_extended`, `has_para_text`를 직접 생산한다. |
| HWP5 line segment | `src/parser/body_text.rs:159`, `428` | `PARA_LINE_SEG` record를 `LineSeg` 배열로 직접 변환한다. |
| HWP5 paragraph raw header | `src/parser/body_text.rs:235`, `250` | `raw_break_type`, `raw_header_extra`는 원본 header 보존/재직렬화 계약이다. |
| HWP5 control data side channel | `src/parser/body_text.rs:199` | `Paragraph.ctrl_data_records`는 control record와 paragraph text 위치를 연결하는 보조 채널이다. |
| HWPX paragraph text/offset | `src/parser/hwpx/section.rs:421`, `595`-`669` | XML run/control 순서를 HWP5형 offset과 field range로 재구성한다. |
| HWPX line segment | `src/parser/hwpx/section.rs:574`, `1403` | `hp:linesegarray`가 있으면 `LineSeg`를 채우고, 없는 경우는 빈 배열을 유지한다. |
| HWPX paragraph raw header | `src/parser/hwpx/section.rs:688`-`744` | HWPX 값을 HWP5 `raw_break_type`/`raw_header_extra` 저장 계약으로 materialize한다. |
| HWP3 paragraph text/offset | `src/parser/hwp3/mod.rs:364`, `366`, `1883`, `1886`, `1887` | legacy 본문 stream을 UTF-16 offset, control side channel, `has_para_text`로 변환한다. |
| HWP3 line segment | `src/parser/hwp3/mod.rs:2045`, `2055`, `2162`, `2226`, `2228` | HWP3 line info와 보정 로직으로 공통 `LineSeg`를 생성한다. 원본 raw record 보존보다 layout 정합 생성에 가깝다. |
| Generated paragraph | `src/document_core/builders/exam_paper.rs:92`-`109` | 외부 ingest에서 공통 IR을 직접 만든다. parser raw 보존 필드는 생산하지 않는다. |

### 13.4 Control/Table/Shape 생산자

| 필드군 | 생산자 | 생산 의미 |
|---|---|---|
| `Field.raw_parameters_xml` | `src/parser/control.rs:136`, `src/parser/hwpx/section.rs:4600` | HWP5는 없음, HWPX는 원문 XML parameter 보존. |
| `Table.raw_ctrl_data` | `src/parser/control.rs:155`, `src/parser/hwp3/mod.rs:853`, `src/document_core/html_table_import.rs:509`-`608`, `src/document_core/converters/hwpx_to_hwp.rs:1362`-`1375` | HWP5는 원본 control header, HWP3/html import/HWPX-HWP 변환은 HWP5 writer용 raw bytes를 생성한다. |
| `Table.raw_table_record_attr`, `raw_table_record_extra` | `src/parser/control.rs:260`, `306`, `src/parser/hwpx/section.rs:1810`, `src/document_core/converters/hwpx_to_hwp.rs:1472`, `1498` | HWP5/HWPX 파서와 HWPX-HWP materializer가 모두 참여한다. |
| `Cell.raw_list_extra`, `field_name` | `src/parser/control.rs:378`-`380`, `src/parser/hwpx/section.rs:1957`-`1962`, `src/document_core/html_table_import.rs:466`-`484`, `src/document_core/converters/hwpx_to_hwp.rs:1413`-`1424` | 셀 semantic과 HWP5 LIST_HEADER 저장 계약이 같은 구조체에 공존한다. |
| `Header/Footer.raw_ctrl_extra` | `src/parser/control.rs:484`, `508`, `src/parser/hwp3/mod.rs:1708`, `1715`, `src/parser/hwpx/section.rs:4220`, `4249` | HWP5/HWP3는 raw 또는 pseudo raw, HWPX는 subList를 HWP5 저장 형태로 materialize한다. |
| `Equation.raw_ctrl_data` | `src/parser/control.rs:790`, `src/document_core/converters/hwpx_to_hwp.rs:1113`-`1116` | HWP5 원본 raw와 변환 시 raw clear 정책이 모두 존재한다. |
| `CommonObjAttr.hwp5_gen_shape_attr_bit26/28` | `src/parser/control/shape.rs:342`-`343`, `src/parser/hwpx/section.rs:2151`, `2706`, `5420`, `5462`, `5489`, `5530` | HWP5는 attr bit 해석, HWPX는 HWP5 저장 시 생성해야 할 bit를 explicit flag로 만든다. |
| `ShapeComponentAttr.render_*`, `raw_rendering` | `src/parser/control/shape.rs:551`, `561`, `src/parser/hwpx/section.rs:3011`, `3133`, `3137`, `src/document_core/converters/hwpx_to_hwp.rs:1164`-`1191` | 계산된 affine 값과 HWP5 raw rendering 행렬이 병존한다. |
| `Picture.raw_picture_extra`, `img_dim` | `src/parser/control/shape.rs:916`, `src/parser/hwpx/section.rs:2157`, `2267`, `2479` | HWP5 잔여 bytes와 HWPX 원본 이미지 pixel dimension 보존은 서로 다른 출처의 format bridge다. |
| `TextBox.vertical_all` | `src/parser/hwpx/section.rs:3432` | HWPX `VERTICALALL` 의미를 별도 semantic/layout flag로 승격한다. |

### 13.5 Page/section 보조 필드 생산자

| 필드군 | 생산자 | 메모 |
|---|---|---|
| `PageDef.pagination_bottom_tolerance` | `src/parser/hwpx/mod.rs:265`, `src/parser/hwp3/mod.rs:2833` | HWP3 계열 pagination 보정값이다. 파일 포맷 저장 필드가 아니라 layout hint다. |
| `PageBorderFill.basis/ui_basis` | `src/parser/body_text.rs:833` 이후, `src/parser/hwpx/section.rs:1137`-`1143`, `src/parser/hwp3/mod.rs:94`-`95` | HWP5/HWPX/HWP3의 페이지 테두리 기준을 공통 renderer 계약으로 변환한다. |
| `SectionDef.raw_ctrl_extra` | `src/parser/body_text.rs:549`, `src/document_core/converters/hwpx_to_hwp.rs:1075`-`1092` | HWP5는 원본 ctrl extra, HWPX-HWP 변환은 writer 계약을 위해 생성한다. |
| `Section.raw_stream` | `src/parser/mod.rs:475`, `531`, `src/parser/hwp3/mod.rs:2877` | HWP5만 원본 body stream을 보존한다. HWP3는 `None`이다. |

### 13.6 생산자 관찰

- 공통 `Document` IR은 단순 AST가 아니라 HWP5/HWPX/HWP3 frontend가 renderer/serializer backend
  계약까지 일부 포함해 lowering한 중간 표현이다.
- 같은 `raw_*` 이름이라도 의미가 세 가지로 갈린다.
  - 원본 bytes 보존: HWP5 `raw_stream`, `raw_ctrl_data`, `raw_rendering`
  - XML verbatim 보존: HWPX `raw_parameters_xml`, `raw_para_heads`, `hwpx_head_tail`
  - HWP5 writer용 materialization: HWP3/HWPX-HWP/html import의 `raw_ctrl_data`,
    `raw_list_extra`, `raw_header_extra`
- HWP3 관련 필드는 `Document.is_hwp3_variant`, `PageDef.pagination_bottom_tolerance`,
  HWP3 parser의 generated line segment처럼 포맷 출처와 layout hint가 공통 IR에 들어온다.
- `document_core`도 일부 경우 frontend처럼 IR 생산자가 된다. 특히 ingest builder와 HTML table
  import, HWPX-HWP converter는 parser가 아니지만 저장 가능한 Document를 만들기 위해 raw/backend
  계약 필드를 직접 생성한다.
- Stage 3 분류에서는 필드 제거 여부보다 먼저 `semantic IR`, `source-preserve`,
  `backend-materialized`, `layout-hint`, `edit-state`를 분리해 판단해야 한다.

## 14. Stage 2.2 document_core/renderer/serializer 소비자 매트릭스

본 절은 Stage 2.1의 생산자와 대응되는 소비자를 기록한다. 소비처는 단순 참조가 아니라
다음 성격으로 구분한다.

- `roundtrip-source`: 원본 또는 materialized raw가 serializer의 우선 입력이다.
- `layout-source`: renderer/typeset/pagination이 조판 source-of-truth로 사용한다.
- `edit-cache-control`: 편집 명령이 원본 stream을 무효화하거나 surgical patch한다.
- `xml-verbatim`: HWPX serializer가 XML 원문 조각을 그대로 splice한다.
- `diagnostic-only`: 진단/테스트 중심이며 runtime 계약은 약하다.

### 14.1 원본 stream과 컨테이너 보존

| 필드 | 주요 소비자 | 소비 성격 | 관찰 |
|---|---|---|---|
| `DocInfo.raw_stream`, `raw_stream_dirty` | `src/serializer/doc_info.rs:22`, `24`, `124`; `src/document_core/commands/text_editing.rs:39`, `105`; `src/document_core/converters/hwpx_to_hwp.rs:254`, `338`, `566` | `roundtrip-source`, `edit-cache-control` | `raw_stream_dirty=false`이면 DocInfo 전체 재생성보다 원본 stream이 우선한다. 편집 명령은 body raw는 무효화하면서 DocInfo는 caret만 surgical update하는 경로가 있다. |
| `Section.raw_stream` | `src/serializer/body_text.rs:26`, `28`; `src/document_core/commands/text_editing.rs:39`; `src/document_core/commands/table_ops.rs:55` 등 | `roundtrip-source`, `edit-cache-control` | section body는 원본 stream이 있으면 그대로 저장된다. 대부분의 편집/표/객체/서식 명령은 해당 section raw를 `None`으로 만들어 모델 재직렬화를 강제한다. |
| `Document.extra_streams` | `src/serializer/cfb_writer.rs:49`, `76`, `156`; `src/document_core/queries/form_query.rs:201`, `327`, `360` | `roundtrip-source`, semantic query | CFB의 Scripts/DocOptions 계열은 HWP 저장 시 그대로 복원되고, form query는 script stream을 직접 읽어 콤보박스 항목 등을 추출한다. |
| `Document.hwpx_aux_entries` | `src/model/document.rs:256`; `src/serializer/hwpx/mod.rs:53`, `57`, `89`, `92` | HWPX aux roundtrip | HWPX `version.xml`, `settings.xml`, `Preview/*`의 원본 보존 우선 경로다. HWP5 `extra_streams`와 성격이 비슷하지만 HWPX ZIP 보조 엔트리 전용 채널이다. |

### 14.2 Paragraph/LineSeg 소비자

| 필드군 | 주요 소비자 | 소비 성격 | 관찰 |
|---|---|---|---|
| `Paragraph.line_segs` | `src/serializer/body_text.rs:260`; `src/renderer/height_measurer.rs:213`, `274`, `385`; `src/renderer/pagination/engine.rs:33`, `134`, `265`; `src/renderer/layout/paragraph_layout.rs:640`, `752`, `4931`; `src/document_core/queries/rendering.rs:3710`-`3846` | `roundtrip-source`, `layout-source` | 라인 세그먼트는 serializer의 `PARA_LINE_SEG` 입력이면서 renderer/pagination의 핵심 조판 기준이다. HWPX처럼 없는 경우의 fallback도 renderer 쪽에 별도 로직이 있다. |
| `Paragraph.char_offsets` | `src/serializer/body_text.rs:414`, `421`, `434`; `src/renderer/composer.rs:178`, `747`, `779`; `src/renderer/layout.rs:1686`; `src/renderer/layout/paragraph_layout.rs:640`, `820`; `src/document_core/helpers.rs:34`, `245`; `src/document_core/commands/text_editing.rs:92`-`105` | serializer placement, layout-source, edit cursor | 텍스트와 control의 원본 UTF-16 위치를 복원하는 핵심 배열이다. 필드/자동번호/컨트롤 삽입 위치와 caret 갱신 모두 여기에 의존한다. |
| `Paragraph.field_ranges` | `src/serializer/body_text.rs:308`, `421`, `434`; `src/renderer/layout/paragraph_layout.rs:4349`; `src/document_core/queries/field_query.rs:67`, `102`, `149` | serializer marker, layout/query | FIELD_END marker 재삽입과 field query 범위 계산에 쓰인다. 단순 HWPX 보존 필드가 아니라 runtime query 계약도 있다. |
| `Paragraph.tab_extended` | `src/renderer/composer.rs:345`; `src/renderer/layout/paragraph_layout.rs:469`, `481`, `2075`, `2260` | layout-source | inline tab/cross-run tab 처리용 renderer 계약이다. serializer HWP5 쪽 직접 소비는 약하고 layout 의미가 강하다. |
| `Paragraph.has_para_text` | `src/serializer/body_text.rs:210`, `132`; `src/document_core/converters/hwpx_to_hwp.rs:1035` | serializer guard | text/control이 없더라도 `PARA_TEXT`를 써야 하는 문맥을 보존하는 저장용 flag 성격이 강하다. |
| `Paragraph.raw_break_type`, `raw_header_extra` | `src/serializer/body_text.rs:348`, `364`, `367`; `src/document_core/converters/hwpx_to_hwp.rs:632`, `965`, `975`, `1235` | `roundtrip-source`, backend materialization | break type과 header extra는 HWP5 `PARA_HEADER` 재생성의 source다. HWPX-HWP 변환 pass가 line seg count와 instance id를 맞추기 위해 직접 갱신한다. |
| `Paragraph.ctrl_data_records` | `src/serializer/body_text.rs:284`; `src/document_core/converters/hwpx_to_hwp.rs:802`, `813`, `2307` | control side-channel | paragraph의 control 순서와 CTRL_DATA record를 연결하는 serializer side-channel이다. semantic 문서 내용이라기보다 HWP backend 계약에 가깝다. |

### 14.3 HWP3/layout hint 소비자

| 필드 | 주요 소비자 | 소비 성격 | 관찰 |
|---|---|---|---|
| `Document.is_hwp3_variant` | `src/document_core/queries/rendering.rs:2070`, `2072`, `2236`, `2253`, `2268`, `3114`; `src/renderer/pagination/engine.rs:134`, `137`, `150`, `265`; `src/renderer/height_measurer.rs:166`, `180` | `layout-source` | renderer/pagination/height 측정의 HWP3-origin 보정 스위치다. 포맷 출처 플래그가 layout 정책에 직접 연결되어 있다. |
| `PageDef.pagination_bottom_tolerance` | `src/renderer/page_layout.rs:88`-`103`, `147`; `src/document_core/queries/rendering.rs:26`, `2236` | `layout-source` | `PageLayoutInfo`가 px tolerance로 변환해 가용 본문 높이에 더한다. 파일 포맷 필드가 아닌 pagination policy hint로 보는 편이 맞다. |
| HWP3 line segment 생성물 | `src/renderer/pagination/engine.rs:265` 이후 vpos reset 감지; `src/renderer/height_measurer.rs:385` 이후 줄 높이 계산 | `layout-source` | HWP3 parser가 만든 `LineSeg`는 단순 입력 보존이 아니라 HWP3 조판 정합을 위한 generated layout IR로 소비된다. |

### 14.4 Table/Control raw 계약 소비자

| 필드군 | 주요 소비자 | 소비 성격 | 관찰 |
|---|---|---|---|
| `Table.raw_ctrl_data` | `src/serializer/control.rs:469`-`476`; `src/document_core/commands/table_ops.rs:989`-`1021`, `1113`-`1115`, `1341`-`1392`; `src/document_core/converters/hwpx_to_hwp.rs:1362`-`1375`; `src/document_core/converters/diagnostics.rs:75`, `128`; `src/renderer/typeset.rs:11410` | `roundtrip-source`, edit backend contract | HWP serializer는 table CTRL_HEADER를 `raw_ctrl_data` 그대로 쓴다. table ops는 offset/size bytes를 직접 patch한다. typeset 주석은 raw보다 `common.vertical_offset`를 권위값으로 본다고 명시한다. |
| `Table.raw_table_record_attr`, `raw_table_record_extra` | `src/serializer/control.rs:501`-`506`, `539`; `src/document_core/converters/hwpx_to_hwp.rs:1457`, `1472`, `1498`; `src/serializer/hwpx/table.rs:68` | `roundtrip-source`, backend materialization | HWP 저장과 HWPX table attr 출력 모두에 영향을 준다. semantic table flag와 raw attr이 병존하므로 Stage 3에서 `split/document` 후보로 본다. |
| `Cell.raw_list_extra` | `src/serializer/control.rs:577`; `src/document_core/converters/hwpx_to_hwp.rs:1402`, `1413`-`1424`; `src/document_core/html_table_import.rs:466`-`484`; `src/document_core/commands/object_ops.rs:1501`, `1860` | `roundtrip-source`, backend materialization | 셀 width/list header extra를 HWP5 저장에 맞추는 raw tail이다. 생성 표에서는 빈 값 또는 synthetic extra가 모두 가능하다. |
| `Cell.field_name` | `src/serializer/hwpx/table.rs:227`; `src/document_core/queries/field_query.rs:781`-`806`; `tests/issue_493_hwpx_cell_field_name.rs` | semantic query, HWPX serialization | HWP5 raw extra에서 파생되지만 HWPX `name` 속성과 field query의 가상 필드로 쓰인다. 단순 raw cache가 아니라 semantic 승격값이다. |
| `SectionDef.raw_ctrl_extra`, `Header/Footer.raw_ctrl_extra` | `src/serializer/control.rs:250`, `654`, `686`; `src/document_core/converters/hwpx_to_hwp.rs:1075`, `1092` | `roundtrip-source`, backend materialization | Section/Header/Footer의 HWP5 저장 계약이다. HWPX-HWP 변환에서 필요 시 synthetic extra를 만든다. |
| `Equation.raw_ctrl_data` | `src/serializer/control.rs:2246`, `2249`; `src/document_core/converters/hwpx_to_hwp.rs:1113`-`1116` | `roundtrip-source`, edit backend contract | raw가 있으면 serializer가 우선 사용한다. HWPX-HWP adapter는 attr 갱신이 무시되지 않게 raw를 clear하는 정책을 갖는다. |
| `UnknownControl` | `src/serializer/body_text.rs:740`; `src/serializer/control.rs:177`-`179`; `src/diagnostics/hwp5_ctrl_data_trace.rs:383` | weak roundtrip, diagnostic | 현재 소비는 control id/header 수준에 치우쳐 있고 payload 재방출 근거가 약하다. Stage 3에서 보존성 결함 후보로 별도 표시한다. |

### 14.5 Shape/Picture/HWPX verbatim 소비자

| 필드군 | 주요 소비자 | 소비 성격 | 관찰 |
|---|---|---|---|
| `CommonObjAttr.hwp5_gen_shape_attr_bit26/28` | `src/document_core/converters/common_obj_attr_writer.rs:104`-`109`; `tests/issue_1251_ole_chart_contents.rs:284` | backend materialization | renderer는 거의 보지 않고 HWP5 CTRL_HEADER attr bit 합성에서 소비한다. 공통 속성에 backend generation flag가 들어온 사례다. |
| `ShapeComponentAttr.raw_rendering` | `src/serializer/control.rs:1896`; `src/serializer/hwpx/shape.rs:387`, `400`; `src/document_core/commands/object_ops.rs:1032`, `3055`, `4657`; `src/document_core/converters/hwpx_to_hwp.rs:1164`-`1191` | `roundtrip-source`, `xml-verbatim`, edit-cache-control | HWP/HWPX serializer가 모두 소비한다. 편집 시에는 raw를 clear하고 `render_*`를 새 source로 삼는 패턴이 있다. |
| `ShapeComponentAttr.render_tx/render_ty/render_sx/render_sy/render_b/render_c` | `src/renderer/layout/shape_layout.rs:573`-`578`, `1398`-`1405`; `src/serializer/control.rs:1947`-`2056`; `src/document_core/commands/object_ops.rs:1033`-`1036`, `4913`-`4915` | `layout-source`, generated serialization | renderer의 실제 affine source다. raw가 없거나 편집된 경우 HWP serializer가 rendering matrix를 재생성한다. |
| `Picture.raw_picture_extra` | `src/serializer/control.rs:997`; diagnostics/tests 다수 | `roundtrip-source` | HWP5 SHAPE_PICTURE tail 보존 필드다. 의미 해석보다 raw roundtrip 쪽에 강하게 묶여 있다. |
| `Picture.img_dim` | `src/serializer/hwpx/picture.rs:93`, `258`-`266`; `src/serializer/hwpx/roundtrip.rs:391` | `xml-verbatim` | HWPX `hp:imgDim` 원본 픽셀 크기를 verbatim 출력한다. crop/current size 파생값으로 대체하면 안 되는 필드로 문서화 필요하다. |
| `TextBox.vertical_all` | `src/serializer/hwpx/shape.rs:232`, `247`-`249`; roundtrip tests | `xml-verbatim`, semantic flag | HWPX `VERTICAL`/`VERTICALALL` 구분을 보존하기 위한 semantic flag다. |
| `Field.raw_parameters_xml` | `src/serializer/hwpx/section.rs:691`, `698`; `src/serializer/hwpx/roundtrip.rs:1034` | `xml-verbatim` | HWPX field parameters를 모델로 완전히 해석하지 않고 XML 조각으로 보존한다. |
| `DocInfo.hwpx_head_tail`, `Numbering.raw_para_heads` | `src/serializer/hwpx/header.rs:88`, `796`; roundtrip tests | `xml-verbatim` | HWPX head tail과 numbering 10수준 paraHead 원문 splice 채널이다. HWP5 semantic model로 손실 없이 표현하기 어려운 정보를 보존한다. |

### 14.6 Page border 소비자

| 필드 | 주요 소비자 | 소비 성격 | 관찰 |
|---|---|---|---|
| `PageBorderFill.basis` | `src/renderer/layout.rs:1828`, `1846`, `1852`, `1860`, `1943`; `src/document_core/queries/rendering.rs:883`, `904` | `layout-source` | 실제 페이지 테두리 렌더링 위치 기준이다. |
| `PageBorderFill.ui_basis` | `src/document_core/queries/rendering.rs:1111`, `1221`-`1228`, `1271` | UI/API semantic | UI에 노출되는 "쪽 기준/종이 기준" 의미다. `basis`와 분리되어 있으므로 Stage 3에서 명시적 문서화가 필요하다. |

### 14.7 소비자 관찰

- `raw_stream` 계열은 제거 후보가 아니라 "원본 보존 캐시 + 편집 무효화 프로토콜"이다.
  특히 Section raw는 편집 시 광범위하게 `None` 처리되고, DocInfo raw는 surgical patch 경로가
  존재한다.
- `line_segs`와 `char_offsets`는 serializer와 renderer가 동시에 쓰는 교차 계층 계약이다.
  이 둘은 단순 parser artifact가 아니라 현재 IR의 핵심 lowering 결과로 보아야 한다.
- `raw_ctrl_data`는 가장 혼재도가 높다. HWP5 원본 보존, HWPX/HWP3/html 생성물의 HWP5 writer
  materialization, document_core 편집 시 직접 byte patch가 한 필드에 몰려 있다.
- `render_*`는 renderer source이고 `raw_rendering`은 serializer/source-preserve다. 편집 명령은
  raw를 지우고 render 값을 갱신하므로 두 표현의 우선순위 규칙을 명시해야 한다.
- HWPX verbatim 필드(`hwpx_head_tail`, `raw_para_heads`, `raw_parameters_xml`, `img_dim`,
  `vertical_all`)는 공통 semantic model의 부족분을 메우는 보존 채널이다. 제거보다는
  HWPX-only contract 문서화 또는 wrapper 분리가 적합해 보인다.
- `UnknownControl`은 현재 소비 근거가 약하다. unknown payload roundtrip 보존 실패 가능성이 있어
  Stage 3의 `followup_issue` 후보로 올린다.

## 15. Stage 3 후보 분류

분류는 실제 코드 변경 지시가 아니라 후속 판단을 위한 감사 결과다. raw preservation 필드는
참조 횟수가 적어도 roundtrip/source-of-truth 계약이면 제거 후보로 보지 않는다.

### 15.1 분류 기준

| 분류 | 의미 |
|---|---|
| `keep` | 현재 Document IR의 핵심 계약으로 유지한다. |
| `document` | 유지하되 우선순위/소유 계층/포맷 전용 의미를 문서화해야 한다. |
| `split` | semantic IR과 raw/backend/layout/edit 상태를 별도 wrapper 또는 계층으로 분리할 후보. |
| `merge` | 동일 의미의 병렬 표현을 하나의 source-of-truth로 통합할 후보. |
| `remove_candidate` | runtime 계약이 약하거나 model 계층에 둘 이유가 약한 제거/이동 후보. |
| `followup_issue` | 현 상태가 기능 결함 또는 보존성 결함으로 이어질 수 있어 별도 구현 이슈 필요. |

### 15.2 keep

| 대상 | 판정 | 근거 |
|---|---|---|
| `Paragraph.line_segs` | `keep` | HWP serializer의 `PARA_LINE_SEG` 입력이고 renderer/height/pagination의 핵심 조판 기준이다. HWPX 무라인세그 fallback이 있어도 필드 자체는 유지해야 한다. |
| `Paragraph.char_offsets` | `keep` | control gap, field marker, caret, auto-number placeholder, line segmentation 매핑에 쓰인다. parser artifact가 아니라 lowering된 text/control position map이다. |
| `Paragraph.field_ranges` | `keep` | HWP FIELD_END 재삽입, field query, renderer range 처리에 쓰인다. |
| `Paragraph.tab_extended` | `keep` | inline tab/cross-run tab layout 계약이다. HWPX/HWP5 입력 차이를 renderer가 공통으로 소비한다. |
| `PageAreas` | `keep` | `src/model/page.rs:170`의 derived-cache 성격이지만 `src/renderer/page_layout.rs:78`, `117`과 table command에서 계산 헬퍼로 쓰인다. 저장 IR이 아니라 계산 타입임을 문서화하면 된다. |
| `PageBorderFill.basis`, `ui_basis` | `keep` + `document` | renderer 기준과 UI 기준을 분리해야 HWP3/HWP5/HWPX 정합을 동시에 만족한다. 통합하면 과거 이슈가 재발할 위험이 크다. |

### 15.3 document

| 대상 | 판정 | 근거 |
|---|---|---|
| `DocInfo.raw_stream`, `raw_stream_dirty`, `Section.raw_stream` | `document` | 원본 보존 캐시이자 편집 무효화 프로토콜이다. serializer 우선순위와 document_core의 무효화 규칙을 별도 문서에 contract로 남겨야 한다. |
| `Document.extra_streams` | `document` | HWP CFB 추가 stream 보존과 form query의 script 해석에 쓰인다. 저장 컨테이너 계층 필드라는 점을 명시한다. |
| `Document.hwpx_aux_entries` | `document` | HWPX ZIP auxiliary entry 보존 전용이다. HWP `extra_streams`와 목적은 비슷하지만 컨테이너가 다르므로 이름/주석으로 구분하면 충분하다. |
| HWPX verbatim 필드: `hwpx_head_tail`, `Numbering.raw_para_heads`, `Field.raw_parameters_xml`, `Picture.img_dim`, `TextBox.vertical_all` | `document` | 공통 semantic model로 완전히 표현되지 않는 HWPX 정보를 lossless roundtrip으로 보존한다. HWPX-only contract로 명시한다. |
| `PageDef.pagination_bottom_tolerance` | `document` | 파일 포맷 필드가 아닌 pagination hint다. `PageLayoutInfo`가 가용 본문 높이에 더하므로 layout 계층 contract로 문서화한다. |
| `Document.is_hwp3_variant` | `document` + `split` 후보 | 현 renderer 정책의 스위치로 유지 필요. 다만 포맷 출처 플래그가 layout policy에 직접 연결되므로 장기적으로 `LayoutCompatibilityProfile` 같은 policy object로 분리할 수 있다. |

### 15.4 split

| 대상 | 판정 | 근거 |
|---|---|---|
| `Table.raw_ctrl_data`와 `Table.common` | `split` | `Table::update_ctrl_dimensions()` 주석상 table만 raw bytes가 serializer source-of-truth이고 `common`도 layout cache로 동기화한다. semantic/source-preserve/backend-materialized가 한 구조체에 공존한다. |
| `Table.raw_table_record_attr`와 semantic flags(`page_break`, `repeat_header`, `attr`) | `split` | HWP/HWPX serializer가 모두 소비한다. raw attr 우선순위와 semantic flag 재구성 규칙을 wrapper나 accessor로 격리할 필요가 있다. |
| `Cell.raw_list_extra`와 `Cell.width`/`list_header_width_ref`/`field_name` | `split` | 셀 LIST_HEADER tail 보존, HWPX name semantic, width materialization이 혼합되어 있다. HWP5 backend contract 영역으로 분리 후보. |
| `Paragraph.raw_header_extra`, `raw_break_type`, `ctrl_data_records` | `split` | paragraph semantic과 HWP5 `PARA_HEADER`/`CTRL_DATA` serializer side-channel이 같은 구조체에 있다. 제거 불가지만 backend-contract wrapper 후보. |
| `CommonObjAttr.hwp5_gen_shape_attr_bit26/28` | `split` | renderer는 거의 소비하지 않고 HWP5 attr bit materializer만 쓴다. 공통 object attr의 semantic 필드라기보다 HWP5 writer generation flag다. |
| `ShapeComponentAttr.raw_rendering`와 `render_*` | `split` | renderer는 `render_*`, serializer는 raw 우선/없으면 generated matrix를 쓴다. 편집 시 raw clear 규칙이 있으므로 source priority를 타입으로 표현할 후보. |

### 15.5 merge

| 대상 | 판정 | 근거 |
|---|---|---|
| `PageBorderFill.basis`/`ui_basis` | `merge` 아님 | 이름상 유사하지만 renderer 기준과 UI 기준이 다르다. 통합 대상이 아니라 문서화 대상이다. |
| `Table.attr`/`raw_table_record_attr` | `merge` 보류 | 둘이 같은 의미처럼 보이나 HWPX/HWP5 출력에서 서로 다른 역할을 한다. 우선 `split`으로 다루고, 후속 설계에서 accessor 기반 단일 source-of-truth 가능성을 검토한다. |
| `CommonObjAttr`의 decoded fields와 `raw_ctrl_data` | `merge` 보류 | Picture/Shape는 decoded `common` 재생성이 source-of-truth이고 Table은 raw가 source-of-truth다. 전면 merge는 회귀 위험이 커서 후속 이슈에서 table serializer 정책 변경과 함께만 가능하다. |

### 15.6 remove_candidate

| 대상 | 판정 | 근거 |
|---|---|---|
| `Table.dirty` | `remove_candidate` | `src/model/table.rs:52`에 선언되고 생성 시 `dirty: true`가 보이지만 Stage 2 검색 기준 runtime 측정/렌더 캐시 invalidation의 source로 소비되는 근거가 약하다. 실제 제거 전 전체 검색과 테스트 필요. |
| `CharShapeMods`, `ParaShapeMods`의 위치 | `remove_candidate` 아님, `move_candidate` | 실제 formatting/header-footer/document command에서 쓰인다. 다만 저장 문서 IR이 아니라 command patch DTO이므로 `src/model/style.rs`에서 command/core DTO 영역으로 이동 후보. |
| `Document.hwpx_aux_entries` | 제거 후보 아님 | HWPX serializer가 원본 auxiliary entries를 우선 출력한다. Stage 2.2 초기 의심과 달리 보존 contract가 명확하다. |

### 15.7 followup_issue

| 대상 | 판정 | 후속 이슈 후보 |
|---|---|---|
| `UnknownControl` | `followup_issue` | 현재 모델은 `ctrl_id`만 보존한다. serializer도 header 수준만 재방출하므로 unknown control payload roundtrip 보존성 결함 가능성이 있다. |
| `Table.raw_ctrl_data` source-of-truth 정책 | `followup_issue` | table만 raw bytes가 serializer source라는 예외가 크다. `CommonObjAttr` writer를 table에도 적용할 수 있는지 별도 설계/회귀 테스트가 필요하다. |
| `raw_rendering`/`render_*` 우선순위 | `followup_issue` | 편집 시 raw clear, parse 시 raw 우선, renderer는 render 값을 쓰는 규칙이 흩어져 있다. source priority 문서와 helper/API 정리가 필요하다. |
| HWP3 layout policy | `followup_issue` | `Document.is_hwp3_variant`, `header.version.major == 3`, `pagination_bottom_tolerance`, HWP3 generated lineSeg가 여러 경로에 걸쳐 있다. `LayoutCompatibilityProfile` 도입 검토 대상. |
| `CharShapeMods`/`ParaShapeMods` 모듈 위치 | `followup_issue` | 모델 구조체와 command patch DTO가 `src/model/style.rs`에 섞여 있다. API 안정성을 고려해 이동/재export 여부를 별도 판단한다. |

### 15.8 Stage 3 결론

- 즉시 제거 가능하다고 판단되는 필드는 없다. `Table.dirty`만 제거 후보로 표시하되 실제
  코드 변경은 별도 이슈에서 전체 검색과 회귀 테스트 후 결정해야 한다.
- 핵심 정리 방향은 삭제보다 계층 분리다. 특히 `raw_ctrl_data`, `raw_header_extra`,
  `raw_rendering` 계열은 `source-preserve`와 `backend-materialized`가 섞여 있어 타입/헬퍼
  경계를 만드는 편이 안전하다.
- rhwp의 Document IR은 parser AST가 아니라 frontend lowering 결과와 backend contract가 함께
  들어간 컴파일러형 IR이다. 따라서 "순수 모델"로 되돌리기보다, IR 내부의 sub-layer를 명명하고
  우선순위 규칙을 문서화하는 쪽이 현실적인 1차 정리 방향이다.

## 16. SOLID 관점 개선 포인트

Document IR 감사 결과를 SOLID 관점으로 다시 보면, rhwp의 현재 구조는 기능적으로는 정합을
확보해 왔지만 책임이 `src/model`의 공통 구조체에 계속 응축되는 경향이 있다. 이 절은 코드
변경 지시가 아니라 후속 리팩터링 이슈를 만들 때의 설계 기준이다.

### 16.1 SRP — 단일 책임 원칙

| 영역 | 현재 책임 혼합 | 개선 방향 |
|---|---|---|
| `Document` | semantic document, HWP CFB extra stream, HWPX auxiliary entry, HWP3-origin flag가 공존 | container 보존 정보와 layout compatibility policy를 별도 sub-structure로 분리한다. |
| `Paragraph` | 텍스트/서식 semantic, HWP5 `PARA_HEADER` raw suffix, `CTRL_DATA` side channel, line segment layout input이 공존 | paragraph semantic, source-position map, HWP5 backend contract를 명명된 내부 구조로 나눈다. |
| `Table` | semantic table, layout cache, HWP5 raw CTRL_HEADER source-of-truth, edit dirty flag가 공존 | table semantic과 HWP5 backend materialization state를 분리한다. |
| `ShapeComponentAttr` | renderer affine source(`render_*`)와 serializer raw source(`raw_rendering`)가 공존 | transform semantic과 backend raw rendering payload의 우선순위 규칙을 타입으로 표현한다. |

SRP 관점의 1차 목표는 필드 삭제가 아니라 책임 이름 붙이기다. 예를 들어 `raw_*`를 모두
같은 범주로 보지 말고 `source_preserve`, `backend_materialized`, `layout_hint`로 나누는 것이
안전하다.

### 16.2 OCP — 개방 폐쇄 원칙

현재는 새 포맷/호환성 이슈가 생길 때 공통 IR 필드가 직접 늘어나는 경향이 있다.

| 사례 | OCP 위험 | 개선 방향 |
|---|---|---|
| HWPX verbatim 필드 증가 | HWPX roundtrip 요구가 공통 semantic 구조체를 계속 확장 | `HwpxPreserve` 또는 format-specific payload 영역으로 묶는다. |
| HWP3 조판 보정 | HWP3-origin 판정과 layout 보정이 renderer 전역 분기에 추가 | `LayoutCompatibilityProfile` 같은 policy object로 확장점을 만든다. |
| table serializer 예외 | Table만 `raw_ctrl_data` source-of-truth라는 예외가 명령/serializer에 퍼짐 | backend writer adapter를 통해 table 정책을 캡슐화한다. |

OCP 관점에서는 enum/struct에 필드를 계속 추가하는 방식보다, frontend lowering 단계가
format-specific payload를 붙이고 backend가 필요한 payload만 소비하는 구조가 바람직하다.

### 16.3 LSP — 리스코프 치환 원칙

Rust의 enum 구조라 전통적 OO 상속 문제는 아니지만, "같은 범주의 객체가 같은 계약으로
동작하는가"라는 관점에서는 LSP 문제가 보인다.

| 범주 | 치환성 문제 | 개선 방향 |
|---|---|---|
| `Table` vs `Picture/Shape` | Picture/Shape는 `common`이 source-of-truth인데 Table은 `raw_ctrl_data`가 source-of-truth | shape-like object의 위치/크기 source 계약을 통일하거나 Table 예외를 명시적 adapter로 격리한다. |
| `Control::Unknown` | 다른 control은 payload를 보존/직렬화하지만 unknown은 `ctrl_id`만 보존 | unknown control도 최소 raw payload를 보존하는 variant로 확장 검토가 필요하다. |
| HWP5/HWPX/HWP3 paragraph | 모두 `Paragraph`지만 lineSeg 존재 여부와 raw header 의미가 다름 | renderer/serializer가 사용하는 전제 조건을 `ParagraphContract` 문서 또는 validation pass로 명시한다. |

### 16.4 ISP — 인터페이스 분리 원칙

현재 소비자들은 큰 `Document`/`Paragraph` 구조체를 직접 본다. 그 결과 renderer가 몰라도 되는
raw preservation 필드와 serializer가 몰라도 되는 layout cache가 같은 타입에 노출된다.

| 소비자 | 필요한 view | 개선 방향 |
|---|---|---|
| renderer/layout | semantic content, styles, line/position map, layout policy | `LayoutDocumentView` 또는 layout input adapter를 둔다. |
| HWP5 serializer | semantic content + HWP5 backend contract/raw preserve | `Hwp5SerializeView` 또는 backend contract helper를 둔다. |
| HWPX serializer | semantic content + HWPX verbatim preserve | `HwpxSerializeView` 또는 HWPX preserve wrapper를 둔다. |
| document_core commands | semantic edit API + invalidation/materialization API | raw byte 직접 조작 대신 command-level helper를 둔다. |

ISP 관점의 핵심은 구조체를 당장 쪼개는 것이 아니라, 소비자가 필요한 slice만 보도록 helper/view
계층을 만드는 것이다.

### 16.5 DIP — 의존성 역전 원칙

상위 계층인 document_core 명령이 serializer byte layout 상수나 raw byte offset을 직접 아는
지점이 있다. 예를 들어 table 이동/크기 조정은 `raw_ctrl_data` offset을 직접 patch한다.

| 현재 의존 | DIP 위험 | 개선 방향 |
|---|---|---|
| `document_core/commands/table_ops.rs` → `common_obj_offsets` byte layout | 편집 명령이 HWP5 backend detail에 직접 의존 | `TableBackendContract` helper가 semantic 변경을 raw/materialized state에 반영하게 한다. |
| object command의 `raw_rendering.clear()` 패턴 | 편집 명령이 serializer 우선순위 규칙을 암묵적으로 알고 있음 | `ShapeTransform::set_edited_transform()` 같은 helper로 raw 무효화와 render 값 갱신을 묶는다. |
| DocInfo surgical update 호출 | command가 serializer 내부 수술 API를 직접 호출 | `DocInfoPreservation` service/helper로 캡슐화한다. |

DIP 관점의 후속 목표는 "문서 편집 계층이 HWP5 byte layout을 몰라도 되는 구조"다. 낮은 수준의
backend detail은 serializer/backend contract helper가 소유하고, document_core는 semantic intent를
전달하는 형태가 좋다.

### 16.6 SOLID 기반 후속 이슈 후보

| 후보 | 연결 원칙 | 우선도 | 메모 |
|---|---|---|---|
| Table backend contract helper 도입 | SRP, LSP, DIP | 높음 | `raw_ctrl_data` 직접 patch와 `common` dual maintenance를 helper로 통합한다. |
| Shape transform source priority helper 도입 | SRP, DIP | 중간 | `raw_rendering` clear + `render_*` 갱신 패턴을 한 곳에 모은다. |
| HWPX preserve wrapper 도입 | SRP, OCP, ISP | 중간 | `hwpx_head_tail`, `raw_para_heads`, `raw_parameters_xml`, `img_dim`, `vertical_all`의 HWPX-only 의미를 묶는다. |
| Layout compatibility profile 도입 | OCP, ISP | 중간 | HWP3-origin layout 보정 플래그를 renderer policy object로 분리한다. |
| UnknownControl raw payload 보존 | LSP | 높음 | unknown control도 다른 control처럼 payload roundtrip 보존 계약을 가져야 한다. |
| `CharShapeMods`/`ParaShapeMods` command DTO 위치 정리 | SRP, ISP | 낮음 | 실제 동작 위험은 낮지만 model 모듈의 책임을 흐린다. |
