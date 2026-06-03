# Task 233 최종 결과 보고서: 양식 개체 상호작용 및 데이터 바인딩

## 개요

5종 양식 개체(명령 단추, 선택 상자, 콤보 상자, 라디오 단추, 편집 상자)에 대한 클릭 상호작용, 값 조회/설정 WASM API, ComboBox 스크립트 항목 추출을 구현하였다.

## 완료 항목

### WASM API (Rust)
- `getFormObjectAt(pageNum, x, y)` — 렌더 트리 좌표 충돌 검사로 양식 개체 감지
- `getFormValue(sec, para, ci)` — 양식 개체 값 조회
- `setFormValue(sec, para, ci, valueJson)` — 값 설정 + recompose_section + 캐시 무효화
- `getFormObjectInfo(sec, para, ci)` — 상세 정보 + ComboBox 항목 목록

### 프론트엔드 상호작용 (TypeScript)
- **CheckBox**: 클릭 시 value 0↔1 토글 → 리렌더링
- **RadioButton**: 클릭 시 GroupName 기반 동일 그룹 해제 → 선택 → 리렌더링
- **ComboBox**: 클릭 시 커스텀 HTML 드롭다운 표시 → 항목 선택 → 리렌더링
- **Edit**: 클릭 시 HTML input 오버레이 → Enter/blur로 확정 → 리렌더링
- **PushButton**: 웹 환경 보안상 비활성 (회색 렌더링, 클릭 무시)

### ComboBox 스크립트 항목 추출
- `Scripts/DefaultJScript` OLE 스트림에서 zlib 해제 + UTF-16LE 디코딩
- `컨트롤이름.InsertString("항목", 인덱스)` 패턴 매칭으로 항목 목록 추출
- `getFormObjectInfo` API 응답의 `items` 배열로 프론트엔드에 전달

### 기술 문서
- `mydocs/tech/hwp_form_object_api.md` — API 레퍼런스, 사용 예시, 스크립트 파싱 원리

## 변경 파일 목록

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/render_tree.rs` | FormObjectNode에 section_index, para_index, control_index, name 추가 |
| `src/renderer/layout/paragraph_layout.rs` | FormObjectNode 생성 시 위치 정보 전달 (2곳) |
| `src/document_core/queries/form_query.rs` | 신규: 4개 네이티브 API + 스크립트 파싱 |
| `src/document_core/queries/mod.rs` | form_query 모듈 등록 |
| `src/wasm_api.rs` | 4개 wasm_bindgen API 추가 |
| `src/renderer/svg.rs` | PushButton 비활성 스타일 |
| `src/renderer/web_canvas.rs` | PushButton 비활성 스타일 |
| `rhwp-studio/src/core/types.ts` | FormObjectHitResult, FormValueResult, FormObjectInfoResult 인터페이스 |
| `rhwp-studio/src/core/wasm-bridge.ts` | 4개 API 래퍼 + typeof 방어 코드 |
| `rhwp-studio/src/engine/input-handler-mouse.ts` | onClick에서 양식 개체 클릭 감지 |
| `rhwp-studio/src/engine/input-handler.ts` | handleFormObjectClick, 라디오 그룹 처리, ComboBox/Edit 오버레이 |
| `rhwp-studio/src/styles/form-overlay.css` | 신규: 드롭다운/입력 오버레이 스타일 |
| `rhwp-studio/src/style.css` | form-overlay.css import 추가 |

## 검증 결과
- `cargo test`: 716개 통과, 0개 실패
- WASM 빌드: 성공
- 브라우저 테스트: 5종 양식 개체 상호작용 정상 동작 확인
