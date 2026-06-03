# Task 249 수행 계획서: 그룹 개체 (다중 선택 + 묶기/풀기)

## 한컴 동작 (도움말 기준)

1. "개체 선택" 모드에서 마우스 드래그로 영역 내 개체 다중 선택
2. "개체 묶기" 커맨드로 선택된 개체들을 GroupShape로 합침
3. "개체 풀기" 커맨드로 GroupShape를 개별 개체로 분리
4. 제한: 같은 쪽, 표는 묶기 불가

## 구현 계획

### 1단계: 다중 선택
- Shift+클릭으로 개체 추가 선택
- cursor에 다중 선택 상태 관리 (selectedPictureRefs: 배열)
- 다중 선택 시 모든 개체의 bbox를 포함하는 합산 핸들 표시

### 2단계: 개체 묶기 (Group)
- Rust: group_shapes_native(sec, para_indices, ctrl_indices) → GroupShape 생성
- 선택된 개체들을 GroupShape.children으로 이동
- WASM API + 커맨드 등록

### 3단계: 개체 풀기 (Ungroup)
- Rust: ungroup_shape_native(sec, para, ctrl_idx) → GroupShape를 개별 개체로 분리
- WASM API + 커맨드 등록
