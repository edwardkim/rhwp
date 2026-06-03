# Task 264 수행 계획서: 문단번호/글머리표 대화상자 탭 통합

## 현재 상태

- 문단 번호 대화상자: 단일 탭 (문단 번호만)
- 글머리표 적용: 도구 상자 팝업으로만 가능
- 컨텍스트 메뉴 "문단 번호 모양" → 문단 번호만 설정 가능
- 글머리표 프리셋: 22종 (체크박스 4종 포함)

## 구현 계획

### 1단계: 대화상자 탭 구조 확장
- 제목 변경: "문단 번호 모양" → "문단 번호/글머리표"
- 탭 바 추가: "문단 번호" / "글머리표" 탭 전환
- 현재 headType에 따라 초기 탭 자동 선택

### 2단계: 글머리표 탭 구현
- 18종 글머리표 프리셋 그리드 (한컴 동일)
- "(없음)" 선택 옵션
- displayChar로 브라우저 표시 (PUA→Unicode 매핑)
- 현재 글머리표 문자 선택 상태 표시 (rawCode 기반 매칭)

### 3단계: 프리셋 정리
- 체크박스 글머리표 4종 제거 (한컴에 없음)
- displayChar 수정: PUA 매핑과 불일치하는 아이콘 수정 (★, ☞)
- BulletPreset 인터페이스 단순화 (isCheckbox/checkedChar 제거)

### 4단계: 콜백 연결
- onApplyBullet 콜백 추가 → applyBullet(bulletChar) 호출
- getBulletList에 rawCode 필드 추가 (PUA 원본 코드)
- Bullet 문단에서 대화상자 열 때 현재 bullet 매칭

## 참조 파일

| 파일 | 역할 |
|------|------|
| rhwp-studio/src/ui/numbering-dialog.ts | 대화상자 (탭 구조 + 글머리표 패널) |
| rhwp-studio/src/core/numbering-defaults.ts | 글머리표 프리셋 정의 |
| rhwp-studio/src/command/commands/format.ts | 커맨드 연결 (onApplyBullet) |
| rhwp-studio/src/core/wasm-bridge.ts | getBulletList 타입 |
| rhwp-studio/src/styles/numbering-dialog.css | 탭/그리드 CSS |
| src/wasm_api.rs | getBulletList rawCode 추가 |
