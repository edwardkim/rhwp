# Task 303 구현계획서: HWPX 이미지 배치 버그 수정

## 1. 현황 분석

### 1.1 문제
- 예제: `samples/hwpx/hang_job_01.hwpx` 3페이지
- 문단 0.36에 Picture(bin_data_id=2), `treat_as_char=false, wrap=TopAndBottom`
- 크기: 40758×31094 HU (약 543×414px)
- 이미지 높이를 무시하고 다음 문단이 출력되어 오버래핑

### 1.2 원인
`calculate_shape_reserved_heights()`에서 `Control::Shape`만 매칭하고 `Control::Picture`를 무시.
TopAndBottom Picture의 높이가 `shape_reserved`에 등록되지 않아 layout.rs의 y_offset 점프가 발생하지 않음.

### 1.3 스펙 확인
Picture와 Shape는 동일한 `CommonObjAttr`을 공유 → TopAndBottom 배치 규칙 동일.

## 2. 구현 계획

### 2.1 단계 1: calculate_shape_reserved_heights에 Picture/Equation 지원 추가
- match arm에 `Control::Picture(pic)` 추가, `pic.common` 사용
- Equation도 TopAndBottom 가능 여부 확인 후 필요 시 동일 처리
- `treat_as_char=true` Picture는 기존과 동일하게 제외 (LINE_SEG가 높이 포함)
- 좌표 계산 로직(v_offset, shape_y, bottom_y)을 공통 헬퍼 함수로 추출하여 중복 방지

### 2.2 단계 2: threshold_y 필터 검증
- 현재 `threshold_y = col_area.y + col_area.height / 3.0` 필터가 있음
- 대상 이미지가 이 필터를 통과하는지 확인
- 필요 시 Picture에 대해 필터 조건 조정

### 2.3 단계 3: 회귀 테스트
- HWPX: hang_job_01.hwpx 3페이지 이미지 배치 정상 확인
- HWP: 기존 TopAndBottom Picture 배치 회귀 확인
- TAC Picture (treat_as_char=true): 변경 없음 확인
- TopAndBottom Shape: 기존 동작 회귀 확인

## 3. 영향 범위
- `src/renderer/layout/shape_layout.rs` — calculate_shape_reserved_heights
- pagination 수정 불필요 (Shape도 process_controls에서 높이 미소비, shape_reserved 메커니즘으로 처리)

## 4. 검토 이력
- 전문가 검토: 단계 2(pagination 수정) 삭제 → shape_reserved만으로 충분
- Equation 검토, threshold_y 필터 검증 추가
