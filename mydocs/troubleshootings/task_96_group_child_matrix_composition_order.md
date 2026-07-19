---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 트러블슈팅: 그룹(객체 묶음) 자식 도형 좌표 변환 오류

## 문제 상황

HWP 문서에서 여러 도형을 그룹으로 묶은 경우, 자식 도형들이 그룹 경계 사각형 위쪽으로 약 100px 벗어나서 렌더링됨.

### 증상

- `samples/basic/docdo.hwp` (독도 그림 + 사각형을 그룹으로 묶은 문서)
- 사각형은 정상 위치 (y≈257)에 렌더링되나, 독도 폴리곤들은 위쪽 (y≈194)에 렌더링
- 동일 도형을 그룹 해제한 `docdo-1.hwp`는 정상 렌더링 (y≈294)

### 비교 (수정 전)

| 도형 | 그룹 (잘못됨) | 비그룹 (정상) | 차이 |
|------|--------------|--------------|------|
| 독도 P1 | y≈194 | y≈294 | ~100px |
| 독도 P6 | y≈183 | y≈283 | ~100px |
| 사각형 | y≈257 | y≈257 | 0px |

## 분석 과정

### 1단계: 그룹 vs 비그룹 데이터 비교

그룹/비그룹 문서의 ShapeComponentAttr를 비교하여 다음을 확인:
- 그룹 CommonObjAttr 크기 == ShapeComponentAttr 크기 (4950×5850) — 스케일링 차이 없음
- 자식 도형의 `position_in_group = grouped_offset - ungrouped_offset` 관계 성립
- 사각형(회전 없음)은 두 버전 모두 정상 → 회전 성분이 있는 도형만 문제

### 2단계: 렌더링 행렬 분석

HWP 스펙 표 86-87에 따른 렌더링 행렬 구조:
```
Translation × (Scale × Rotation) × cnt쌍
```

독도 P1의 행렬 데이터:
```
T = [1, 0, -16935; 0, 1, -6942]     # Translation
S₁ = [5.290, 0, 3229; 0, 5.394, -4683]  # Scale (cnt=1의 2번째 쌍)
R₁ = [0.999, -0.011, 0; -0.011, 0.999, 0]  # Rotation
```

### 3단계: 행렬 합성 순서 검증 (핵심)

**기존 코드 (잘못됨)**: `T × S × R` 순서
```
result = compose(T, S₁)  →  tx = 3229 (S의 tx 유지)
result = compose(result, R₁)  →  ty ≈ -4683 (음수 → 위쪽으로 벗어남)
```

**올바른 순서**: `T × R × S` 순서
```
result = compose(T, R₁)  →  tx ≈ -16935, ty ≈ -6942 (T 유지)
result = compose(result, S₁)  →  tx ≈ 1865, ty ≈ 2812 (양수 → 정상 위치)
```

수동 계산으로 `T × R × S`의 결과가 실제 `position_in_group` 값 (1852, 2798)과 약 13-18 HWPUNIT 이내로 일치함을 확인. 소수점 3자리 정밀도의 수동 계산 오차 범위.

## 근본 원인

`src/parser/control.rs`의 `parse_shape_component_full()` 함수에서 렌더링 행렬 합성 시 Scale과 Rotation의 적용 순서가 반대였다.

### 잘못된 코드

```rust
for i in 0..cnt {
    let scale = read_matrix(&mut r);
    let rotation = read_matrix(&mut r);
    result = compose(&result, &scale);     // Scale 먼저
    result = compose(&result, &rotation);  // Rotation 나중
}
```

이 순서로 합성하면 Scale 행렬의 translation 성분(tx, ty)이 Rotation에 의해 변형되어 음수 좌표가 생성됨.

### 올바른 순서

```rust
for _ in 0..cnt {
    let scale = read_matrix(&mut r);
    let rotation = read_matrix(&mut r);
    result = compose(&result, &rotation);  // Rotation 먼저
    result = compose(&result, &scale);     // Scale 나중
}
```

Rotation을 먼저 적용하면 Translation의 이동 성분이 보존되고, 이후 Scale이 적용되어 올바른 좌표가 산출됨.

## 수정 내용

| 파일 | 수정 |
|------|------|
| `src/parser/control.rs` (680-681행) | 행렬 합성 순서를 `R → S`로 변경 |

## 수정 결과

| 도형 | 그룹 (수정 후) | 비그룹 (참조) | 차이 |
|------|---------------|--------------|------|
| 독도 P1 | y≈294.41 | y≈294.29 | ~0.1px |
| 독도 P6 | y≈282.92 | y≈282.80 | ~0.1px |
| 사각형 | y≈256.99 | y≈256.99 | 0px |

~0.1px 차이는 아핀 변환 경로와 직접 좌표 경로 간의 부동소수점 연산 차이로 인한 것이며, 시각적으로 구분 불가.

## 교훈

- 아핀 변환 행렬 합성에서 **순서가 결과를 결정**한다. `A × B ≠ B × A`
- 회전이 없는 도형(사각형)은 Scale 행렬이 대각 행렬이므로 순서에 무관하게 같은 결과를 내어 문제가 드러나지 않았음
- 회전 성분이 있는 도형에서만 순서 차이가 발현됨 — 테스트 시 회전 도형 포함 필수
- 그룹 해제된 동일 문서를 참조 기준으로 활용하여 정확한 좌표 비교가 가능했음
