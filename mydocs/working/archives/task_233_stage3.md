# Task 233 3단계 완료 보고서: ComboBox 드롭다운 및 Edit 입력 오버레이

## 완료 내용

### ComboBox 드롭다운 오버레이
- `showComboBoxOverlay()`: 양식 개체 bbox 위치에 HTML `<select>` 요소 오버레이
- `getFormObjectInfo`로 properties에서 `ItemCount`, `Item0`, `Item1`, ... 항목 목록 추출
- 항목 선택 시 `setFormValue` → 리렌더링
- 포커스 이탈 시 자동 제거
- 드롭다운 자동 열기 (`requestAnimationFrame` → `focus` → `click`)

### Edit 입력 오버레이
- `showEditOverlay()`: bbox 위치에 HTML `<input>` 요소 오버레이
- Enter 키 → 값 확정 + 리렌더링
- Escape 키 → 취소 (값 미변경)
- blur 이벤트 → 값 확정 + 리렌더링
- 파란색 테두리 포커스 표시

### 공통 인프라
- `formOverlay` 상태 변수: 현재 활성 오버레이 추적
- `removeFormOverlay()`: 기존 오버레이 정리
- `formBboxToOverlayRect()`: 페이지 좌표 → scroll-content 내 절대 좌표 변환 (줌 반영)
  - `virtualScroll.getPageOffset()` + `getPageLeft()` 활용 (캐럿 위치 계산과 동일 패턴)

### 변경 파일
- `rhwp-studio/src/engine/input-handler.ts`: formOverlay 상태 변수 + 4개 메서드 추가 (`formBboxToOverlayRect`, `removeFormOverlay`, `showComboBoxOverlay`, `showEditOverlay`)

## 검증 결과
- TypeScript 컴파일: 새 에러 없음
