# Task 341 수행계획서: TAC 수식 인라인 렌더링

## 현상

`samples/exam_math.hwp`에서 수식(Equation)이 `treat_as_char=true`로 문단 텍스트 안에 인라인 배치되어야 하지만, 현재는 텍스트와 분리되어 별도 위치에 렌더링됨.

예: pi=18 "2. 함수 [수식] 에 대하여 [수식] 의 값은?" — 수식이 텍스트 흐름 안에 있어야 함.

## 원인 분석

### 현재 흐름
1. **Composer** (composer.rs:110-135): 수식은 `tac_controls`에 항상 등록됨 ✓
2. **Paragraph Layout** (paragraph_layout.rs:562-569): `tac_offsets_px`로 수식 폭 전달 ✓
3. **Paragraph Layout**: 수식 위치를 `tree.set_inline_shape_position()`으로 **등록하지 않음** ✗
4. **Shape Layout** (shape_layout.rs:202-206): `get_inline_shape_position()` 조회 → None → 단독 배치

### 비교: TAC Picture/Shape
- Picture/Shape는 `paragraph_layout.rs`에서 `set_inline_shape_position()`으로 위치 등록
- Shape Layout에서 해당 좌표를 사용하여 인라인 배치
- **수식에는 이 등록 로직이 없음**

## 구현 계획

### 1단계: paragraph_layout에서 수식 인라인 위치 등록

- `tac_offsets_px` 순회 시 수식 컨트롤에 대해 `set_inline_shape_position()` 호출
- Picture/Shape와 동일한 패턴으로 처리

### 2단계: shape_layout에서 인라인 좌표 사용

- 수식 렌더링 시 `get_inline_shape_position()` 조회
- 좌표가 있으면 해당 위치에 수식 배치 (베이스라인 정렬 포함)
- 좌표가 없으면 기존 방식(단독 배치) 유지

### 3단계: 검증

- exam_math.hwp 전체 페이지 SVG 비교
- 수식이 텍스트 흐름 안에 올바르게 배치되는지 확인
- 기존 테스트 716건 통과 확인
- exam_kor, hwpspec-w.hwp 등 기존 샘플 영향 없음 확인

## 영향 범위

- `src/renderer/layout/paragraph_layout.rs` — 수식 인라인 위치 등록
- `src/renderer/layout/shape_layout.rs` — 수식 인라인 좌표 사용
