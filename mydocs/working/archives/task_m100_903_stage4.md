# task_m100_903 Stage 4 — grouped picture renderingInfo 직렬화

## 1. 배경

Stage 2에서는 HWPX embedded BinData가 HWP 저장 후 `Link`로 오인되어 payload가
사라지는 문제를 고쳤다.

Stage 3에서는 `hwpx-h-01.hwpx`의 묶음 그림에서 `<hp:curSz width="0" height="0">`
가 저장 가능한 HWP IR로 넘어오면서 effective size가 0이 되는 문제를 고쳤다.

그 결과 그림 payload와 기본 크기는 살아났지만, 정답 HWP와 비교하면 묶음 내부 그림의
`renderingInfo` scale/translation이 HWP 저장 후 identity matrix로 바뀌는 차이가 남았다.

## 2. 현상

HWPX 원본 파싱 IR과 한컴 정답 HWP는 문단 `0:29`의 묶음 그림 자식에서 다음 값을 갖는다.

```text
child[0] M=[0.724,0.000,0;     0.000,0.724,1133]
child[1] M=[0.518,0.000,9360;  0.000,0.446,0]
child[2] M=[1.472,0.000,38559; 0.000,1.287,474]
```

Stage 3 산출물을 HWP로 재로드하면 scale이 모두 1.0으로 바뀐다.

```text
child[0] M=[1.000,0.000,-310;  0.000,1.000,1133]
child[1] M=[1.000,0.000,9360;  0.000,1.000,0]
child[2] M=[1.000,0.000,39096; 0.000,1.000,657]
```

## 3. 원인 가설

HWPX 파서는 `<hp:renderingInfo>`를 `ShapeComponentAttr.render_*` 필드로 파싱한다.

하지만 HWP serializer의 `write_shape_component_base()`는 `raw_rendering`이 없으면
다음과 같은 기본 행렬을 새로 생성한다.

- translation: `offset_x`, `offset_y`
- scale: identity
- rotation: identity

즉 HWPX에서 파싱된 `render_sx`, `render_sy`, `render_tx`, `render_ty`가 HWP
`SHAPE_COMPONENT`의 rendering matrix로 직렬화되지 않는다.

## 4. Stage 4 목표

`raw_rendering`이 비어 있어도 `ShapeComponentAttr.render_*`가 identity가 아니면
그 값을 HWP rendering matrix로 직렬화한다.

검증 기준:

- HWPX 원본 파싱 IR의 묶음 그림 자식 `render_*` 값이 HWP 저장/재로드 후 유지된다.
- 기존 embedded BinData 보존 테스트는 계속 통과한다.
- 전체 `hwpx_to_hwp_adapter` 테스트가 통과한다.

## 5. RED 테스트

추가할 테스트:

```text
task903_hwpx_h_01_rendering_info_matrix_survives_hwp_save_reload
```

테스트 절차:

1. `samples/hwpx/hwpx-h-01.hwpx` 로드
2. 문단 `0:29` 묶음 그림의 자식 `ShapeComponentAttr.render_*` 값을 수집
3. adapter HWP 저장
4. 저장된 HWP를 재로드
5. 같은 위치의 자식 그림 `render_*` 값이 원본과 근사 일치하는지 확인

RED 결과:

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01_rendering_info_matrix_survives_hwp_save_reload -- --nocapture

child[0].render_sx: left=1 right=0.723629 delta=0.27637100000000003
```

## 6. 구현 방향

`write_shape_component_base()`에서 다음 우선순위를 적용한다.

1. `raw_rendering`이 있으면 기존처럼 원본 바이트를 보존한다.
2. `render_*`가 identity가 아니면 parsed affine matrix를 직렬화한다.
3. 나머지는 기존 fallback matrix를 사용한다.

parsed affine matrix는 HWP parser가 다시 합성했을 때 같은 결과가 나오도록
다음 형태로 기록한다.

```text
cnt = 1
translation = [1, 0, tx, 0, 1, ty]
scale       = [sx, b, 0, c, sy, 0]
rotation    = identity
```

## 7. 구현 결과

`src/serializer/control.rs`의 `write_shape_component_base()`에서 다음 분기를 추가했다.

- `raw_rendering`이 있으면 기존처럼 원본 바이트 보존
- `ShapeComponentAttr.render_*`가 identity가 아니면 parsed affine matrix 직렬화
- 그 외에는 기존 fallback matrix 사용

GREEN 결과:

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01_rendering_info_matrix_survives_hwp_save_reload -- --nocapture
# ok

cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01 -- --nocapture
# 3 passed

cargo test --test hwpx_to_hwp_adapter
# 34 passed

cargo test --lib
# 1248 passed; 0 failed; 2 ignored

git diff --check
# ok
```

## 8. Stage 4 산출물

```text
output/poc/hwpx2hwp/task903/stage4/hwpx-h-01_adapter.hwp
```

기본 정보:

```text
size: 680448 bytes
sections: 2
pages: 9
BinData: 5 records, 5 loaded payloads
```

문단 `0:29` 재로드 덤프:

```text
child[0] M=[0.724,0.000,0;     0.000,0.724,1133], eff=24.2mm×8.4mm
child[1] M=[0.518,0.000,9360;  0.000,0.446,0],    eff=98.0mm×13.1mm
child[2] M=[1.472,0.000,38559; 0.000,1.287,474],  eff=31.6mm×11.2mm
```

이 값은 HWPX 원본 파싱 IR 및 한컴 정답 HWP의 묶음 그림 matrix와 같은 수준이다.

## 9. 남은 차이

`ir-diff` 기준 문단 `0:29`의 주요 잔여 차이는 다음 쪽으로 남는다.

- `shape wrap`: `TopAndBottom` vs `Square`
- `shape vert_rel`: `Para` vs `Paper`
- `shape horz_rel`: `Para` vs `Paper`
- 일부 `ParaShape` indent/margin 차이

이번 Stage 4의 범위는 묶음 그림 내부 `renderingInfo` 보존까지로 제한한다.

## 10. 작업지시자 판정 요청

한컴 에디터와 rhwp-studio에서 다음 파일을 확인한다.

```text
output/poc/hwpx2hwp/task903/stage4/hwpx-h-01_adapter.hwp
```

판정 항목:

- 한컴 에디터에서 파일 손상 판정이 사라지는지
- 9페이지로 열리는지
- 문단 `0:29` 주변의 묶음 로고/그림이 정답 HWP와 유사한 크기와 배치로 보이는지
- rhwp-studio에서 다시 열었을 때 embedded image payload와 그림 배치가 유지되는지
