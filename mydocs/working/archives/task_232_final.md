# Task 232 최종 결과 보고서: 양식 개체 파싱 및 렌더링

## 개요

HWPTAG_FORM_OBJECT 파싱 및 5종 양식 개체(명령 단추, 체크 박스, 콤보 상자, 라디오 버튼, 편집 상자)의 시각 렌더링을 SVG/Canvas로 구현하였다.

## 완료 단계

### 1단계: 모델 정의 및 파서 구현
- `FormType` enum (5종), `FormObject` 구조체, `Control::Form` 변형 추가
- `parse_form_control()`: ctrl_data에서 width/height 추출 + HWPTAG_FORM_OBJECT 바이너리 파싱
- 속성 문자열 파서: 공백 구분 속성, 콜론 구분 key:type:value 포맷 해석
- `apply_form_property()`: Name, Caption, Text, ForeColor, BackColor, Value, Enabled 매핑

### 2단계: 렌더 트리 + 레이아웃 배치
- `FormObjectNode` 구조체, `RenderNodeType::FormObject` 변형 추가
- paragraph_layout에서 TAC 인라인 배치 (run 내부 + 빈 문단 모두 처리)
- composer에서 `Control::Form` → tac_controls 등록

### 3단계: SVG 렌더링
- 5종 양식 개체별 SVG 출력 구현
  - PushButton: 회색 배경 + 3D 테두리 + 캡션 중앙 정렬
  - CheckBox: □/☑ + 캡션 우측 배치
  - RadioButton: ○/◉ + 캡션 우측 배치
  - ComboBox: 입력 영역 + ▼ 드롭다운 버튼
  - Edit: 테두리 사각형 + 내부 텍스트
- 빈 문단(text_len=0) 양식 개체 배치 문제 해결

### 4단계: Canvas 렌더링 + 마무리
- Canvas 2D API로 동일한 5종 양식 개체 렌더링 구현
- HTML 렌더러에 `[양식]` placeholder 추가
- `Control::Form` match exhaustiveness 전수 대응
- WASM 빌드 및 브라우저 렌더링 검증 완료

## 변경 파일 목록

| 파일 | 변경 내용 |
|------|-----------|
| `src/model/control.rs` | FormType enum, FormObject 구조체, Control::Form 변형 |
| `src/parser/tags.rs` | CTRL_FORM 상수 |
| `src/parser/control.rs` | parse_form_control, decode_utf16le, parse_form_properties, apply_form_property |
| `src/renderer/render_tree.rs` | FormObjectNode, RenderNodeType::FormObject |
| `src/renderer/composer.rs` | Control::Form → tac_controls 등록 |
| `src/renderer/layout/paragraph_layout.rs` | TAC 인라인 배치 + 빈 문단 처리 + form_color_to_css |
| `src/renderer/svg.rs` | render_form_object (5종 SVG 렌더링) |
| `src/renderer/web_canvas.rs` | render_form_object (5종 Canvas 렌더링) |
| `src/renderer/html.rs` | FormObject → [양식] placeholder |
| `src/serializer/body_text.rs` | Control::Form char code 매핑 |
| `src/serializer/control.rs` | Control::Form 직렬화 arm |
| `src/main.rs` | Control::Form dump 출력 |
| `src/parser/control/tests.rs` | Control::Form match arm (2곳) |
| `src/wasm_api/tests.rs` | Control::Form match arm (3곳) |

## 검증 결과

- `cargo test`: 전체 통과
- `samples/form-01.hwp` SVG 내보내기: 5종 양식 개체 정상 렌더링
- WASM 빌드 + 브라우저 렌더링: 정상 확인
