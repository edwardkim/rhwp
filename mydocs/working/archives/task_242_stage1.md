# Task 242 완료 보고서: 도형 기본 삽입 (직선/사각형/타원)

## 구현 항목

### Rust 백엔드
- **object_ops.rs**: `create_shape_control_native()` — `shape_type` 파라미터 추가
  - `"rectangle"`: 기존 Rectangle+TextBox (글상자)
  - `"ellipse"`: EllipseShape (center/axis 좌표 자동 계산, 흰색 채움)
  - `"line"`: LineShape (start/end 좌표, `lineFlipX/Y`로 방향 결정, 채움/TextBox 없음)
  - 도형 기본 배치: 글 앞으로(InFrontOfText) + 종이 기준(Paper) 왼쪽/위
- **render_tree.rs**: LineNode, EllipseNode에 section/para/control 인덱스 필드 추가
- **shape_layout.rs**: Line/Ellipse 렌더 노드에 인덱스 설정
- **rendering.rs**: `collect_controls`에서 Line/Ellipse를 선택 가능한 도형으로 수집
  - Line은 `"line"` 타입 + 시작/끝 좌표(x1,y1,x2,y2) 포함

### TypeScript 프론트엔드
- **shape-picker.ts**: 도형 선택 드롭다운 (직선/사각형/타원 3종)
- **shape-picker.css**: 드롭다운 스타일
- **input-handler.ts**:
  - `enterShapePlacementMode(shapeType)` — 도형 배치 모드
  - SVG 오버레이: 직선→line, 타원→ellipse, 사각형→rect
  - Shift+직선 → 0°/45°/90° 스냅
  - 드래그 중심점 기준 종이 좌표 오프셋 계산
- **input-handler-picture.ts**:
  - `pointToSegmentDist()` — 직선 히트: 점-선분 거리 5px 이내
  - `findPictureAtClick` — line 타입 별도 히트 판정
  - `renderPictureObjectSelection` — line 타입 → `renderLine()` 호출
- **table-object-renderer.ts**: `renderLine()` — 시작/끝점 2개 핸들 + 점선 가이드
- **cursor.ts**: `'line'` 타입 추가
- **insert.ts**: `insert:shape` 커맨드 활성화
- **index.html**: 도형 버튼 `data-cmd` + 메뉴 활성화

## 검증
- Rust 테스트 716개 통과
- TypeScript 컴파일 오류 없음
- WASM 빌드 성공
- 3종 도형 삽입/선택/리사이즈/이동/삭제 동작 확인
