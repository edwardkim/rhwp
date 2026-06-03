# Stage 1 단계별 완료보고서: header.xml IR 기반 동적 생성

- **타스크**: [#182](https://github.com/edwardkim/rhwp/issues/182)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task182`
- **일자**: 2026-04-17
- **단계**: Stage 1 / 5 — header.xml IR 기반 동적 생성

## 1. 범위 재확정 (작업지시자 지시에 따라)

**1단계 목표**: 기존 HWPX 문서를 parse → (편집) → serialize 했을 때 한컴2020이 온전히 다시 연다.

**1단계 범위 제외**: 완전히 새 빈 문서를 `Document::default()`에서 생성하여 저장하는 기능.

이 제약 덕분에 "IR에 없는 속성은 한컴 기본값으로 채운다"는 별도 로직(`default_doc_info_resources()`)이 본 단계에서 불필요함을 확정.

## 2. 산출물

### 2.1 수정 파일

**`src/serializer/hwpx/header.rs`** (13줄 → 약 760줄)

기존 `include_str!("templates/empty_header.xml")` 정적 반환 제거. `write_header(doc, ctx)` 로 교체하여 DocInfo IR 전체를 동적으로 XML로 직렬화.

구성:
- `write_header` — 루트 `<hh:head>` + 14개 네임스페이스 선언 + `version="1.2"` + `secCnt`
- `write_begin_num` — DocProperties → `<hh:beginNum page=... tbl=... ...>`
- `write_fontfaces` — 7 언어 그룹(HANGUL..USER) → `<hh:fontfaces itemCnt>` + `<hh:fontface lang fontCnt>` + `<hh:font id face type isEmbedded>`
- `write_border_fills` — `<hh:borderFill>` + 자식 8종 (slash, backSlash, left/right/top/bottomBorder, diagonal, fillBrush)
- `write_char_properties` — `<hh:charPr>` + 자식 15종 (fontRef, ratio, spacing, relSz, offset, italic, bold, underline, strikeout, outline, shadow, emboss, engrave, supscript, subscript)
- `write_tab_properties` — `<hh:tabPr>` + `<hh:tabItem>`
- `write_numberings` — `<hh:numbering>` + 10 레벨 `<hh:paraHead>`
- `write_para_properties` — `<hh:paraPr>` + 자식 7종 (align, heading, breakSetting, margin/children, lineSpacing, border, autoSpacing)
- `write_styles` — `<hh:style>` (PARA/CHAR)
- `write_compatible_document` + `write_doc_option` + `write_track_change_config` — 정적 마감 섹션

**`src/serializer/hwpx/mod.rs`**: `header::write_header(doc)` → `header::write_header(doc, &ctx)`

### 2.2 신규 테스트

단위 테스트 (`header.rs` 내부):
- `write_header_runs_on_empty_document` — 빈 Document 입력 시 정상 XML 생성
- `write_header_preserves_char_shape_count` — IR `char_shapes.len()` == 출력 `<hh:charPr>` 개수
- `write_header_emits_seven_fontfaces_when_populated` — ref_empty 입력 시 7개 fontface 출력
- `canonical_attr_order_charpr` — `<hh:charPr>` 속성 순서가 한컴 OWPML 기준(id → height → textColor → shadeColor → useFontSpace → useKerning → symMark → borderFillIDRef)과 일치

통합 테스트 (`tests/hwpx_roundtrip_integration.rs`):
- `stage1_ref_empty_roundtrip` — IrDiff 0
- `stage1_ref_text_roundtrip` — IrDiff 0
- `stage1_ref_mixed_header_level_regression_probe` — IrDiff 0 (현재 뼈대 비교 수준)

## 3. Canonical 속성·자식 순서 준수

한컴 OWPML 공식 프로젝트(hancom-io/hwpx-owpml-model, Apache 2.0) 기준:

| 클래스 | 속성 순서 출처 | 자식 순서 출처 |
|---|---|---|
| charPr | CharShapeType.cpp:79-86 | CharShapeType.cpp:59-73 |
| paraPr | ParaShapeType.cpp:62-68 | ParaShapeType.cpp:50-56 |
| borderFill | BorderFillType.cpp:64-68 | BorderFillType.cpp:51-58 |
| tbl/pic | TableType.cpp / PictureType.cpp (Stage 3/4에서 적용) | |

`canonical_defaults.rs`의 상수와 함께, 각 writer 함수의 배열 순서로 강제.

## 4. 검증 결과

### 4.1 단위 테스트

```
serializer::hwpx 관련: 29 passed, 0 failed
- canonical_defaults::tests: 5 ✅
- context::tests: 5 ✅
- fixtures::tests: 2 ✅
- roundtrip::tests: 3 ✅
- header::tests: 4 ✅ (신규)
- mod::tests (기존): 11 ✅
```

### 4.2 통합 테스트

```
running 4 tests
test stage0_blank_hwpx_roundtrip ... ok
test stage1_ref_empty_roundtrip ... ok
test stage1_ref_text_roundtrip ... ok
test stage1_ref_mixed_header_level_regression_probe ... ok

test result: ok. 4 passed; 0 failed
```

### 4.3 전체 라이브러리

**822 passed, 0 failed, 1 ignored** — 회귀 없음.

## 5. 완료 기준 대조

수행계획서 Stage 1 완료 기준:

| 기준 | 상태 | 근거 |
|---|---|---|
| Stage 0 하네스 유지 | ✅ | `stage0_blank_hwpx_roundtrip` 계속 통과 |
| `blank_hwpx.hwpx` + `ref_empty.hwpx` IrDiff 0 | ✅ | 통합 테스트 통과 |
| `char_shapes.len() == N` → `<hh:charPr>` 정확히 N개 | ✅ | `write_header_preserves_char_shape_count` |
| 단위 테스트: canonical 속성 순서 일치 | ✅ | `canonical_attr_order_charpr` |
| charPrIDRef/paraPrIDRef/borderFillIDRef/tabPrIDRef/fontRef/styleID 전 참조 resolve | ✅ | `assert_all_refs_resolved()` 모든 테스트 통과 |
| `default_doc_info_resources()` 구현 | ❌ (범위 외) | 작업지시자 확인: 1단계에서 제외 |

## 6. 주요 설계 결정

### 6.1 "IR에 없으면 뼈대값, IR에 있으면 그대로" 원칙

IR에 없는 필드(예: BreakSetting의 widowOrphan)는 정적 기본값을 출력하되, IR에 있는 값은 그대로 반영. 1단계 범위의 "기존 문서 수정 후 저장" 시나리오에서는 IR에 모든 값이 파싱돼 있으므로 거의 모든 값이 IR 기반으로 출력됨.

### 6.2 네임스페이스 전수 선언

한컴 샘플은 `<hh:head>` 루트에 14개 네임스페이스를 모두 선언. 파서가 어떤 네임스페이스든 해석할 수 있도록 rhwp도 전수 선언으로 맞춤. 누락 시 한컴2020이 알 수 없는 네임스페이스에서 오류 가능.

### 6.3 BorderFill id 1-based 변환

한컴 샘플 관찰 결과 `<hh:borderFill id="1">`, `<hh:borderFill id="2">` 로 **1-based** id 사용. rhwp IR은 0-based 배열 인덱스지만 XML에선 `id + 1` 로 출력. 파서는 이미 1-based를 0-based로 역변환.

### 6.4 `section0.xml`에서 참조하는 paraPrIDRef/styleIDRef

Stage 1에선 `write_header`만 동적화되고 section.xml은 기존 템플릿 치환 경로를 유지. 그러나 template이 `paraPrIDRef="0"`을 쓰므로 header.xml에 `<hh:paraPr id="0">` 이 존재해야 함 — ref_empty.hwpx 는 이미 IR에 20개 paraShape가 있어 자동 만족.

**경고**: `Document::default()` 로 새 문서를 만들면 header는 비어있지만 section.xml 템플릿은 `paraPrIDRef="0"` 참조 → `assert_all_refs_resolved()` 가 잡아낼 것. 이는 Stage 2에서 section도 동적화하며 자연스럽게 해소됨.

### 6.5 정적 섹션(compatibleDocument, docOption, trackchageConfig)

IR 매핑이 없는 마감 섹션은 한컴 샘플 관찰값을 그대로 정적 출력. 2/3단계에서 필요 시 IR 확장으로 동적화 가능.

## 7. 알려진 제한

1. **Font.alt_type 추측**: IR `Font.alt_type` 이 0인 경우 "TTF"로 기본 치환. 실 샘플에서 0으로 파싱되는 경우가 있어 안전한 추측.
2. **shadeColor "none" 처리**: text color는 `#000000` 인데 shade는 `none` 으로 써야 함 (한컴 관찰). `shade_color == 0`일 때 "none" 출력.
3. **numbering paraHead 일부 속성 정적**: `numFormat="DIGIT"`, `charPrIDRef="4294967295"` 등 10 레벨을 루프로 찍되 `width_adjust` 외엔 정적값. Stage 2/3에서 NumberingHead의 추가 필드를 검사하며 확장 예정.
4. **fillBrush 빈 래퍼**: borderFill의 Fill이 없을 때는 생략, 있을 때는 빈 `<hc:fillBrush></hc:fillBrush>` 만 출력. 실제 그러데이션·패턴 등은 Stage 2+에서 확장.

## 8. 다음 단계 (Stage 2)

**Stage 2 — section.xml 동적화 + charPrIDRef 매핑**:

- `section.rs` 의 `empty_section0.xml` 템플릿 치환 제거
- `SectionWriter`가 `<hs:sec>` 루트부터 빌드
- `secPr/pagePr/grid/startNum/visibility/footNotePr/endNotePr/pageBorderFill`를 `Section.section_def`/`PageDef`에서 매핑
- `paragraph.rs` 신규: `write_paragraph(w, para, doc, ctx, pi, vert_cursor)` + `ParagraphChild` enum
- `Paragraph.char_shapes`(UTF-16 범위별 id) → `<hp:run charPrIDRef>` 분할

완료 기준:
- 기존 문단/탭/줄바꿈 테스트 유지
- `ref_text.hwpx` IrDiff 0 (현재는 뼈대 비교 수준)
- 다중 run 문서: run 개수·charPrIDRef 값 원본 일치

## 9. 승인 요청

본 Stage 1 완료보고서 검토 후 승인 시 Stage 2 착수. 피드백 요청 시 `mydocs/feedback/` 에 등록 부탁드립니다.
