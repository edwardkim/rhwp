# Task 231 3단계 완료 보고서: F11 블록 선택 + 상태 표시줄

## 구현 결과

### F11 필드 블록 선택

**`rhwp-studio/src/engine/input-handler-keyboard.ts`**:
- `default` case에 F11 키 처리 추가
- 커서가 필드 내에 있으면 `getFieldInfoAt()`으로 범위 조회
- `startCharIdx` → `endCharIdx` 전체 블록 선택 (anchor + moveTo)
- 필드 밖이면 무시

### 상태 표시줄 필드 정보 강화

**`rhwp-studio/src/main.ts`**:
- `field-info-changed` 이벤트에 `guideName` 포함
- 표시 형식: `[누름틀] {안내문}` (안내문 있으면) 또는 `[누름틀] #{fieldId}` (없으면)

**`rhwp-studio/src/engine/input-handler-mouse.ts`**:
- 필드 클릭 시 `getFieldInfoAt()`으로 `guideName` 조회 후 이벤트에 포함

## 테스트 결과

- 703개 테스트 전체 통과
- Rust/TypeScript 빌드 정상
