# Task 312 구현계획서: 줄간격 조절 단축키 구현

## 1. 기능
- Alt+Shift+A: 줄간격 10% 줄이기
- Alt+Shift+Z: 줄간격 10% 늘리기
- 블록 선택 시 선택된 문단들에 일괄 적용

## 2. 기존 인프라
- `setLineSpacing(value)`: 이미 구현됨 (`input-handler.ts:2097`)
- `applyParaFormat()`: 이미 구현됨 (선택 범위, 셀/머리말 모드 지원)
- `getParaProperties()`: 현재 줄간격 조회 가능
- Rust WASM: `applyParaFormat` → `parse_para_shape_mods` → `find_or_create_para_shape`

## 3. 구현 계획

### 3.1 단계 1: 단축키 등록
- `shortcut-map.ts`에 Alt+Shift+A, Alt+Shift+Z 추가

### 3.2 단계 2: 커맨드 핸들러
- `format.ts`에 `format:line-spacing-decrease`, `format:line-spacing-increase` 추가
- 현재 줄간격 조회 → ±10% 계산 → `setLineSpacing()` 호출
- 최소 50%, 최대 500% 제한

### 3.3 단계 3: 테스트
- 웹에서 Alt+Shift+A/Z 동작 확인

## 4. 영향 범위
- `rhwp-studio/src/command/shortcut-map.ts`
- `rhwp-studio/src/command/commands/format.ts`
