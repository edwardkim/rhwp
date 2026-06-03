# Task 298 수행계획서: 인라인 Shape 커서 글자 단위 이동 구현

## 1. 현황 분석

### 1.1 문제 정의
`treat_as_char=true`로 설정된 인라인 Shape(사각형, 원 등 단순 도형)가 문단 텍스트 사이에 배치될 때, 커서가 해당 Shape를 글자 한 개처럼 인식하여 좌우 이동해야 하지만 현재 구현되어 있지 않다.

### 1.2 현재 동작
- 인라인 Shape는 렌더링만 되고, 커서 이동 시 Shape 위치를 건너뛰거나 인식하지 못함
- hitTest에서 Shape 영역 클릭 시 정확한 char_offset으로 매핑되지 않음

### 1.3 기대 동작 (한컴 워드 기준)
- `[사각형] [Set ID] ScrollPosInfo` 문단에서:
  - 커서가 사각형 앞 → 오른쪽 화살표 → 사각형 뒤(= 공백 앞)
  - 사각형은 1글자 폭으로 커서가 통과
  - Shift+Arrow로 사각형을 선택 범위에 포함 가능

## 2. 기술 분석

### 2.1 기존 인프라
| 구성요소 | 파일 | 상태 |
|----------|------|------|
| 컨트롤 위치 복원 | `helpers.rs` → `find_control_text_positions()` | char_offsets 갭 기반 위치 복원 완료 |
| 네비게이션 텍스트 길이 | `helpers.rs` → `navigable_text_len()` | 인라인 컨트롤 위치 반영 완료 |
| 좌우 이동 | `doc_tree_nav.rs` → `navigate_next_editable()` | TextBox/Table 진입은 구현, 단순 Shape 통과 미구현 |
| hitTest | `cursor_rect.rs` → `hit_test_native()` | TextRun 기반 매칭만 — Shape 영역 미지원 |
| 인라인 Shape 좌표 | `render_tree.rs` → `inline_shape_positions` | (sec, para, ctrl) → (x, y) 저장 완료 |

### 2.2 핵심 과제
1. **navigate_next_editable()**: 단순 도형(TextBox 없는 Shape)을 만나면 1칸 건너뛰기 (Table/TextBox 진입과 구분)
2. **hitTest**: 인라인 Shape BoundingBox 내 클릭 → 해당 char_offset 반환
3. **커서 렌더링**: Shape 위치에 커서가 있을 때 Shape 왼쪽/오른쪽에 캐럿 표시

## 3. 구현 계획

### 3.1 단계 1: navigate_next_editable()에서 단순 Shape 통과 처리
- `doc_tree_nav.rs`에서 컨트롤 위치에 Shape가 있을 때:
  - TextBox가 있는 Shape → 기존 로직(진입)
  - TextBox가 없는 Shape → Picture/Equation처럼 1칸 건너뛰기
- 역방향(delta < 0)도 동일 처리

### 3.2 단계 2: hitTest에서 인라인 Shape 영역 클릭 지원
- `hit_test_native()`에서 `inline_shape_positions`를 조회하여 Shape BoundingBox 내 클릭 감지
- Shape 왼쪽 절반 클릭 → Shape 앞 char_offset, 오른쪽 절반 → Shape 뒤 char_offset

### 3.3 단계 3: 커서 렌더링 보정
- `cursor_rect_native()`에서 char_offset이 Shape 위치일 때 Shape 좌표 기준 캐럿 위치 반환
- Shape 높이에 맞춘 캐럿 높이 조정

## 4. 영향 범위
- `src/document_core/queries/doc_tree_nav.rs` — 좌우 이동 로직
- `src/document_core/queries/cursor_rect.rs` — hitTest + 커서 위치 계산
- `src/document_core/helpers.rs` — 필요 시 유틸 함수 추가

## 5. 테스트 계획
- `samples/inline-bug-01.hwp`로 좌우 이동 동작 확인
- 기존 글상자(TextBox) 인라인 Shape 진입 동작 회귀 확인
- 인라인 이미지/수식 통과 동작 회귀 확인

## 6. 리스크
- 인라인 Shape가 여러 개인 문단에서 char_offset 매핑 정확도
- TextBox 유무 판별 시 Group Shape 내부 검사 필요 여부
