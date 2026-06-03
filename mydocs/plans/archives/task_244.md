# Task 244 수행 계획서: 고급 도형 편집

## 현재 상태

다각형/곡선/호/그룹 모두 모델+파서+직렬화+렌더링 완성.
생성 API와 UI만 추가하면 됨.

## 구현 범위

### 1. 도형 생성 API 확장 (create_shape_control_native)
- `"polygon"`: PolygonShape — 드래그 bbox를 기반으로 기본 다각형(삼각형/오각형) 생성
- `"arc"`: ArcShape — 타원 호 (기본: 반원)
- 곡선(curve)은 제어점 편집이 필요하므로 별도 고려
- 그룹(group)은 다중 선택 → 그룹화 커맨드로 처리

### 2. shape-picker UI 확장
- 기존 3종 + 호, 다각형 추가
- 그룹은 별도 커맨드 (Ctrl+G 등)

### 3. 그룹 개체
- 다중 선택 (Shift+클릭)
- 그룹화/해제 커맨드
