# Task 309 수행계획서: HWPX 미파싱 속성 전수 조사 및 파싱 완성

## 1. 현황

Task 308에서 표 캡션 skip으로 10px 렌더링 차이 발견.
HWPX 파서에서 XML에 명시된 속성이 IR에 미반영된 지점을 전수 조사.

## 2. 전수 조사 결과 (전문가 검토)

### P0 — 즉시 수정
- 표 `<pos>` 속성: **vertAlign, horzAlign** 미파싱 (그림/도형에서는 파싱함)

### P1 — 높은 영향
- `<secPr>` 내 **pageBorderFill** — 쪽 테두리/배경 (공공문서에서 빈번)

### P2 — 중간 영향
- `<secPr>` 내 **startNum** — 쪽번호 시작값
- `<secPr>` 내 **visibility** — 머리말/꼬리말 숨김
- **footNotePr / endNotePr** — 각주/미주 설정

### P3 — 낮은 영향
- grid, lineNumberShape, flowWithText, allowOverlap, zOrder, noAdjust

## 3. 구현 계획

### 3.1 단계 1: P0 수정 — 표 pos 속성 파싱
- `section.rs:549`의 `_ => {}`에 vertAlign, horzAlign 추가
- 그림/도형의 파싱 코드(961, 1202행)와 동일하게 구현

### 3.2 단계 2: P1 수정 — pageBorderFill
- secPr 자식 요소 pageBorderFill 파싱
- borderFillIDRef 참조 처리

### 3.3 단계 3: P2 수정 — startNum, visibility
- secPr 자식 요소 파싱

### 3.4 단계 4: 검증
- 비교 대조군 HWP ↔ HWPX IR 비교
- cargo test 716개 통과

## 4. 검토 이력
- 전문가 검토: 우선순위 분류 + 이미 특정된 것부터 즉시 수정 권장
