# 타스크 169 단계별 완료 보고서: 글자모양 서식바 강화

## 1단계: Rust 측 emboss/engrave + font_ids 보완

### 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/model/style.rs` | CharShapeMods에 `emboss`, `engrave`, `font_ids` 필드 추가 + `apply_to()` 로직 |
| `src/document_core/helpers.rs` | `parse_char_shape_mods`에 emboss/engrave/fontIds 파싱 + `json_u16_array()` 헬퍼 |
| `src/document_core/commands/formatting.rs` | `build_char_properties_json`에 emboss/engrave 키 + `find_or_create_font_id_for_lang()` |
| `src/wasm_api.rs` | `findOrCreateFontIdForLang` WASM API |

### 주요 구현 사항

- **emboss/engrave 상호 배타**: 양각 활성화 시 음각 자동 해제, 역도 마찬가지
- **font_ids[7]**: 7개 언어 카테고리(한글/영문/한자/일어/기타/기호/사용자) 별 개별 글꼴 ID 지원
- **find_or_create_font_id_for_lang**: 특정 언어 카테고리의 font_faces에서 글꼴 검색/등록

### 검증

- `cargo test`: 613개 통과

---

## 2단계: 서식바 5개 버튼 추가 (HTML + CSS + JS)

### 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `rhwp-studio/index.html` | 양각/음각/외곽선/위첨자/아래첨자 5개 버튼 HTML |
| `rhwp-studio/src/styles/style-bar.css` | 5개 버튼 아이콘 CSS (.sb-emboss, .sb-engrave, .sb-outline, .sb-sup, .sb-sub) |
| `rhwp-studio/src/core/types.ts` | CharProperties에 `emboss`, `engrave`, `fontIds` 필드 추가 |
| `rhwp-studio/src/command/commands/format.ts` | 5개 커맨드 등록 (format:emboss/engrave/outline/superscript/subscript) |
| `rhwp-studio/src/engine/input-handler.ts` | `toggleFormat` 타입 확장 + `applyToggleFormat` 상호 배타 로직 |
| `rhwp-studio/src/ui/toolbar.ts` | 5개 버튼 참조/이벤트 + `updateState` active 토글 |

### 주요 구현 사항

- **상호 배타 토글**: emboss↔engrave, superscript↔subscript
- **outline 토글**: outlineType 0↔1 전환
- **커서 이동 시 자동 반영**: 5개 속성 모두 active 클래스 토글

### 검증

- Studio 빌드: 성공

---

## 3단계: 언어별 글꼴 선택

### 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `rhwp-studio/index.html` | 글꼴 언어 카테고리 콤보 (전체/한글/영문) |
| `rhwp-studio/src/styles/style-bar.css` | `.sb-font-lang` 스타일 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `findOrCreateFontIdForLang` 래퍼 |
| `rhwp-studio/src/ui/toolbar.ts` | 언어별 글꼴 변경 로직 + 커서 이동 시 선택 언어 글꼴명 표시 |

### 주요 구현 사항

- **"전체" 모드**: 기존대로 `fontId` 단일값으로 전체 언어 일괄 적용
- **"한글"(0) / "영문"(1) 모드**: `fontIds[7]` 배열 중 해당 인덱스만 교체
- **커서 이동 시**: 선택된 언어 카테고리의 글꼴명을 드롭다운에 표시
- **언어 콤보 변경 시**: 마지막 fontFamilies 배열에서 해당 언어 글꼴명을 즉시 표시

### 검증

- WASM 빌드: 성공
- Studio 빌드: 성공
- `cargo test`: 613개 통과

---

*작성일: 2026-02-27*
