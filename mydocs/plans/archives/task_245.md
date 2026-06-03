# Task 245 수행 계획서: 연결선($col) 구현

## 조사 결과

### 바이너리 구조
- 연결선은 **별도 태그가 아닌 SC_LINE(HWPTAG_SHAPE_COMPONENT_LINE)을 재사용**
- SHAPE_COMPONENT의 ctrl_id로 `$col`(0x24636f6c)을 사용하여 일반 선과 구분
- CTRL_HEADER는 `gso `(일반 GSO)와 동일하지만 description이 "직선 연결선입니다." 등

### SC_LINE 데이터 구조 (hwplib 기준)

| 오프셋 | 타입 | 필드 | 비고 |
|--------|------|------|------|
| 0 | i32 | startX | 시작점 X |
| 4 | i32 | startY | 시작점 Y |
| 8 | i32 | endX | 끝점 X |
| 12 | i32 | endY | 끝점 Y |
| 16 | u32 | type | LinkLineType (0~8) |
| 20 | u32 | startSubjectID | 시작 연결 개체 instance_id |
| 24 | u32 | startSubjectIndex | 시작 연결점 인덱스 |
| 28 | u32 | endSubjectID | 끝 연결 개체 instance_id |
| 32 | u32 | endSubjectIndex | 끝 연결점 인덱스 |
| 36 | u32 | countOfControlPoints | 제어점 수 |
| 40+ | CP[] | x(u32)+y(u32)+type(u16) × count | 꺽인/곡선용 |
| 끝 | u32 | padding | 0x00000000 |

- 일반 LineShape: 20바이트 (startX/Y, endX/Y, started_right_or_bottom)
- 연결선: 40+ 바이트 (type 필드 이후 연결 정보 추가)

### LinkLineType 열거형 (9종)

| 값 | 이름 | 설명 |
|----|------|------|
| 0 | Straight_NoArrow | 직선 연결선 |
| 1 | Straight_OneWay | 직선 화살표 연결선 |
| 2 | Straight_Both | 직선 양쪽 화살표 연결선 |
| 3 | Stroke_NoArrow | 꺽인 연결선 |
| 4 | Stroke_OneWay | 꺽인 화살표 연결선 |
| 5 | Stroke_Both | 꺽인 양쪽 화살표 연결선 |
| 6 | Arc_NoArrow | 곡선 연결선 |
| 7 | Arc_OneWay | 곡선 화살표 연결선 |
| 8 | Arc_Both | 곡선 양쪽 화살표 연결선 |

### 제약 사항
- 같은 쪽(페이지)에 있는 개체만 연결 가능
- SubjectID는 개체의 instance_id를 참조

### 참조
- HWPML 스펙: 표 141 CONNECTLINE 엘리먼트 (hwp_spec_3.0_hwpml.md:4563)
- HWP 5.0 바이너리 스펙: 미문서화 (SC_LINE 재사용)
- 한컴 도움말: draw/draw/drawing(connect).htm
- hwplib: ForControlObjectLinkLine.java, ShapeComponentLineForObjectLinkLine.java
- 예제: samples/cline-00~03.hwp

## 구현 계획

### 1단계: 모델 + 파서 + 렌더러 (뷰어)
- Rust 모델: `ConnectorLine` 구조체 (LineShape 확장 또는 별도)
- 파서: ctrl_id='$col' 감지 → SC_LINE 확장 데이터 파싱
- 렌더러: 직선/꺽인/곡선 연결선 SVG 경로 생성
- 직렬화: SC_LINE 확장 데이터 기록

### 2단계: 편집 기능
- 연결선 생성 UI (도형 드롭다운에 연결선 타입 추가)
- 개체 연결점(connection point) 히트 테스트
- 연결된 개체 이동 시 연결선 자동 추적

### 3단계: 직렬화 + 한컴 호환
- 연결선 저장/로드 라운드트립
- 한컴에서 열기/저장 호환성 테스트
