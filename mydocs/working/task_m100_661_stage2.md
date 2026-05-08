# Task M100 #661 Stage 2 완료 보고서

## 개요

- 이슈: #661 `rhwp-studio: 텍스트 드래그 선택 중 커서와 스크롤 위치가 튀는 현상`
- 기준 브랜치: `local/task661`
- 기반 코드: `upstream/pr/664` (`b1b18c2`) 위에 Stage 1 커밋 적용
- 단계 목표: 드래그 선택 중 caret 갱신 경로에서 viewport 스크롤 부작용 제거

## 변경 내용

### 1. 드래그 전용 caret 갱신에서 자동 스크롤 제거

- 파일: `rhwp-studio/src/engine/input-handler.ts`
- 함수: `updateCaretDuringDrag()`
- `this.scrollCaretIntoView(rect)` 호출을 제거했다.
- 드래그 중에는 caret rect 기준 스크롤을 하지 않고, 다음 단계에서 포인터 edge 기준 자동 스크롤만 별도 경로로 다룬다는 의도 주석을 남겼다.

## 검증

```bash
cd rhwp-studio
npm run build
```

- 결과: 성공
- 비고: Vite의 기존 chunk size 경고는 발생했지만 빌드 실패는 없었다.

## 영향 범위

- 일반 키보드 입력, 클릭 이동, 조합 입력 경로의 `updateCaret()` 및 `scrollCaretIntoView()` 동작은 변경하지 않았다.
- `updateCaretDuringDrag()` 경로만 변경했으므로 드래그 중 선택 하이라이트/라이브 caret 갱신 흐름은 유지된다.

## 남은 작업

1. Stage 3에서 드래그 포인터가 편집 영역 edge를 벗어날 때만 자동 스크롤되는 경로를 추가한다.
2. 자동 스크롤 중 hit-test 좌표와 선택 anchor/focus가 안정적으로 갱신되는지 확인한다.
3. E2E 또는 수동 검증 절차를 정리해 회귀 확인 기준을 남긴다.

## 승인 요청

Stage 2 변경 내용과 검증 결과를 승인해 주시면 Stage 3 자동 스크롤 보정 구현으로 진행한다.
