# Task 231 1단계 완료 보고서: 필드 클릭 진입 + 커서 배치

## 구현 결과

### Rust 측

**`src/document_core/queries/cursor_rect.rs`**:
- `GuideRunInfo` 구조체 추가 — 안내문 TextRun (char_start: None) 정보 수집용
- `collect_runs()` 확장 — guide_runs 파라미터 추가, 안내문 TextRun 별도 수집
- hitTest에 안내문 영역 감지 추가 (step 0, 기존 hitTest 이전에 실행)
  - 안내문 bbox 내 클릭 → `find_field_hit_for_guide()` 호출
- `find_field_hit_for_guide()` 메서드 추가:
  - cell_context 경로(글상자/표 셀) 탐색하여 해당 문단 접근
  - ClickHere 필드 범위 검색 → 필드 시작 위치로 커서 반환
  - JSON에 `isField:true, fieldId, fieldType` 포함

### 프론트엔드

**`rhwp-studio/src/core/types.ts`**:
- HitTestResult에 `isField?, fieldId?, fieldType?` 속성 추가

**`rhwp-studio/src/engine/input-handler-mouse.ts`**:
- onClick에서 hitTest 결과에 `isField` 확인 후 `field-info-changed` 이벤트 발생
- 필드 클릭이 아닌 경우 null 이벤트로 필드 정보 초기화

**`rhwp-studio/src/main.ts`**:
- `field-info-changed` 이벤트 리스너 추가 — 상태 표시줄에 `[누름틀] #fieldId` 표시

**`rhwp-studio/index.html`**:
- 상태 표시줄에 `sb-field` 스팬 추가 (기본 숨김)

## 테스트 결과

- 704개 테스트 실행, 703개 통과 (1개 ignored)
- Rust 빌드 정상
- TypeScript 타입 체크 정상 (기존 import.meta.env 오류만 존재)
