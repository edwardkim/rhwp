# Task m100 #903 Stage 3 - 묶음 그림 current size materialize

## 1. 목표

Stage 2 이후 `rhwp info`에서 다음 문제가 보였다.

```text
도형 [구역0:문단29]: 묶음
  자식[0]: 그림(묶음내), orig=9480×3300, scale=(1.000,1.000), eff=0×0
  자식[1]: 그림(묶음내), orig=53640×8340, scale=(1.000,1.000), eff=0×0
  자식[2]: 그림(묶음내), orig=6082×2457, scale=(1.000,1.000), eff=0×0
```

정답 HWP는 같은 문단에서 묶음/자식 그림의 `current size`가 0이 아니다.

## 2. 비교

명령:

```text
cargo run --bin rhwp -- dump samples/hwpx/hancom-hwp/hwpx-h-01.hwp -s 0 -p 29
cargo run --bin rhwp -- dump output/poc/hwpx2hwp/task903/stage2/hwpx-h-01_adapter.hwp -s 0 -p 29
cargo run --bin rhwp -- dump samples/hwpx/hwpx-h-01.hwpx -s 0 -p 29
```

관찰:

```text
한컴 정답 HWP:
  group curr=47509×3721
  child[0] curr=9480×3300, M scale=(0.724, 0.724)
  child[1] curr=53640×8340, M scale=(0.518, 0.446)
  child[2] curr=6082×2457, M scale=(1.472, 1.287)

HWPX 원본:
  group curr=0×0
  child current=0×0
  orgSz와 renderingInfo scale은 존재

Stage 2 저장본:
  group curr=0×0
  child current=0×0
  저장 후 effective size 0
```

원인:

```text
parse_shape_object()에는 curSz=0이면 orgSz로 current size를 보강하는 폴백이 있었다.
parse_picture()와 parse_container()에는 같은 폴백이 없었다.
```

## 3. 수정

파일:

```text
src/parser/hwpx/section.rs
```

추가:

```text
materialize_shape_current_size_from_original()
```

적용 대상:

```text
parse_picture()
parse_shape_object()
parse_container()
```

정책:

```text
shape_attr.current_width == 0 && original_width > 0 이면 current_width = original_width
shape_attr.current_height == 0 && original_height > 0 이면 current_height = original_height
common.width/height가 0이면 같이 보강
```

## 4. 테스트

추가 테스트:

```text
task903_hwpx_h_01_group_picture_current_size_is_materialized
```

RED:

```text
child[0] current size must be materialized from orgSz when curSz is zero
```

GREEN:

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01_group_picture_current_size_is_materialized -- --nocapture
=> ok
```

#903 관련 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01 -- --nocapture
=> 2 passed
```

어댑터 통합 테스트:

```text
cargo test --test hwpx_to_hwp_adapter
=> 33 passed
```

## 5. Stage 3 산출물

```text
output/poc/hwpx2hwp/task903/stage3/hwpx-h-01_adapter.hwp
```

요약:

```text
bytes: 680960
pages: 9
bindata_records: 5
bindata_content: 5
```

문단 0:29 재로드 결과:

```text
group curr=47509×3721, eff=167.6mm×13.1mm
child[0] curr=9480×3300, eff=33.4mm×11.6mm
child[1] curr=53640×8340, eff=189.2mm×29.4mm
child[2] curr=6082×2457, eff=21.5mm×8.7mm
```

## 6. 남은 차이

Stage 3는 zero-size 문제를 해결했다.

하지만 정답 HWP와 아직 다르다.

```text
한컴 정답 HWP:
  child[0] M scale=(0.724, 0.724), eff=24.2mm×8.4mm
  child[1] M scale=(0.518, 0.446), eff=98.0mm×13.1mm
  child[2] M scale=(1.472, 1.287), eff=31.6mm×11.2mm

Stage 3 저장본:
  child[0] M scale=(1.000, 1.000), eff=33.4mm×11.6mm
  child[1] M scale=(1.000, 1.000), eff=189.2mm×29.4mm
  child[2] M scale=(1.000, 1.000), eff=21.5mm×8.7mm
```

해석:

```text
HWPX 파서는 renderingInfo scale 값을 IR에 보유한다.
하지만 HWP 직렬화 시 raw_rendering이 없으면 identity matrix를 생성한다.
따라서 Stage 4 후보는 parsed renderingInfo matrix를 SHAPE_COMPONENT rendering_info로 직렬화하는 것이다.
```

## 7. 작업지시자 판정 요청

다음 파일을 확인한다.

```text
output/poc/hwpx2hwp/task903/stage3/hwpx-h-01_adapter.hwp
```

판정 항목:

- 한컴 에디터 파일 손상 판정 여부
- 묶음 그림이 출력되는지
- 0 크기/누락 그림 현상이 사라졌는지
- 묶음 그림의 배율이 정답 HWP 대비 과대/과소인지

## 8. Stage 4 후보

```text
SHAPE_COMPONENT rendering_info 직렬화에서 shape_attr.render_sx/render_sy/render_tx/render_ty를 반영한다.
```

주의:

```text
현재 serializer/control.rs는 raw_rendering이 없으면 identity scale matrix를 기록한다.
이 주석은 "스케일은 current/original으로 표현"한다는 가정인데,
HWPX grouped picture는 org/current와 renderingInfo scale을 동시에 사용한다.
```
