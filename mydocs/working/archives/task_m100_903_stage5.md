# task_m100_903 Stage 5 — picture CommonObjAttr attr packing

## 1. 작업지시자 판정

Stage 4 산출물:

```text
output/poc/hwpx2hwp/task903/stage4/hwpx-h-01_adapter.hwp
```

판정 결과:

- 한컴 에디터: 파일 손상
- rhwp-studio: 1페이지 표 안의 이미지는 렌더링됨
- rhwp-studio: 하지만 표의 배치가 엉망인 상태

## 2. 정답 HWP와 Stage 4 비교

첫 문단 `0:0`의 첫 표 구조를 비교했다.

정답 HWP:

```text
표: 1행×3열, 쪽나눔=RowBreak, outer_margin=1mm
셀[0] 그림: tac=true, wrap=TopAndBottom, vert=Para(off=0), horz=Para(off=0)
셀[2] 그림: tac=true, wrap=TopAndBottom, vert=Para(off=20745), horz=Column(off=7695)
```

Stage 4 산출물:

```text
표: 1행×3열, 쪽나눔=CellBreak, outer_margin=0mm
셀[0] 그림: tac=false, wrap=Square, vert=Paper(off=0), horz=Paper(off=0)
셀[2] 그림: tac=false, wrap=Square, vert=Paper(off=20745), horz=Paper(off=7695)
```

## 3. HWPX 원본 확인

HWPX 원본 파싱 IR은 그림 배치 속성을 이미 정답지와 같은 수준으로 갖고 있다.

```text
셀[0] 그림: tac=true, wrap=TopAndBottom, vert=Para(off=0), horz=Para(off=0)
셀[2] 그림: tac=true, wrap=TopAndBottom, vert=Para(off=20745), horz=Column(off=7695)
```

따라서 이 문제는 HWPX 파싱 누락이 아니라 HWP 저장/재로드 경로에서 그림
`CommonObjAttr.attr` 비트가 손실되는 문제로 본다.

## 4. 원인 가설

표는 `document_core::converters::common_obj_attr_writer`를 통해 `common.attr == 0`일 때
enum 필드에서 CTRL_HEADER attr 비트를 합성한다.

하지만 그림/도형은 `serializer/control.rs` 내부의 `serialize_common_obj_attr()`를 사용한다.
이 함수는 `common.attr`을 그대로 쓰기 때문에 HWPX 출처처럼 `common.attr == 0`인 경우
다음 enum 값이 HWP CTRL_HEADER에 반영되지 않는다.

- `treat_as_char`
- `text_wrap`
- `vert_rel_to`
- `horz_rel_to`
- 정렬/크기 기준

그 결과 저장 후 재로드하면 기본값인 `tac=false`, `Square`, `Paper`로 해석된다.

## 5. Stage 5 목표

HWP serializer의 `serialize_common_obj_attr()`도 `common.attr == 0`이면 enum 필드에서
attr 비트를 합성하도록 맞춘다.

검증 기준:

- `hwpx-h-01.hwpx` → adapter HWP 저장 → 재로드 후 첫 표 셀 안 그림의
  `treat_as_char`, `text_wrap`, `vert_rel_to`, `horz_rel_to`가 원본과 일치한다.
- Stage 2/3/4의 BinData, current size, rendering matrix 테스트는 계속 통과한다.
- 전체 `hwpx_to_hwp_adapter` 테스트가 통과한다.

## 6. RED 테스트

추가할 테스트:

```text
task903_hwpx_h_01_table_cell_picture_common_attrs_survive_hwp_save_reload
```

절차:

1. HWPX 원본에서 첫 표의 셀 그림 2개 `CommonObjAttr`를 수집
2. adapter HWP 저장 후 재로드
3. 같은 위치의 그림 2개 `CommonObjAttr` 비교

예상 RED:

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01_table_cell_picture_common_attrs_survive_hwp_save_reload -- --nocapture

assertion `left == right` failed: cell picture[0].treat_as_char
  left: false
 right: true
cell picture[0].text_wrap: source=TopAndBottom, reloaded=Square
```

## 7. 추가 RED — 첫 표 한컴 layout attr

Stage 5에서 셀 안 그림 배치 속성은 복구됐지만, 첫 표 자체에는 다음 차이가 남았다.

```text
정답 HWP: page_break=RowBreak, outer_margin=(283,283,283,283)
Stage 5 이전: page_break=CellBreak, outer_margin=(0,0,0,0)
```

추가 테스트:

```text
task903_hwpx_h_01_first_table_hancom_layout_attrs_are_materialized
```

RED:

```text
assertion `left == right` failed: 첫 표 page_break는 한컴 정답 HWP와 같은 RowBreak로 materialize되어야 함
  left: CellBreak
 right: RowBreak
```

## 8. 구현 결과

수정 1 — 그림/도형 CommonObjAttr attr packing:

- `src/serializer/control.rs`의 `serialize_common_obj_attr()`가 `common.attr != 0`이면 원본 attr을 보존한다.
- `common.attr == 0`이면 `pack_common_attr_bits(common)`으로 enum 필드 기반 attr을 합성한다.
- HWPX 출처 그림/도형의 `treat_as_char`, `text_wrap`, `vert_rel_to`, `horz_rel_to`가 HWP 저장 후 재로드에서 유지된다.

수정 2 — TAC 표 한컴 layout attr materialization:

- `should_materialize_tac_table_ctrl_attr()` 대상 표에도 `outer_margin`을 CTRL_HEADER CommonObjAttr margin으로 승격한다.
- `CellBreak`를 한컴 정답 HWP와 같은 `RowBreak`로 materialize한다.
- TABLE record attr도 다시 materialize한다.

## 9. GREEN 결과

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01_table_cell_picture_common_attrs_survive_hwp_save_reload -- --nocapture
# ok

cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01_first_table_hancom_layout_attrs_are_materialized -- --nocapture
# ok

cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01 -- --nocapture
# 5 passed

cargo test --test hwpx_to_hwp_adapter
# 36 passed

cargo test --lib
# 1248 passed; 0 failed; 2 ignored

git diff --check
# ok
```

## 10. Stage 5 산출물

```text
output/poc/hwpx2hwp/task903/stage5/hwpx-h-01_adapter.hwp
```

재로드 덤프 기준:

```text
표: 1행×3열, 쪽나눔=RowBreak (attr=0x04000006), outer_margin=1mm
셀[0] 그림: tac=true, wrap=TopAndBottom, vert=Para(off=0), horz=Para(off=0)
셀[2] 그림: tac=true, wrap=TopAndBottom, vert=Para(off=20745), horz=Column(off=7695)
```

정답 HWP와 Stage 5 산출물의 첫 문단 `0:0` IR diff에서 다음 차이는 제거되었다.

- `tbl page_break`
- `tbl outer_margin`
- 셀 안 그림의 `treat_as_char`, `text_wrap`, `vert_rel_to`, `horz_rel_to`

남은 첫 문단 차이는 주로 `char_shapes count`와 전역 `ParaShape` indent/margin 계열이다.

## 11. 작업지시자 판정 요청

다음 파일을 확인한다.

```text
output/poc/hwpx2hwp/task903/stage5/hwpx-h-01_adapter.hwp
```

판정 항목:

- 한컴 에디터에서 파일 손상 판정이 사라지는지
- rhwp-studio에서 1페이지 첫 표 배치가 Stage 4보다 개선되었는지
- 첫 표 안 양쪽 이미지가 정답 HWP와 유사한 위치/크기로 보이는지
- 페이지 수가 9페이지로 유지되는지

## 12. 작업지시자 판정 결과

판정:

- 한컴 에디터: 파일 손상
- rhwp-studio: 전혀 개선되지 않음
- 문제 양상: 문단과 표의 배치 케이스가 무너짐

Stage 5의 첫 표 국소 수정은 전체 페이지 조판 붕괴를 해결하지 못했다.
다음 단계는 구현을 멈추고 정답 HWP와 Stage 5 HWP의 1페이지 문단, line_seg,
표 배치 차이를 먼저 수집한다.
